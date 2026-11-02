import type { Animation, AnimFrame, Box as ABox } from '../mugen/air';
import { computeTotal } from '../mugen/air';
import { defaultConstants, type CharacterDef, type Sprite } from '../engine/chardef';
import { parseCommand, type CommandDef } from '../engine/commands';
import { parseStates } from '../mugen/cns';
import type { NativeChar, NativeMove, HitSpec, ProjectileSpec, NativeFrame } from './types';
import { renderPose, renderEffect, type CanvasFactory } from './rig';

/** Standard animation names → MUGEN action numbers */
export const STANDARD_ANIMS: Record<string, number> = {
  idle: 0, turn: 5, crouchTurn: 6, crouchStart: 10, crouch: 11, crouchEnd: 12, walk: 20, walkBack: 21,
  jumpStart: 40, jumpUp: 41, jumpFwd: 42, jumpBack: 43, jumpDown: 44, land: 47, run: 100, hop: 105,
  guardStart: 120, crouchGuardStart: 121, airGuardStart: 122, guard: 130, crouchGuard: 131, airGuard: 132,
  guardEnd: 140, guardHit: 150, crouchGuardHit: 152, airGuardHit: 154, lose: 170, draw: 175, win: 181, intro: 190,
  hitLight: 5000, hitMedium: 5001, hitHeavy: 5002, hitLowLight: 5010, hitLowMedium: 5011, hitLowHeavy: 5012,
  hitCrouch: 5020, hitAir: 5030, fall: 5050, bounce: 5100, bounceAir: 5101, lieDown: 5110, getUp: 5120, dead: 5150,
  trip: 5070, airRecover: 5040,
};

const LEVEL = { light: { dmg: 1, stop: 9, stun: 12, kb: -3.5, anim: 'Light' }, medium: { dmg: 1, stop: 11, stun: 16, kb: -4.5, anim: 'Medium' }, heavy: { dmg: 1, stop: 13, stun: 20, kb: -6, anim: 'Hard' } };
const GUARDFLAG: Record<string, string> = { high: 'HA', low: 'L', mid: 'MA', air: 'MA', unblockable: '' };

function hitdefLines(h: HitSpec | (ProjectileSpec & { at: number }), isProj: boolean, attr: string): string {
  const lv = LEVEL[(h as HitSpec).level ?? 'medium'];
  const guard = (h as HitSpec).guard ?? 'mid';
  const kb = h.knockback ?? [lv.kb, 0];
  const launch = (h as HitSpec).launch;
  const air = (h as HitSpec).airKnockback ?? [kb[0] * 0.8, launch ? -6 : -3.5];
  const stop = (h as HitSpec).hitstop ?? lv.stop;
  const stun = (h as HitSpec).hitstun ?? lv.stun;
  const gd = (h as HitSpec).guardDamage ?? Math.round(h.damage / 8);
  const lines = [
    `attr = ${attr}`,
    `damage = ${h.damage}, ${gd}`,
    `animtype = ${lv.anim}`,
    `guardflag = ${GUARDFLAG[guard] ?? 'MA'}`,
    `hitflag = MAF`,
    `pausetime = ${isProj ? 0 : stop}, ${stop}`,
    `guard.pausetime = ${isProj ? 0 : stop - 2}, ${stop - 2}`,
    `sparkno = ${(h as HitSpec).level === 'heavy' ? 2 : (h as HitSpec).level === 'light' ? 0 : 1}`,
    `guard.sparkno = 40`,
    `sparkxy = 0, 0`,
    `hitsound = 5, ${(h as HitSpec).level === 'heavy' ? 2 : (h as HitSpec).level === 'light' ? 0 : 1}`,
    `guardsound = 6, 0`,
    `ground.type = ${guard === 'low' ? 'Low' : 'High'}`,
    `ground.slidetime = ${Math.round(stun * 0.6)}`,
    `ground.hittime = ${stun}`,
    `ground.velocity = ${kb[0]}, ${launch ? (kb[1] || -7) : kb[1]}`,
    `air.velocity = ${air[0]}, ${air[1]}`,
    `air.hittime = ${stun + 6}`,
    `guard.velocity = ${kb[0] * 0.7}`,
    `fall = ${launch ? 1 : 0}`,
    `air.fall = ${launch || (h as HitSpec).level === 'heavy' ? 1 : 0}`,
    `fall.recover = 1`,
    `getpower = ${Math.round(h.damage * 1.2)}, ${Math.round(h.damage * 0.5)}`,
    `givepower = ${Math.round(h.damage * 0.6)}, ${Math.round(h.damage * 0.3)}`,
  ];
  if ((h as HitSpec).shake) lines.push(`envshake.time = ${(h as HitSpec).shake}`);
  return lines.join('\n');
}

