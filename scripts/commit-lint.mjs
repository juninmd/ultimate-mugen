#!/usr/bin/env node
// PR gate for semantic releases: the PR title (squash flow) or every commit (merge flow) must be a conventional commit.
//   commit-lint.mjs --title "feat: x" --range origin/master..HEAD
import { execFileSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';
import { parseCommit } from './release-plan.mjs';

export const TYPES = ['feat', 'fix', 'perf', 'refactor', 'docs', 'test', 'build', 'ci', 'chore', 'style', 'revert'];

export const isConventional = (msg) => { const p = parseCommit(msg); return p !== null && TYPES.includes(p.type); };

/** Returns `{ ok, reason }`. Squash merges use the title; merge commits keep the PR's own commits. */
export function lint({ title, messages }) {
  if (title && isConventional(title)) return { ok: true, reason: 'PR title is a conventional commit (squash flow)' };
  const bad = messages.filter((m) => !isConventional(m));
  if (messages.length && !bad.length) return { ok: true, reason: 'every commit is conventional (merge flow)' };
  const sample = (bad.length ? bad : [title ?? '']).slice(0, 5).map((m) => `  - ${m.split('\n')[0]}`).join('\n');
  return { ok: false, reason: `use "<type>(scope)?(!)?: <description>" with type in ${TYPES.join('|')}, in the PR title or in every commit. Offending:\n${sample}` };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  const argv = process.argv.slice(2);
  const opt = (n) => { const i = argv.indexOf(n); return i >= 0 ? argv[i + 1] : null; };
  const range = opt('--range');
  const raw = range ? execFileSync('git', ['log', range, '--no-merges', '--format=%B%x1e'], { encoding: 'utf8' }) : '';
  const messages = raw.split('\x1e').map((m) => m.trim()).filter(Boolean);
  const { ok, reason } = lint({ title: opt('--title'), messages });
  console[ok ? 'log' : 'error'](ok ? `ok: ${reason}` : `commit-lint: ${reason}`);
  process.exit(ok ? 0 : 1);
}
