import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const { parseArgs } = require('./record-gameplay.js');

test('parseArgs applies defaults correctly', () => {
  const opts = parseArgs(['node', 'record-gameplay.js']);
  assert.equal(opts.gif, 'gameplay.gif');
  assert.equal(opts.duration, 6);
  assert.equal(opts.shots, true);
  assert.match(opts.out, /recordings$/);
});

test('parseArgs parses custom CLI flags', () => {
  const opts = parseArgs([
    'node',
    'record-gameplay.js',
    '--out', 'docs/evidence/test',
    '--gif', 'custom.gif',
    '--duration', '10.5',
    '--no-shots',
  ]);
  assert.equal(opts.gif, 'custom.gif');
  assert.equal(opts.duration, 10.5);
  assert.equal(opts.shots, false);
  assert.match(opts.out, /test$/);
});
