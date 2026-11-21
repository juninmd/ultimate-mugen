#!/usr/bin/env node
import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');

export const EN_SECTIONS = [
  ['Players', ['Installation', 'Auto-Update', 'Configuration', 'MUGEN-Content', 'Marketplace', 'Story-Mode', 'Stage-Objects', 'Audio', 'Character-Select', 'Game-Modes', 'Netplay-and-Replays', 'Lore', 'Characters', 'Troubleshooting']],
  ['Creators', ['Compatibility-Decisions', 'Lua-Scripts', 'Route-Files', 'Make-a-Character', 'Remaking-a-MUGEN-Fighter', 'UMF-Character', 'Passives', 'Rush-Cancel', 'Combo-Tiers', 'Combo-Strings', 'Bleach-Inspired-Kits', 'Effects', 'CNS-Scripts', 'AssertSpecial', 'Life-Systems', 'ZSS', 'Ikemen-Extensions', 'Screenpacks', 'Screenpack-Study', 'Sprite-Atlas-Import', 'Sprite-Toolkit', 'Move-Generator', 'Make-a-Stage', 'Stages', 'Visual-Effects', 'AI-Tactics']],
  ['Contributors', ['Contributing', 'Architecture', 'Extension-Points', 'Engine', 'File-Layout', 'Dev-Tools', 'Performance', 'Web-Match-Smoke', 'Media-Capture', 'Stress-and-Resilience', 'Native-Animation-Audit']],
  ['Research', ['MUGEN-Code-Study', 'MUGEN-Dissection', 'Fighter-Study', 'Research-Fighting-Games', 'Research-Story-Mode', 'Rust-Engine-UI-References', 'Authoring-Tool-Survey', 'Ikemen-Web-Reverse-Engineering']],
];

export const COVERED = [['CONTRIBUTING.md', 'Contributing.md'], ['CONTRIBUTING.pt-BR.md', 'pt-BR/Contribuindo.md']];

export function pageTitle(text, fallback) {
  const match = /^#\s+(.+?)\s*$/m.exec(text);
  return match ? match[1].replace(/[\[\]]/g, '') : fallback;
}

export function summaryLine(indent, title, path) {
  return `${indent}- [${title}](${path})`;
}

export function buildSummary({ enPages, ptPages, adrPages, titles }) {
  const lines = ['# Summary', '', summaryLine('', titles['Home.md'] ?? 'Home', 'Home.md'), ''];
  const listed = new Set(['Home']);
  for (const [heading, names] of EN_SECTIONS) {
    const present = names.filter((name) => enPages.includes(name));
    present.forEach((name) => listed.add(name));
    if (present.length === 0) continue;
    lines.push(`# ${heading}`, '');
    for (const name of present) lines.push(summaryLine('', titles[`${name}.md`] ?? name, `${name}.md`));
    lines.push('');
  }
  const rest = enPages.filter((name) => !listed.has(name)).sort();
  if (rest.length > 0) {
    lines.push('# Reference', '');
    for (const name of rest) lines.push(summaryLine('', titles[`${name}.md`] ?? name, `${name}.md`));
    lines.push('');
  }
  if (adrPages.length > 0) {
    lines.push('# Architecture decisions', '');
    for (const name of adrPages) lines.push(summaryLine('', titles[`adr/${name}.md`] ?? name, `adr/${name}.md`));
    lines.push('');
  }
  if (ptPages.length > 0) {
    lines.push('# Português (BR)', '');
    const home = ptPages.includes('Home') ? ['Home'] : [];
    for (const name of [...home, ...ptPages.filter((n) => n !== 'Home').sort()]) {
      lines.push(summaryLine('', titles[`pt-BR/${name}.md`] ?? name, `pt-BR/${name}.md`));
    }
    lines.push('');
  }
  return lines.join('\n');
}

export function pageNames(dir) {
  if (!existsSync(dir)) return [];
  return readdirSync(dir).filter((f) => f.endsWith('.md') && f !== 'SUMMARY.md').map((f) => f.slice(0, -3));
}

export function stageBook(outDir, root = ROOT) {
  rmSync(outDir, { recursive: true, force: true });
  mkdirSync(outDir, { recursive: true });
  cpSync(join(root, 'docs/wiki'), outDir, { recursive: true });
  for (const [from, to] of COVERED) {
    if (existsSync(join(root, from))) cpSync(join(root, from), join(outDir, to));
  }
  const enPages = pageNames(outDir);
  const ptPages = pageNames(join(outDir, 'pt-BR'));
  const adrNames = pageNames(join(outDir, 'adr')).sort();
  const adrPages = [...adrNames.filter((n) => n === 'README'), ...adrNames.filter((n) => n !== 'README')];
  const titles = {};
  for (const name of enPages) titles[`${name}.md`] = pageTitle(readFileSync(join(outDir, `${name}.md`), 'utf8'), name);
  for (const name of ptPages) titles[`pt-BR/${name}.md`] = pageTitle(readFileSync(join(outDir, 'pt-BR', `${name}.md`), 'utf8'), name);
  for (const name of adrPages) titles[`adr/${name}.md`] = pageTitle(readFileSync(join(outDir, 'adr', `${name}.md`), 'utf8'), name);
  writeFileSync(join(outDir, 'SUMMARY.md'), buildSummary({ enPages, ptPages, adrPages, titles }));
  return { enPages, ptPages, adrPages };
}

export function missingFromSections(enPages) {
  const named = new Set(EN_SECTIONS.flatMap(([, names]) => names));
  return [...named].filter((name) => !enPages.includes(name));
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  const out = process.argv[2] ?? join(ROOT, 'target/wiki-book/src');
  const { enPages, ptPages } = stageBook(out);
  const missing = missingFromSections(enPages);
  if (missing.length > 0) {
    console.error(`wiki-book: sections name pages that do not exist: ${missing.join(', ')}`);
    process.exit(1);
  }
  console.log(`wiki-book: staged ${enPages.length} English and ${ptPages.length} Portuguese pages in ${out}`);
}
