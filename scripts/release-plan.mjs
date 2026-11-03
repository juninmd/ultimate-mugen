#!/usr/bin/env node
// Release planner: conventional commits since the last vX.Y.Z tag -> next version + release notes.
// The git tag is the source of truth for the version; Cargo.toml is only the pre-first-tag baseline.
import { execFileSync } from 'node:child_process';
import { appendFileSync, readFileSync, writeFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

const HEADER = /^(?<type>[a-z]+)(?:\((?<scope>[^)]+)\))?(?<bang>!)?:\s+(?<desc>\S.*)$/;
const TAG = /^v(\d+)\.(\d+)\.(\d+)$/;
const SECTIONS = [['breaking', 'Breaking changes'], ['feat', 'Features'], ['fix', 'Fixes'], ['perf', 'Performance']];

export function parseCommit(message) {
  const [subject, ...rest] = message.trim().split('\n');
  const m = HEADER.exec(subject.trim());
  if (!m) return null;
  const breaking = Boolean(m.groups.bang) || /^BREAKING[ -]CHANGE:/m.test(rest.join('\n'));
  return { type: m.groups.type, scope: m.groups.scope ?? null, desc: m.groups.desc.trim(), breaking };
}

export function parseVersion(s) {
  const m = /^(\d+)\.(\d+)\.(\d+)$/.exec(s);
  if (!m) throw new Error(`invalid version: ${s}`);
  return m.slice(1).map(Number);
}

export function compareVersions(a, b) {
  const [x, y] = [parseVersion(a), parseVersion(b)];
  for (let i = 0; i < 3; i++) if (x[i] !== y[i]) return x[i] - y[i];
  return 0;
}

export function bumpFor(parsed) {
  if (parsed.some((c) => c.breaking)) return 'major';
  if (parsed.some((c) => c.type === 'feat')) return 'minor';
  if (parsed.some((c) => c.type === 'fix' || c.type === 'perf')) return 'patch';
  return null; // chore/docs/ci/test/refactor/style/build alone never cut a release
}

export function nextVersion(current, bump) {
  let [maj, min, pat] = parseVersion(current);
  // pre-1.0 a breaking change bumps minor; 1.0.0 is a deliberate `--bump major`
  if (bump === 'major' && maj > 0) return `${maj + 1}.0.0`;
  if (bump === 'major' || bump === 'minor') return `${maj}.${min + 1}.0`;
  if (bump === 'patch') return `${maj}.${min}.${pat + 1}`;
  throw new Error(`invalid bump: ${bump}`);
}

export function latestTag(tags) {
  const valid = tags.filter((t) => TAG.test(t)).map((t) => t.slice(1));
  if (!valid.length) return null;
  return `v${valid.sort(compareVersions).at(-1)}`;
}

export function renderNotes(entries, { tag, prev, repo }) {
  const out = [];
  for (const [key, title] of SECTIONS) {
    const items = entries.filter((e) => (key === 'breaking' ? e.breaking : !e.breaking && e.type === key));
    if (!items.length) continue;
    out.push(`## ${title}`, ...items.map((e) => `- ${e.scope ? `**${e.scope}:** ` : ''}${e.desc} (${e.sha.slice(0, 7)})`), '');
  }
  if (repo && prev) out.push(`**Full changelog:** https://github.com/${repo}/compare/${prev}...${tag}`);
  return out.join('\n').trim() + '\n';
}

/** Pure core: commits are `{ sha, message }` newest first. */
export function plan({ commits, tags, allTags = tags, baseline, override = null, repo = null }) {
  const prev = latestTag(tags);
  const current = prev ? prev.slice(1) : baseline;
  const entries = commits.flatMap((c) => { const p = parseCommit(c.message); return p ? [{ ...p, sha: c.sha }] : []; });
  const bump = override ?? bumpFor(entries);
  if (!bump) return { release: false, reason: commits.length ? 'no releasable commits (feat/fix/perf/breaking)' : 'no commits since last tag', current, prev };
  const next = nextVersion(current, bump);
  const tag = `v${next}`;
  if (allTags.includes(tag)) throw new Error(`tag ${tag} already exists; refusing to overwrite`);
  const notes = renderNotes(entries.length ? entries : [{ type: 'fix', desc: `Manual ${bump} release`, sha: commits[0]?.sha ?? '0000000', breaking: false, scope: null }], { tag, prev, repo });
  return { release: true, bump, current, prev, next, tag, notes };
}

const git = (...args) => execFileSync('git', args, { encoding: 'utf8', maxBuffer: 64 << 20 }).trim();

function readBaseline() {
  const m = /^\[package\][^[]*?^version\s*=\s*"(\d+\.\d+\.\d+)"/ms.exec(readFileSync('Cargo.toml', 'utf8'));
  if (!m) throw new Error('Cargo.toml [package] version not found');
  return m[1];
}

function main(argv) {
  const opt = (name) => { const i = argv.indexOf(name); return i >= 0 ? argv[i + 1] : null; };
  const override = opt('--bump');
  if (override && !['major', 'minor', 'patch'].includes(override)) throw new Error(`invalid --bump: ${override}`);
  const tags = git('tag', '--list', 'v[0-9]*', '--merged', 'HEAD').split('\n').filter(Boolean);
  const prev = latestTag(tags);
  const range = prev ? [`${prev}..HEAD`] : ['HEAD'];
  const raw = git('log', ...range, '--no-merges', '--format=%H%x1f%B%x1e');
  const commits = raw.split('\x1e').map((r) => r.trim()).filter(Boolean).map((r) => { const [sha, message] = r.split('\x1f'); return { sha, message }; });
  // tags on other branches are not "merged" into HEAD but must still never be reused
  const allTags = git('tag', '--list', 'v[0-9]*').split('\n').filter(Boolean);
  const result = plan({ commits, tags, allTags, baseline: readBaseline(), override, repo: process.env.GITHUB_REPOSITORY ?? null });
  const { notes, ...summary } = result;
  const notesFile = opt('--notes-file');
  if (notesFile && notes) writeFileSync(notesFile, notes);
  if (process.env.GITHUB_OUTPUT && argv.includes('--github-output')) {
    const lines = Object.entries({ release: summary.release, version: summary.next ?? '', tag: summary.tag ?? '', bump: summary.bump ?? '', prev: summary.prev ?? '' }).map(([k, v]) => `${k}=${v}`);
    appendFileSync(process.env.GITHUB_OUTPUT, lines.join('\n') + '\n');
  }
  console.log(JSON.stringify(summary));
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  try { main(process.argv.slice(2)); } catch (e) { console.error(`release-plan: ${e.message}`); process.exit(1); }
}
