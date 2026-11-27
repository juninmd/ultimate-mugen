import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

const require = createRequire(import.meta.url);
const lib = require('./web-qa-lib.js');
const dir = path.dirname(fileURLToPath(import.meta.url));
const manifest = lib.loadManifest();

test('every scenario has a unique name, an existing script and a shard', () => {
  const names = manifest.scenarios.map(s => s.name);
  assert.equal(new Set(names).size, names.length);
  for (const s of manifest.scenarios) {
    assert.ok(s.shard, `${s.name} has no shard`);
    assert.ok(fs.existsSync(path.join(dir, s.script)), `${s.name}: ${s.script} is missing`);
  }
});

test('no web smoke script can be forgotten: it runs in a scenario or is excluded with a reason', () => {
  const scripts = fs.readdirSync(dir).filter(f => /^web-.*-smoke\.js$/.test(f)).map(f => f.replace(/\.js$/, ''));
  const covered = new Set(manifest.scenarios.map(s => s.script.replace(/\.js$/, '')));
  for (const name of scripts) {
    assert.ok(covered.has(name) || manifest.excluded[name], `${name} is neither in web-qa.json scenarios nor excluded`);
  }
  for (const [name, reason] of Object.entries(manifest.excluded)) assert.ok(reason.length > 3, `${name} needs a reason`);
});

test('partition balances by weight and is deterministic', () => {
  const sc = ['a', 'b', 'c', 'd'].map(name => ({ name }));
  const bins = lib.partition(sc, 2, { a: 100, b: 60, c: 40, d: 10 });
  assert.deepEqual(bins.map(b => b.map(s => s.name)), [['a', 'd'], ['b', 'c']]);
  assert.throws(() => lib.select({ scenarios: sc }, { part: '3/2' }), /bad --part/);
});

test('the browser flows named in the request stay covered', () => {
  const scripts = manifest.scenarios.map(s => s.script);
  for (const needed of ['web-smoke.js', 'smoke-web.js', 'web-vpad-smoke.js', 'web-match-smoke.js', 'web-mugen-smoke.js', 'web-fatal-smoke.js', 'web-preflight-smoke.js']) {
    assert.ok(scripts.includes(needed), `${needed} dropped from web-qa.json`);
  }
  assert.ok(manifest.scenarios.some(s => s.env?.SMOKE_MOBILE === '1'), 'in-engine smoke no longer runs on a phone viewport');
});

test('select filters by shard and by name', () => {
  const ui = lib.select(manifest, { shard: 'ui' });
  assert.ok(ui.length > 1 && ui.every(s => s.shard === 'ui'));
  assert.deepEqual(lib.select(manifest, { only: 'fatal-trap' }).map(s => s.name), ['fatal-trap']);
});

test('expandArgs fills placeholders', () => {
  assert.deepEqual(lib.expandArgs(['{dist}', '{out}', 'x'], { dist: '/d', out: '/o' }), ['/d', '/o', 'x']);
});

test('summary lists failures with problems and the log tail', () => {
  const md = lib.summaryMarkdown([
    { name: 'ok-one', ok: true, ms: 4000, problems: [], tail: [] },
    { name: 'bad-one', ok: false, code: 1, ms: 9000, problems: ['[problem] panic: unreachable executed'], tail: ['WEB SMOKE FAILED'] },
  ], { shard: 'ui' });
  assert.match(md, /1 of 2 failed/);
  assert.match(md, /\| bad-one \| FAIL/);
  assert.match(md, /unreachable executed/);
  assert.match(md, /WEB SMOKE FAILED/);
});
