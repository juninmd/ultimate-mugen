import { unzipSync } from 'fflate';
import type { CharacterDef } from '../engine/chardef';
import type { StageInfo } from '../engine/match';
import { imageFromRGBA, makeCanvas } from '../engine/canvas';
import { findCharDefs, findStageDefs, loadMugenCharacter, loadMugenStage, normPath, text, type VFS } from '../mugen/loader';
import { parseIni, findSection, sectionGet, unquote } from '../mugen/ini';
import { compileNative } from '../native/compile';
import type { NativeChar } from '../native/types';
import { ROSTER } from '../chars';
import { STAGES } from '../stages';
import { savePack, loadPacks, type StoredPack } from './storage';

export interface CharEntry {
  id: string;
  name: string;
  series: string;
  author: string;
  color: string;
  kind: 'native' | 'mugen';
  description?: string;
  load(palNo?: number): CharacterDef;
  portrait(): CanvasImageSource | null;
}
export interface StageEntry { id: string; name: string; thumb: string; kind: 'native' | 'mugen'; build(): StageInfo }

const factory = (w: number, h: number) => makeCanvas(w, h);

function nativeEntry(nc: NativeChar): CharEntry {
  const cache = new Map<number, CharacterDef>();
  let portrait: CanvasImageSource | null | undefined;
  const e: CharEntry = {
    id: `umf:${nc.id}`, name: nc.name, series: nc.series, author: nc.author, color: nc.color, kind: 'native', description: nc.description,
    load(palNo = 1) {
      let d = cache.get(palNo);
      if (!d) { d = compileNative(nc, { factory, palNo }); cache.set(palNo, d); }
      return d;
    },
    portrait() {
      if (portrait !== undefined) return portrait;
      const d = e.load(1);
      portrait = renderPortrait(d);
      return portrait;
    },
  };
  return e;
}

/** Render the idle frame of a character into a square portrait canvas. */
export function renderPortrait(d: CharacterDef, size = 160): CanvasImageSource | null {
  const p9 = d.source === 'mugen' ? (d.sprites.get('9000,1') ?? d.sprites.get('9000,0')) : undefined;
  const c = makeCanvas(size, size);
  if (!c) return null;
  if (p9?.image) {
    const k = Math.min(size / p9.width, size / p9.height);
    c.ctx.imageSmoothingEnabled = false;
    c.ctx.drawImage(p9.image, (size - p9.width * k) / 2, (size - p9.height * k) / 2, p9.width * k, p9.height * k);
    return c.canvas;
  }
  const a = d.anims.get(0);
  const fr = a?.frames[0];
  const spr = fr ? d.sprites.get(`${fr.group},${fr.index}`) : undefined;
  if (!spr?.image) return c.canvas;
  const s = spr.scale ?? 1;
  // crop upper body: show from head to waist
  const hgt = spr.axisY * s; // height above feet
  const k = (size * 1.55) / Math.max(60, hgt);
  c.ctx.translate(size / 2, size * 1.45);
  c.ctx.scale(k * s, k * s);
  c.ctx.drawImage(spr.image, -spr.axisX, -spr.axisY, spr.width, spr.height);
  return c.canvas;
}

export class Registry {
  chars: CharEntry[] = [];
  stages: StageEntry[] = [];
  packs: StoredPack[] = [];
  constructor() {
    for (const nc of ROSTER) this.chars.push(nativeEntry(nc));
    for (const s of STAGES) this.stages.push({ id: `native:${s.id}`, name: s.name, thumb: s.thumb, kind: 'native', build: s.build });
  }

  async loadStored() {
    const packs = await loadPacks();
    for (const p of packs) this.registerVfs(p, false);
  }

  /** Import files (zip archives and/or loose files with relative paths). Returns number of chars/stages added. */
  async importFiles(files: { name: string; data: Uint8Array }[], packName = 'Importado'): Promise<{ chars: string[]; stages: string[]; errors: string[] }> {
    const vfs: Record<string, Uint8Array> = {};
    for (const f of files) {
      if (/\.zip$/i.test(f.name)) {
        try {
          const z = unzipSync(f.data);
          for (const [k, v] of Object.entries(z)) if (!k.endsWith('/')) vfs[normPath(k)] = v;
        } catch (e) { return { chars: [], stages: [], errors: [`${f.name}: zip inválido (${(e as Error).message})`] }; }
      } else vfs[normPath(f.name)] = f.data;
    }
    const pack: StoredPack = { id: `pack-${Date.now()}-${Math.floor(Math.random() * 1e6)}`, name: packName, added: Date.now(), files: vfs };
    const res = this.registerVfs(pack, true);
    if (res.chars.length || res.stages.length) await savePack(pack);
    return res;
  }

  registerVfs(pack: StoredPack, fresh: boolean): { chars: string[]; stages: string[]; errors: string[] } {
    const vfs: VFS = new Map(Object.entries(pack.files));
    const out = { chars: [] as string[], stages: [] as string[], errors: [] as string[] };
    for (const defPath of findCharDefs(vfs)) {
      try {
        const secs = parseIni(text(vfs.get(defPath)));
        const info = findSection(secs, 'info');
        const name = unquote(sectionGet(info, 'displayname') ?? sectionGet(info, 'name')) || defPath;
        const author = unquote(sectionGet(info, 'author'));
        const id = `mugen:${pack.id}:${defPath}`;
        if (this.chars.some((c) => c.id === id)) continue;
        const cache = new Map<number, CharacterDef>();
        let portrait: CanvasImageSource | null | undefined;
        const entry: CharEntry = {
          id, name, series: 'M.U.G.E.N', author: author || '?', color: '#6b7a99', kind: 'mugen',
          load(palNo = 1) {
            let d = cache.get(palNo);
            if (!d) { d = loadMugenCharacter(vfs, defPath, { palNo, makeImage: imageFromRGBA, id }); cache.set(palNo, d); }
            return d;
          },
          portrait() { if (portrait === undefined) portrait = renderPortrait(entry.load(1)); return portrait; },
        };
        if (fresh) entry.load(1); // validate now
        this.chars.push(entry);
        out.chars.push(name);
      } catch (e) {
        out.errors.push(`${defPath}: ${(e as Error).message}`);
      }
    }
    for (const defPath of findStageDefs(vfs)) {
      const secs = parseIni(text(vfs.get(defPath)));
      const name = unquote(sectionGet(findSection(secs, 'info'), 'displayname') ?? sectionGet(findSection(secs, 'info'), 'name')) || defPath;
      const id = `mugenstage:${pack.id}:${defPath}`;
      if (this.stages.some((s) => s.id === id)) continue;
      this.stages.push({ id, name, thumb: 'linear-gradient(#333,#111)', kind: 'mugen', build: () => loadMugenStage(vfs, defPath, imageFromRGBA) });
      out.stages.push(name);
    }
    if (out.chars.length || out.stages.length) this.packs.push(pack);
    return out;
  }
}
