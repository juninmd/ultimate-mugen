import { parseIni, sectionGet, unquote, type IniSection } from './ini';
import { makeParam, nextUid, type StateController, type StateDef } from '../engine/chardef';
import { parseExpr, type Node } from '../engine/expr';
import { parseCommand, type CommandDef } from '../engine/commands';

function safeExpr(src: string): Node {
  try { return parseExpr(src); } catch { return { k: 'num', v: 0, float: false }; }
}

/**
 * Parse [Statedef]/[State] sections. States are merged into `into`; when `override` is false
 * existing statedefs are kept (used for common states loaded after the character's own).
 */
export function parseStates(text: string, into: Map<number, StateDef>, override = true) {
  const secs = parseIni(text);
  let cur: StateDef | null = null;
  let skip = false;
  for (const s of secs) {
    const m = /^statedef\s+(-?\d+)/i.exec(s.name);
    if (m) {
      const id = parseInt(m[1]);
      if (into.has(id) && !override) { skip = true; cur = null; continue; }
      skip = false;
      // negative states are merged (concatenated) when defined across files
      if (id < 0 && into.has(id)) {
        cur = into.get(id)!;
      } else {
        cur = { id, params: new Map(), controllers: [] };
        into.set(id, cur);
      }
      for (const [k, v] of s.entries) cur.params.set(k, makeParam(v));
      continue;
    }
    if (/^state\s/i.test(s.name) || /^state\s*-?\d/i.test(s.name)) {
      if (skip || !cur) continue;
      const sc = buildController(s);
      if (sc) cur.controllers.push(sc);
      continue;
    }
    if (!/^(state|statedef)/i.test(s.name)) { cur = null; skip = false; }
  }
}

export function buildController(s: IniSection): StateController | null {
  const type = (sectionGet(s, 'type') ?? '').trim().toLowerCase();
  if (!type) return null;
  const sc: StateController = {
    type, label: s.name, triggerAll: [], triggers: [], params: new Map(),
    persistent: 1, ignoreHitPause: false, uid: nextUid(),
  };
  for (const [k, v] of s.entries) {
    if (k === 'type') continue;
    if (k === 'triggerall') { sc.triggerAll.push(safeExpr(v)); continue; }
    const tm = /^trigger(\d+)$/.exec(k);
    if (tm) {
      const n = parseInt(tm[1]) - 1;
      if (n < 0 || n > 64) continue;
      while (sc.triggers.length <= n) sc.triggers.push([]);
      sc.triggers[n].push(safeExpr(v));
      continue;
    }
    if (k === 'persistent') { sc.persistent = parseInt(v) || 0; continue; }
    if (k === 'ignorehitpause') { sc.ignoreHitPause = parseInt(v) !== 0; continue; }
    sc.params.set(k, makeParam(v));
  }
  // MUGEN stops at the first missing trigger number
  const firstEmpty = sc.triggers.findIndex((t) => t.length === 0);
  if (firstEmpty >= 0) sc.triggers.length = firstEmpty;
  return sc;
}

/** Read [Data] [Size] [Velocity] [Movement] constants into map */
export function parseConstants(text: string, into: Map<string, number>) {
  for (const s of parseIni(text)) {
    const sec = s.lname;
    if (!['data', 'size', 'velocity', 'movement'].includes(sec)) continue;
    for (const [k, v] of s.entries) {
      const parts = v.split(',').map((x) => parseFloat(x));
      const key = `${sec}.${k}`;
      if (parts.length === 1) into.set(key, Number.isFinite(parts[0]) ? parts[0] : 0);
      else {
        // vectors: .x .y (.z)
        const comps = ['x', 'y', 'z'];
        parts.forEach((p, i) => into.set(`${key}.${comps[i] ?? i}`, Number.isFinite(p) ? p : 0));
        // MUGEN also allows const(velocity.walk.fwd.x) where key was "walk.fwd"
      }
      // scalar keys like "size.xscale" are already fine
    }
  }
}

export function parseCmd(text: string): { commands: CommandDef[]; states: Map<number, StateDef> } {
  const secs = parseIni(text);
  let defTime = 15, defBuf = 1;
  const d = secs.find((s) => s.lname === 'defaults');
  if (d) {
    defTime = parseInt(sectionGet(d, 'command.time') ?? '15') || 15;
    defBuf = parseInt(sectionGet(d, 'command.buffer.time') ?? '1') || 1;
  }
  const commands: CommandDef[] = [];
  for (const s of secs) {
    if (s.lname !== 'command') continue;
    const name = unquote(sectionGet(s, 'name'));
    const cmd = sectionGet(s, 'command');
    if (!name || cmd == null) continue;
    const time = parseInt(sectionGet(s, 'time') ?? '') || defTime;
    const buf = parseInt(sectionGet(s, 'buffer.time') ?? '') || defBuf;
    commands.push(parseCommand(name, cmd, time, buf));
  }
  const states = new Map<number, StateDef>();
  parseStates(text, states);
  return { commands, states };
}
