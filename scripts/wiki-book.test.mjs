import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import { EN_SECTIONS, buildSummary, missingFromSections, pageTitle, stageBook } from './wiki-book.mjs';

test('pageTitle reads the first heading and falls back to the file name', () => {
  assert.equal(pageTitle('intro\n# Hello [x]\nbody', 'f'), 'Hello x');
  assert.equal(pageTitle('no heading', 'fallback'), 'fallback');
});

test('buildSummary groups known pages and collects the rest under Reference', () => {
  const summary = buildSummary({ enPages: ['Home', 'Installation', 'Zebra'], ptPages: ['Home', 'Instalacao'], adrPages: ['0001-x'], titles: {} });
  assert.match(summary, /# Players\n\n- \[Installation\]\(Installation\.md\)/);
  assert.match(summary, /# Reference\n\n- \[Zebra\]\(Zebra\.md\)/);
  assert.match(summary, /# Architecture decisions\n\n- \[0001-x\]\(adr\/0001-x\.md\)/);
  assert.match(summary, /# Português \(BR\)\n\n- \[Home\]\(pt-BR\/Home\.md\)\n- \[Instalacao\]\(pt-BR\/Instalacao\.md\)/);
});

test('every section entry exists in the real wiki and every page is reachable', () => {
  const out = mkdtempSync(join(tmpdir(), 'wiki-book-'));
  const { enPages, ptPages } = stageBook(out);
  assert.deepEqual(missingFromSections(enPages), []);
  const summary = readFileSync(join(out, 'SUMMARY.md'), 'utf8');
  for (const name of enPages) if (name !== 'SUMMARY') assert.ok(summary.includes(`(${name}.md)`), `${name} missing from SUMMARY`);
  for (const name of ptPages) assert.ok(summary.includes(`(pt-BR/${name}.md)`), `pt-BR/${name} missing from SUMMARY`);
  assert.ok(EN_SECTIONS.length > 0);
});
