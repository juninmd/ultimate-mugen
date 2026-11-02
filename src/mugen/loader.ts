import { parseAir, type Animation } from './air';
import { decodeSff } from './sff';
import { decodeSnd } from './snd';
import { decodeAct } from './act';
import { spriteToRGBA, type DecodedSff } from './sprite';
import { findSection, parseIni, sectionGet, unquote, nums } from './ini';
import { parseCmd, parseConstants, parseStates, scaleDefaultConstants } from './cns';
import { defaultConstants, type CharacterDef, type Sprite } from '../engine/chardef';
import { parseCommand } from '../engine/commands';
import type { StageInfo, Match } from '../engine/match';

export type VFS = Map<string, Uint8Array>;
export type ImageFactory = (rgba: Uint8ClampedArray, w: number, h: number) => CanvasImageSource | null;

export const normPath = (p: string) => p.replace(/\\/g, '/').replace(/^\.\//, '').replace(/\/+/g, '/').toLowerCase();

const td = new TextDecoder('latin1');
export function text(buf: Uint8Array | undefined): string { return buf ? td.decode(buf) : ''; }

export function resolveFile(vfs: VFS, baseDir: string, name: string): Uint8Array | undefined {
  if (!name) return undefined;
  const n = normPath(unquote(name));
  const tries = [baseDir ? `${baseDir}/${n}` : n, n, `data/${n}`];
  for (const t of tries) { const v = vfs.get(normPath(t)); if (v) return v; }
  // fallback: basename anywhere in the folder
  const base = n.split('/').pop()!;
  for (const [k, v] of vfs) if (k.startsWith(baseDir) && k.endsWith('/' + base)) return v;
  for (const [k, v] of vfs) if (k.endsWith('/' + base) || k === base) return v;
  return undefined;
}

/** Find character .def files inside a VFS (excluding storyboards). */
export function findCharDefs(vfs: VFS): string[] {
  const out: string[] = [];
  for (const [k, v] of vfs) {
    if (!k.endsWith('.def')) continue;
    const t = text(v).toLowerCase();
    if (t.includes('[files]') && /\bcns\s*=/.test(t) && /\bsprite\s*=/.test(t)) out.push(k);
  }
  return out;
}
export function findStageDefs(vfs: VFS): string[] {
  const out: string[] = [];
  for (const [k, v] of vfs) {
    if (!k.endsWith('.def')) continue;
    const t = text(v).toLowerCase();
    if (t.includes('[bgdef') && (t.includes('[stageinfo]') || t.includes('[camera]'))) out.push(k);
  }
  return out;
}

function buildSprites(sff: DecodedSff, makeImage: ImageFactory, palOverride: Uint8ClampedArray | null, palTarget: number): Map<string, Sprite> {
  const map = new Map<string, Sprite>();
  for (const s of sff.sprites) {
    const key = `${s.group},${s.index}`;
    if (map.has(key)) continue;
    let pal = s.palette;
    if (palOverride && s.paletteIndex === palTarget && s.group !== 9000) pal = palOverride;
    const rgba = spriteToRGBA(s, pal);
    map.set(key, { image: s.width && s.height ? makeImage(rgba, s.width, s.height) : null, width: s.width, height: s.height, axisX: s.axisX, axisY: s.axisY });
  }
  return map;
}

export interface LoadCharOptions { palNo?: number; makeImage: ImageFactory; id?: string }

export function loadMugenCharacter(vfs: VFS, defPath: string, opt: LoadCharOptions): CharacterDef {
  defPath = normPath(defPath);
  const baseDir = defPath.includes('/') ? defPath.slice(0, defPath.lastIndexOf('/')) : '';
  const defText = text(vfs.get(defPath));
  const secs = parseIni(defText);
  const info = findSection(secs, 'info');
  const files = findSection(secs, 'files');
  const f = (k: string) => sectionGet(files, k);
  const name = unquote(sectionGet(info, 'name')) || defPath;
  const displayName = unquote(sectionGet(info, 'displayname')) || name;
  const author = unquote(sectionGet(info, 'author'));
  const lc = nums(sectionGet(info, 'localcoord'), [320, 240]);

  const constants = defaultConstants();
  const states = new Map();
  // constants + states
  const cnsText = text(resolveFile(vfs, baseDir, f('cns') ?? ''));
  const provided = new Set<string>();
  parseConstants(cnsText, constants, provided);
  scaleDefaultConstants(constants, provided, (lc[0] || 320) / 320);
  parseStates(cnsText, states);
  for (const k of ['st', 'st0', 'st1', 'st2', 'st3', 'st4', 'st5', 'st6', 'st7', 'st8', 'st9']) {
    const fn = f(k);
    if (!fn || normPath(unquote(fn)) === normPath(unquote(f('cns') ?? ''))) continue;
    const t = text(resolveFile(vfs, baseDir, fn));
    if (t) parseStates(t, states);
  }
  // commands (+ statedef -1)
  const cmdText = text(resolveFile(vfs, baseDir, f('cmd') ?? ''));
  const cmd = parseCmd(cmdText);
  for (const [id, sd] of cmd.states) {
    if (states.has(id) && id < 0) states.get(id).controllers.push(...sd.controllers);
    else states.set(id, sd);
  }
  // character-provided common file (only if inside the char folder)
  const stc = f('stcommon');
  if (stc) {
    const local = vfs.get(normPath(`${baseDir}/${unquote(stc)}`));
    if (local) parseStates(text(local), states, false);
  }
  const commands = cmd.commands;
  const has = (n: string) => commands.some((c) => c.name === n);
  if (!has('holdfwd')) commands.push(parseCommand('holdfwd', '/$F', 1, 1));
  if (!has('holdback')) commands.push(parseCommand('holdback', '/$B', 1, 1));
  if (!has('holdup')) commands.push(parseCommand('holdup', '/$U', 1, 1));
  if (!has('holddown')) commands.push(parseCommand('holddown', '/$D', 1, 1));

  // animations
  const anims: Map<number, Animation> = parseAir(text(resolveFile(vfs, baseDir, f('anim') ?? '')));

  // sprites + palettes
  const sffBuf = resolveFile(vfs, baseDir, f('sprite') ?? '');
  let sprites = new Map<string, Sprite>();
  if (sffBuf) {
    const sff = decodeSff(sffBuf);
    const palNo = opt.palNo ?? 1;
    let override: Uint8ClampedArray | null = null;
    let target = -1;
    // find the palette most sprites use (the shared one)
    const counts = new Map<number, number>();
    for (const s of sff.sprites) if (s.group !== 9000) counts.set(s.paletteIndex, (counts.get(s.paletteIndex) ?? 0) + 1);
    let best = -1, bestN = -1;
    for (const [k, n] of counts) if (n > bestN) { best = k; bestN = n; }
    target = best;
    if (sff.version === 1) {
      const act = f(`pal${palNo}`);
      const buf = act ? resolveFile(vfs, baseDir, act) : undefined;
      if (buf && buf.length >= 768) override = decodeAct(buf);
    } else {
      const p = sff.palettes.find((p) => p.group === 1 && p.index === palNo);
      if (p && palNo !== 1) override = p.data;
      const actName = f(`pal${palNo}`);
      if (!p && actName) { const buf = resolveFile(vfs, baseDir, actName); if (buf && buf.length >= 768) override = decodeAct(buf); }
    }
    sprites = buildSprites(sff, opt.makeImage, override, target);
  }
  // sounds
  const sndBuf = resolveFile(vfs, baseDir, f('sound') ?? '');
  const sounds = new Map<string, Uint8Array>();
  if (sndBuf) for (const [k, v] of decodeSnd(sndBuf)) sounds.set(k, v);

  const def: CharacterDef = {
    id: opt.id ?? `mugen:${defPath}`,
    name, displayName, author, source: 'mugen',
    localW: lc[0] || 320, localH: lc[1] || 240,
    constants, sprites, anims, commands, states, sounds: sounds as any,
    portrait: sprites.get('9000,0'), bigPortrait: sprites.get('9000,1'),
    aiCommands: commands.map((c) => c.name).filter((n) => !/^hold|^recovery|^fwd|^back|^up|^down|^ff$|^bb$/i.test(n)),
  };
  const movelist = resolveFile(vfs, baseDir, f('movelist') ?? '');
  if (movelist) def.description = text(movelist).slice(0, 400);
  return def;
}

// ------------------------------------------------------------------ stages

interface BgLayer {
  type: string; sprite: [number, number]; start: [number, number]; delta: [number, number];
  tile: [number, number]; tileSpacing: [number, number]; velocity: [number, number]; layer: number;
  trans: string; anim?: Animation; mask: boolean; off: [number, number];
  t: number;
}

export function loadMugenStage(vfs: VFS, defPath: string, makeImage: ImageFactory): StageInfo {
  defPath = normPath(defPath);
  const baseDir = defPath.includes('/') ? defPath.slice(0, defPath.lastIndexOf('/')) : '';
  const t = text(vfs.get(defPath));
  const secs = parseIni(t);
  const g = (s: string, k: string, d: number[]) => nums(sectionGet(findSection(secs, s), k), d);
  const name = unquote(sectionGet(findSection(secs, 'info'), 'displayname') ?? sectionGet(findSection(secs, 'info'), 'name')) || defPath;
  const lc = g('stageinfo', 'localcoord', [320, 240]);
  const scl = 320 / (lc[0] || 320);
  const zoffset = g('stageinfo', 'zoffset', [200])[0];
  const bgdef = secs.find((s) => s.lname === 'bgdef');
  const sprFile = sectionGet(bgdef, 'spr');
  const sffBuf = sprFile ? resolveFile(vfs, baseDir, sprFile) ?? resolveFile(vfs, 'stages', sprFile) : undefined;
  const sprites = new Map<string, Sprite>();
  if (sffBuf) {
    const sff = decodeSff(sffBuf);
    for (const s of sff.sprites) {
      const k = `${s.group},${s.index}`;
      if (sprites.has(k)) continue;
      sprites.set(k, { image: makeImage(spriteToRGBA(s), s.width, s.height), width: s.width, height: s.height, axisX: s.axisX, axisY: s.axisY });
    }
  }
  const anims = parseAir(t);
  const layers: BgLayer[] = [];
  for (const s of secs) {
    if (!/^bg\s/i.test(s.name) && s.lname !== 'bg') continue;
    const type = (sectionGet(s, 'type') ?? 'normal').toLowerCase();
    const n = (k: string, d: number[]) => nums(sectionGet(s, k), d);
    const L: BgLayer = {
      type, sprite: n('spriteno', [0, 0]) as [number, number], start: n('start', [0, 0]) as [number, number],
      delta: n('delta', [1, 1]) as [number, number], tile: n('tile', [0, 0]) as [number, number],
      tileSpacing: n('tilespacing', [0, 0]) as [number, number], velocity: n('velocity', [0, 0]) as [number, number],
      layer: n('layerno', [0])[0], trans: (sectionGet(s, 'trans') ?? 'none').toLowerCase(), mask: !!n('mask', [0])[0], off: [0, 0], t: 0,
    };
    if (L.delta.length < 2) L.delta[1] = L.delta[0];
    if (type === 'anim') L.anim = anims.get(n('actionno', [0])[0]);
    layers.push(L);
  }
  const draw = (front: boolean) => (ctx: CanvasRenderingContext2D, m: Match, scale: number) => {
    const camX = m.camera.x / scl, camY = m.camera.y / scl;
    for (const L of layers) {
      if ((L.layer === 1) !== front) continue;
      let spr: Sprite | undefined;
      if (L.anim) {
        const tt = m.gameTime;
        const tot = L.anim.totalTime > 0 ? L.anim.totalTime : 1;
        let acc = 0, idx = 0;
        const tm = tt % tot;
        for (let i = 0; i < L.anim.frames.length; i++) { const ft = L.anim.frames[i].time; if (ft < 0 || tm < acc + ft) { idx = i; break; } acc += ft; idx = i; }
        const fr = L.anim.frames[idx];
        spr = sprites.get(`${fr.group},${fr.index}`);
      } else spr = sprites.get(`${L.sprite[0]},${L.sprite[1]}`);
      if (!spr || !spr.image) continue;
      L.off[0] += L.velocity[0] / 60 * 0; // velocity handled by time below
      const vx = L.velocity[0] * m.gameTime, vy = L.velocity[1] * m.gameTime;
      const sx = (L.start[0] + vx - camX * L.delta[0]) * scl + m.viewW / 2;
      const sy = (L.start[1] + vy - camY * L.delta[1]) * scl + (m.viewH - 240) / 2;
      const w = spr.width * scl, h = spr.height * scl;
      const x0 = sx - spr.axisX * scl, y0 = sy - spr.axisY * scl;
      const tx = L.tile[0], ty = L.tile[1];
      const stepX = w + L.tileSpacing[0] * scl, stepY = h + L.tileSpacing[1] * scl;
      ctx.save();
      if (L.trans === 'add' || L.trans === 'add1') ctx.globalCompositeOperation = 'lighter';
      const xs: number[] = [], ys: number[] = [];
      if (tx === 1) {
        let start = x0 - Math.ceil(x0 / stepX) * stepX;
        for (let x = start; x < m.viewW; x += stepX) xs.push(x);
      } else if (tx > 1) for (let i = 0; i < tx; i++) xs.push(x0 + i * stepX);
      else xs.push(x0);
      if (ty === 1) {
        let start = y0 - Math.ceil(y0 / stepY) * stepY;
        for (let y = start; y < m.viewH; y += stepY) ys.push(y);
      } else if (ty > 1) for (let i = 0; i < ty; i++) ys.push(y0 + i * stepY);
      else ys.push(y0);
      for (const x of xs) for (const y of ys) ctx.drawImage(spr.image, x * scale, y * scale, w * scale + 0.5, h * scale + 0.5);
      ctx.restore();
    }
  };
  const bl = g('camera', 'boundleft', [-150])[0] * scl, br = g('camera', 'boundright', [150])[0] * scl;
  return {
    name,
    boundLeft: bl, boundRight: br,
    boundHigh: g('camera', 'boundhigh', [-25])[0] * scl, boundLow: g('camera', 'boundlow', [0])[0] * scl,
    p1x: g('playerinfo', 'p1startx', [-70])[0] * scl, p2x: g('playerinfo', 'p2startx', [70])[0] * scl,
    zoffset: zoffset * scl, verticalFollow: g('camera', 'verticalfollow', [0.2])[0], tension: g('camera', 'tension', [50])[0] * scl,
    drawBack: draw(false), drawFront: draw(true),
  };
}
