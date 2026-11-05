import assert from 'node:assert/strict';
import test from 'node:test';
import { MAX_LINES, countLines, evaluate, formatReport } from './file-size.mjs';

const over = MAX_LINES + 10;

test('countLines ignores the final newline and handles CRLF', () => {
  assert.equal(countLines(''), 0);
  assert.equal(countLines('\n'), 0);
  assert.equal(countLines('a'), 1);
  assert.equal(countLines('a\n'), 1);
  assert.equal(countLines('a\r\nb\r\n'), 2);
});

test('a new file over the ceiling is reported', () => {
  const result = evaluate({ files: [{ path: 'src/big.rs', lines: over }] });
  assert.equal(result.ok, false);
  assert.deepEqual(result.oversized, [{ path: 'src/big.rs', lines: over, limit: MAX_LINES }]);
});

test('a file at the ceiling is accepted', () => {
  assert.equal(evaluate({ files: [{ path: 'src/ok.rs', lines: MAX_LINES }] }).ok, true);
});

test('a grandfathered file may stay over the ceiling but may not grow', () => {
  const files = [{ path: 'src/legacy.rs', lines: over }];
  assert.equal(evaluate({ files, baseline: { 'src/legacy.rs': over } }).ok, true);
  const grown = evaluate({ files: [{ path: 'src/legacy.rs', lines: over + 1 }], baseline: { 'src/legacy.rs': over } });
  assert.deepEqual(grown.grown, [{ path: 'src/legacy.rs', lines: over + 1, limit: over }]);
});

test('a file that fits under the ceiling must leave the baseline', () => {
  const result = evaluate({ files: [{ path: 'src/split.rs', lines: MAX_LINES }], baseline: { 'src/split.rs': 900 } });
  assert.equal(result.ok, false);
  assert.deepEqual(result.stale, [{ path: 'src/split.rs', lines: MAX_LINES, limit: 900 }]);
});

test('formatReport names every offending path', () => {
  const report = formatReport(evaluate({ files: [{ path: 'src/a.rs', lines: over }, { path: 'src/b.rs', lines: over + 5 }] }));
  assert.match(report, /src\/a\.rs/);
  assert.match(report, /src\/b\.rs/);
  assert.equal(formatReport(evaluate({ files: [{ path: 'src/a.rs', lines: 10 }] })), '');
});
