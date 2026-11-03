import { test } from 'node:test';
import assert from 'node:assert/strict';
import { bumpFor, compareVersions, latestTag, nextVersion, parseCommit, plan } from './release-plan.mjs';

const c = (message, sha = 'abcdef1234567') => ({ sha, message });

test('parseCommit reads type, scope, bang and footer breaking markers', () => {
  assert.deepEqual(parseCommit('feat(ui): add banner'), { type: 'feat', scope: 'ui', desc: 'add banner', breaking: false });
  assert.equal(parseCommit('fix!: drop save format').breaking, true);
  assert.equal(parseCommit('feat: x\n\nBREAKING CHANGE: y').breaking, true);
  assert.equal(parseCommit('Gamepad: pure button mapping'), null);
  assert.equal(parseCommit('Merge pull request #22 from a/b'), null);
  assert.equal(parseCommit('feat:no space'), null);
});

test('bumpFor maps feat/fix/breaking and ignores housekeeping', () => {
  const p = (...m) => m.map(parseCommit);
  assert.equal(bumpFor(p('fix: a', 'feat: b')), 'minor');
  assert.equal(bumpFor(p('fix: a', 'chore: b')), 'patch');
  assert.equal(bumpFor(p('feat: a', 'refactor!: b')), 'major');
  assert.equal(bumpFor(p('docs: a', 'ci: b', 'chore: c', 'test: d')), null);
});

test('nextVersion keeps pre-1.0 breaking changes on the minor line', () => {
  assert.equal(nextVersion('0.2.0', 'major'), '0.3.0');
  assert.equal(nextVersion('0.2.0', 'minor'), '0.3.0');
  assert.equal(nextVersion('0.2.5', 'patch'), '0.2.6');
  assert.equal(nextVersion('1.4.2', 'major'), '2.0.0');
  assert.equal(nextVersion('1.4.2', 'minor'), '1.5.0');
});

test('latestTag compares numerically and ignores non-release tags', () => {
  assert.equal(latestTag(['v0.9.0', 'v0.10.0', 'v0.2.0', 'v1.0.0-rc.1', 'nightly']), 'v0.10.0');
  assert.equal(latestTag(['nightly']), null);
  assert.ok(compareVersions('0.10.0', '0.9.9') > 0);
});

test('plan: no tag uses the Cargo baseline', () => {
  const r = plan({ commits: [c('feat: boss rush'), c('Merge pull request #1')], tags: [], baseline: '0.2.0' });
  assert.equal(r.release, true);
  assert.equal(r.tag, 'v0.3.0');
  assert.match(r.notes, /## Features\n- boss rush \(abcdef1\)/);
});

test('plan: latest tag wins over the baseline and patch fixes bump patch', () => {
  const r = plan({ commits: [c('fix(hud): clipped text')], tags: ['v0.3.0', 'v0.2.0'], baseline: '0.2.0', repo: 'o/r' });
  assert.equal(r.tag, 'v0.3.1');
  assert.match(r.notes, /\*\*hud:\*\* clipped text/);
  assert.match(r.notes, /compare\/v0\.3\.0\.\.\.v0\.3\.1/);
});

test('plan: housekeeping-only and empty ranges do not release', () => {
  assert.equal(plan({ commits: [c('docs: readme')], tags: ['v0.3.0'], baseline: '0.2.0' }).release, false);
  const none = plan({ commits: [], tags: ['v0.3.0'], baseline: '0.2.0' });
  assert.equal(none.release, false);
  assert.equal(none.reason, 'no commits since last tag');
});

test('plan: override forces a bump even for non-conventional history', () => {
  const r = plan({ commits: [c('Gamepad: mapping')], tags: [], baseline: '0.2.0', override: 'minor' });
  assert.equal(r.tag, 'v0.3.0');
  assert.match(r.notes, /Manual minor release/);
});

test('plan: refuses to reuse an existing tag', () => {
  // v0.3.0 exists on another branch (not merged into HEAD) and would be the next minor
  assert.throws(() => plan({ commits: [c('feat: a')], tags: [], allTags: ['v0.3.0'], baseline: '0.2.0' }), /already exists/);
});
