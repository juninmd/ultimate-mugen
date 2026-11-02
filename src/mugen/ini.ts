/** Generic MUGEN-style INI parser (keeps duplicate keys & order; used by DEF/CNS/CMD/AIR/stage files). */
export interface IniSection {
  name: string; // original case, trimmed
  lname: string; // lowercase
  entries: [string, string][]; // [lowercase key, raw value]
  lines: string[]; // raw non-key lines (AIR frames etc.)
}

export function stripComment(line: string): string {
  let inQ = false;
  for (let i = 0; i < line.length; i++) {
    const c = line[i];
    if (c === '"') inQ = !inQ;
    else if (c === ';' && !inQ) return line.slice(0, i);
  }
  return line;
}

export function parseIni(text: string): IniSection[] {
  const out: IniSection[] = [];
  let cur: IniSection | null = null;
  const lines = text.replace(/^﻿/, '').split(/\r?\n|\r/);
  for (const raw of lines) {
    const line = stripComment(raw).trim();
    if (!line) continue;
    if (line.startsWith('[')) {
      const end = line.indexOf(']');
      const name = line.slice(1, end < 0 ? undefined : end).trim();
      cur = { name, lname: name.toLowerCase(), entries: [], lines: [] };
      out.push(cur);
      continue;
    }
    if (!cur) {
      cur = { name: '', lname: '', entries: [], lines: [] };
      out.push(cur);
    }
    const eq = line.indexOf('=');
    // AIR-style "Clsn2[0] = ..." handled by consumer through lines too
    if (eq > 0 && !/^-?\d/.test(line)) {
      cur.entries.push([line.slice(0, eq).trim().toLowerCase(), line.slice(eq + 1).trim()]);
    }
    cur.lines.push(line);
  }
  return out;
}

export function sectionGet(s: IniSection | undefined, key: string): string | undefined {
  if (!s) return undefined;
  key = key.toLowerCase();
  for (const [k, v] of s.entries) if (k === key) return v;
  return undefined;
}

export function findSection(secs: IniSection[], name: string): IniSection | undefined {
  const l = name.toLowerCase();
  return secs.find((s) => s.lname === l);
}

export function unquote(v: string | undefined): string {
  if (v == null) return '';
  v = v.trim();
  if (v.startsWith('"')) {
    const e = v.indexOf('"', 1);
    return v.slice(1, e < 0 ? undefined : e);
  }
  return v;
}

export function nums(v: string | undefined, def: number[] = []): number[] {
  if (v == null || v.trim() === '') return def;
  const parts = v.split(',').map((p) => parseFloat(p.trim()));
  return parts.map((n, i) => (Number.isFinite(n) ? n : (def[i] ?? 0)));
}

/** Split on top-level commas (respecting parens/brackets/quotes). */
export function splitTop(v: string): string[] {
  const out: string[] = [];
  let depth = 0, inQ = false, start = 0;
  for (let i = 0; i < v.length; i++) {
    const c = v[i];
    if (c === '"') inQ = !inQ;
    else if (inQ) continue;
    else if (c === '(' || c === '[') depth++;
    else if (c === ')' || c === ']') depth--;
    else if (c === ',' && depth === 0) {
      out.push(v.slice(start, i).trim());
      start = i + 1;
    }
  }
  out.push(v.slice(start).trim());
  return out;
}