export interface CompileOptions { factory: CanvasFactory; palNo?: number }

export function compileNative(nc: NativeChar, opt: CompileOptions): CharacterDef {
  const palette = nc.palettes && opt.palNo && opt.palNo > 1 ? nc.palettes[(opt.palNo - 2) % nc.palettes.length] : undefined;
  const sprites = new Map<string, Sprite>();
  const anims = new Map<number, Animation>();
  const animNo = new Map<string, number>();
  let nextAnim = 1000;
  for (const name of Object.keys(nc.anims)) {
    animNo.set(name, STANDARD_ANIMS[name] ?? nextAnim++);
  }
  const poseCache = new Map<string, { key: string; hurt: ABox[]; hit: ABox[] }>();
  let spriteIdx = 0;
  const renderFrame = (fr: NativeFrame): { key: string; hurt: ABox[]; hit: ABox[] } => {
    const cacheKey = JSON.stringify([fr.pose, fr.effect, fr.hit, fr.sprite]);
    const c = poseCache.get(cacheKey);
    if (c) return c;
    const key = `1,${spriteIdx++}`;
    let out;
    if (nc.art.type === 'rig') {
      if (fr.effect) out = renderEffect(fr.effect, opt.factory, palette);
      else {
        const pose = typeof fr.pose === 'string' ? nc.art.poses[fr.pose] : fr.pose;
        if (typeof fr.pose === 'string' && !pose) console.warn(`[${nc.id}] missing pose ${fr.pose}`);
        out = renderPose(nc.art.style, pose ?? {}, fr.hit, opt.factory, palette);
      }
    } else {
      out = { canvas: null, width: 0, height: 0, axisX: 0, axisY: 0, scale: 1, hurt: [] as ABox[], hit: [] as ABox[] };
    }
    sprites.set(key, { image: out.canvas, width: out.width, height: out.height, axisX: out.axisX, axisY: out.axisY, scale: out.scale });
    const r = { key, hurt: out.hurt, hit: out.hit };
    poseCache.set(cacheKey, r);
    return r;
  };
  for (const [name, a] of Object.entries(nc.anims)) {
    const no = animNo.get(name)!;
    const frames: AnimFrame[] = a.frames.map((fr) => {
      const r = renderFrame(fr);
      const [g, i] = r.key.split(',').map(Number);
      const hurt = fr.hurt === 'none' ? [] : (fr.hurt ?? r.hurt);
      const hit = fr.hit?.box ? [fr.hit.box] : r.hit;
      return {
        group: g, index: i, x: fr.offset?.[0] ?? 0, y: fr.offset?.[1] ?? 0, time: fr.ticks, flipH: false, flipV: false,
        blend: fr.effect ? 'add' : 'none', xscale: 1, yscale: 1, angle: 0, clsn1: hit, clsn2: fr.effect ? hit : hurt,
      } as AnimFrame;
    });
    const anim: Animation = { id: no, frames, loopStart: a.loop ?? 0, totalTime: 0 };
    computeTotal(anim);
    anims.set(no, anim);
  }
  // portrait = idle frame 0
  const idle = anims.get(0);

  // ---- constants
  const k = defaultConstants();
  const s = nc.stats ?? {};
  if (s.life) k.set('data.life', s.life);
  if (s.power) k.set('data.power', s.power);
  if (s.attack) k.set('data.attack', s.attack);
  if (s.defense) k.set('data.defence', s.defense);
  if (s.walk) { k.set('velocity.walk.fwd.x', s.walk[0]); k.set('velocity.walk.back.x', -Math.abs(s.walk[1])); }
  if (s.run) { k.set('velocity.run.fwd.x', s.run[0]); k.set('velocity.run.back.x', -Math.abs(s.run[1])); }
  if (s.jump) {
    k.set('velocity.jump.y', s.jump[0]); k.set('velocity.jump.back.x', s.jump[1]); k.set('velocity.jump.neu.x', s.jump[2]); k.set('velocity.jump.fwd.x', s.jump[3]);
    k.set('velocity.airjump.y', s.jump[0] * 0.9);
  }
  if (s.gravity) k.set('movement.yaccel', s.gravity);
  if (s.airJumps !== undefined) k.set('movement.airjump.num', s.airJumps);
  if (s.height) k.set('size.height', s.height);
  if (s.width) { k.set('size.ground.front', s.width[0]); k.set('size.ground.back', s.width[1]); }

  // ---- commands & states (generated CNS)
  const commands: CommandDef[] = [
    parseCommand('holdfwd', '/$F', 1, 1), parseCommand('holdback', '/$B', 1, 1), parseCommand('holdup', '/$U', 1, 1), parseCommand('holddown', '/$D', 1, 1),
    parseCommand('FF', 'F, F', 10, 1), parseCommand('BB', 'B, B', 10, 1),
  ];
  let cns = '';
  let minus1 = `[Statedef -1]\n\n[State -1, run]\ntype = ChangeState\nvalue = 100\ntriggerall = command = "FF"\ntrigger1 = statetype = S && ctrl\n\n[State -1, hop]\ntype = ChangeState\nvalue = 105\ntriggerall = command = "BB"\ntrigger1 = statetype = S && ctrl\n\n`;
  const order = [...nc.moves].sort((a, b) => rank(b) - rank(a));
  function rank(m: NativeMove) { return (m.kind === 'super' ? 300 : m.kind === 'special' ? 200 : 100) + m.input.split(',').length * 5 + (m.input.includes('+') ? 2 : 0); }
  let nextState = { normal: 200, crouch: 400, air: 600, special: 1000, super: 3000 };
  const usedCmd = new Set<string>();
  const movelist: { name: string; input: string; command: string }[] = [];
  for (const m of order) {
    const when = m.when ?? 'stand';
    const cat = m.kind === 'normal' ? (when === 'crouch' ? 'crouch' : when === 'air' ? 'air' : 'normal') : m.kind;
    const sid = (nextState as any)[cat]++;
    const cname = `m_${m.id}`;
    if (!usedCmd.has(cname)) {
      usedCmd.add(cname);
      const isSimpleBtn = /^[abcxyz](\+[abcxyz])*$/.test(m.input.trim());
      commands.push(parseCommand(cname, m.input, m.kind === 'super' ? 30 : m.kind === 'special' ? 20 : 15, isSimpleBtn ? 1 : 2));
    }
    movelist.push({ name: m.name, input: m.input, command: cname });
    if (m.simple) commands.push(parseCommand(`simple_${m.simple}_${m.id}`, m.simple === 'sp1' ? 'd' : m.simple === 'sp2' ? 'w' : m.simple === 'sp3' ? 'e' : 'd+w', 1, 2));
    const cost = m.cost ?? (m.kind === 'super' ? 1000 : 0);
    const stateCond = when === 'air' ? 'statetype = A' : when === 'crouch' ? 'statetype = C || (statetype = S && command = "holddown")' : 'statetype = S || statetype = C && command != "holddown"';
    const crouchOK = when === 'crouch' ? '\ntriggerall = command = "holddown"' : when === 'stand' ? '\ntriggerall = command != "holddown" || statetype = S' : '';
    const cancel = m.kind === 'normal'
      ? (when === 'air' ? '' : `\ntrigger2 = movecontact && stateno = [200, 299] && stateno != ${sid} && time > 2`)
      : m.kind === 'special'
        ? `\ntrigger2 = movecontact && stateno = [200, 699]\ntrigger3 = movecontact && stateno = [1000, 2999] && stateno != ${sid} && time > 6`
        : `\ntrigger2 = movecontact && stateno = [200, 2999]`;
    const simpleCmd = m.simple ? ` || command = "simple_${m.simple}_${m.id}"` : '';
    minus1 += `[State -1, ${m.name}]\ntype = ChangeState\nvalue = ${sid}\ntriggerall = command = "${cname}"${simpleCmd}\ntriggerall = ${stateCond}${crouchOK}\n${cost ? `triggerall = power >= ${cost}\n` : ''}trigger1 = ctrl${cancel}\n\n`;
    if (m.simple) minus1 = minus1.replace(`triggerall = command = "${cname}"${simpleCmd}`, `triggerall = command = "${cname}"${simpleCmd}`);

    const anim = animNo.get(m.anim);
    if (anim === undefined) console.warn(`[${nc.id}] move ${m.id} missing anim ${m.anim}`);
    const st = when === 'air' ? 'A' : when === 'crouch' ? 'C' : 'S';
    const ph = when === 'air' ? 'A' : when === 'crouch' ? 'C' : (m.moves?.some((x) => x.vel[1] !== null && x.vel[1] !== 0) ? 'N' : 'S');
    cns += `[Statedef ${sid}]\ntype = ${st}\nmovetype = A\nphysics = ${ph}\nanim = ${anim ?? 0}\nctrl = 0\n${when === 'air' ? '' : 'velset = 0, 0\n'}poweradd = ${cost ? -cost : (m.kind === 'special' ? 30 : 15)}\nsprpriority = 2\n\n`;
    if (m.kind === 'super') {
      cns += `[State ${sid}, super]\ntype = SuperPause\ntrigger1 = Time = 0\ntime = 40\nanim = 30\npos = 0, -60\nsound = -1\n\n`;
      cns += `[State ${sid}, ai]\ntype = AfterImage\ntrigger1 = Time = 0\ntime = 60\nlength = 10\n\n`;
    } else if (m.kind === 'special' && m.fx === 'aura') {
      cns += `[State ${sid}, ai]\ntype = AfterImage\ntrigger1 = Time = 0\ntime = 20\nlength = 6\n\n`;
    }
    if (m.invuln) {
      cns += `[State ${sid}, inv]\ntype = NotHitBy\ntrigger1 = AnimElemNo(0) = [${m.invuln[0]}, ${m.invuln[1]}]\nvalue = SCA\ntime = 1\n\n`;
    }
    if (m.teleport) {
      cns += `[State ${sid}, tp]\ntype = PosAdd\ntrigger1 = AnimElem = ${m.teleport}\nx = p2dist x + 40\n\n[State ${sid}, tpt]\ntype = Turn\ntrigger1 = AnimElem = ${m.teleport}\n\n`;
    }
    for (const mv of m.moves ?? []) {
      cns += `[State ${sid}, vel]\ntype = VelSet\ntrigger1 = AnimElem = ${mv.at}\n${mv.vel[0] !== null ? `x = ${mv.vel[0]}\n` : ''}${mv.vel[1] !== null ? `y = ${mv.vel[1]}\n` : ''}\n`;
    }
    if (ph === 'N' && when !== 'air') {
      cns += `[State ${sid}, grav]\ntype = VelAdd\ntrigger1 = Pos Y < 0 || Vel Y < 0\ny = const(movement.yaccel)\n\n[State ${sid}, land]\ntype = ChangeState\ntrigger1 = Pos Y >= 0 && Vel Y > 0\nvalue = 52\n\n[State ${sid}, fix]\ntype = PosSet\ntrigger1 = Pos Y > 0\ny = 0\n\n`;
    }
    const attr = `${st}, ${m.kind === 'normal' ? 'N' : m.kind === 'special' ? 'S' : 'H'}A`;
    for (const h of m.hits ?? []) {
      cns += `[State ${sid}, hit]\ntype = HitDef\ntrigger1 = AnimElem = ${h.at}\n${hitdefLines(h, false, attr)}\n\n`;
    }
    const projs = m.projectile ? (Array.isArray(m.projectile) ? m.projectile : [m.projectile]) : [];
    let pid = sid;
    for (const p of projs) {
      const pa = animNo.get(p.anim);
      const ph2 = p.hitAnim ? animNo.get(p.hitAnim) : undefined;
      cns += `[State ${sid}, proj]\ntype = Projectile\ntrigger1 = AnimElem = ${p.at}\nprojid = ${pid++}\nprojanim = ${pa ?? 0}\n${ph2 !== undefined ? `projhitanim = ${ph2}\nprojremanim = ${ph2}\n` : ''}offset = ${p.offset[0]}, ${p.offset[1]}\nvelocity = ${p.speed[0]}, ${p.speed[1]}\n${p.accel ? `accel = ${p.accel[0]}, ${p.accel[1]}\n` : ''}projhits = ${p.hits ?? 1}\nprojmisstime = 6\nprojpriority = ${p.priority ?? 1}\nprojremovetime = ${p.removeTime ?? -1}\n${hitdefLines({ ...p, level: p.level ?? 'medium', launch: p.launch, knockback: p.knockback } as any, true, `${st}, ${m.kind === 'super' ? 'H' : 'S'}P`)}\n\n`;
    }
    const endState = when === 'air' ? '' : `[State ${sid}, end]\ntype = ChangeState\ntrigger1 = AnimTime = 0\nvalue = ${when === 'crouch' ? 11 : 0}\nctrl = 1\n\n`;
    cns += endState;
  }
  const states = new Map();
  parseStates(minus1 + cns, states);
  const def: CharacterDef = {
    id: `umf:${nc.id}`, name: nc.name, displayName: nc.name, author: nc.author, source: 'native',
    localW: 320, localH: 240, constants: k, sprites, anims, commands, states, sounds: new Map(),
    portrait: idle ? sprites.get(`${idle.frames[0].group},${idle.frames[0].index}`) : undefined,
    movelist, aiCommands: movelist.map((m) => m.command), color: nc.color, series: nc.series, description: nc.description,
  };
  (def as any).generatedCns = minus1 + cns;
  return def;
}
