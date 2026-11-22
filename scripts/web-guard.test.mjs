import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const dir = path.dirname(new URL(import.meta.url).pathname);
const hasBrowser = (() => { try { require('playwright'); return true; } catch { return false; } })();

const probe = body => `
const { chromium } = require('playwright');
(async () => {
  const b = await chromium.launch();
  const p = await b.newPage();
  await p.setContent('<body style="background:#246"><h1>guard</h1></body>');
  await p.evaluate(() => { ${body} });
  await p.waitForTimeout(300);
  await b.close();
})();`;

const runProbe = (body, strict) => {
  const work = fs.mkdtempSync(path.join(os.tmpdir(), 'web-guard-'));
  const script = path.join(work, 'probe.js');
  fs.writeFileSync(script, probe(body));
  const out = path.join(work, 'out');
  const env = { ...process.env, WEB_QA_OUT: out, WEB_QA_STRICT: strict ? '1' : '0', NODE_PATH: [process.env.NODE_PATH, path.join(dir, '..', 'node_modules')].filter(Boolean).join(path.delimiter) };
  const result = spawnSync(process.execPath, ['--require', path.join(dir, 'web-guard.js'), script], { env, encoding: 'utf8', timeout: 60000 });
  return { result, out };
};

test('a wasm panic fails the run, keeps a screenshot and writes the console log', { skip: !hasBrowser && 'playwright not installed' }, () => {
  const { result, out } = runProbe("console.error('panicked at src/x.rs:1: unreachable executed')", true);
  assert.equal(result.status, 1, result.stderr);
  assert.match(fs.readFileSync(path.join(out, 'console.log'), 'utf8'), /\[problem\] panic: panicked at/);
  assert.ok(fs.existsSync(path.join(out, 'problem-1.png')));
  assert.ok(fs.existsSync(path.join(out, 'final-1.png')));
  assert.equal(JSON.parse(fs.readFileSync(path.join(out, 'guard.json'), 'utf8')).problems.length, 1);
});

test('a gpu error and an uncaught page error are fatal', { skip: !hasBrowser && 'playwright not installed' }, () => {
  const gpu = runProbe("console.error('gpu error: Validation Error')", true);
  assert.equal(gpu.result.status, 1);
  const thrown = runProbe("setTimeout(() => { throw new Error('boom'); }, 0)", true);
  assert.equal(thrown.result.status, 1);
});

test('a clean run passes and non-strict mode only records', { skip: !hasBrowser && 'playwright not installed' }, () => {
  const clean = runProbe("console.log('fine')", true);
  assert.equal(clean.result.status, 0, clean.result.stderr);
  assert.equal(JSON.parse(fs.readFileSync(path.join(clean.out, 'guard.json'), 'utf8')).problems.length, 0);
  const lax = runProbe("console.error('panicked at x')", false);
  assert.equal(lax.result.status, 0);
  assert.match(fs.readFileSync(path.join(lax.out, 'console.log'), 'utf8'), /\[problem\]/);
});
