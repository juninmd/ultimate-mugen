import { test } from 'node:test';
import assert from 'node:assert/strict';
import { isConventional, lint } from './commit-lint.mjs';

test('only known conventional types pass', () => {
  assert.equal(isConventional('feat(ui): banner'), true);
  assert.equal(isConventional('fix!: x'), true);
  assert.equal(isConventional('wip: x'), false);
  assert.equal(isConventional('Gamepad: mapping'), false);
});

test('squash flow: a conventional title is enough', () => {
  assert.equal(lint({ title: 'feat: x', messages: ['whatever', 'else'] }).ok, true);
});

test('merge flow: all commits conventional passes, one stray fails and is named', () => {
  assert.equal(lint({ title: 'Release stuff', messages: ['feat: a', 'fix: b'] }).ok, true);
  const r = lint({ title: 'Release stuff', messages: ['feat: a', 'oops typo'] });
  assert.equal(r.ok, false);
  assert.match(r.reason, /oops typo/);
});

test('no title and no commits fails closed', () => {
  assert.equal(lint({ title: '', messages: [] }).ok, false);
});
