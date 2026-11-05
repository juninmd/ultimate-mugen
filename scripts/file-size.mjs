#!/usr/bin/env node
// Hard ceiling of 250 lines per source file, so several features can be worked on in parallel without sharing the same file.
// Files that are already over the ceiling are grandfathered in file-size-baseline.json: they may not grow, and they must
// leave the baseline in the same pull request that splits them under the limit.
//   node scripts/file-size.mjs            check the tree (used by CI)
//   node scripts/file-size.mjs --update   rewrite the baseline from the current tree
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

export const MAX_LINES = 250;
export const SOURCE_EXTENSIONS = ['.rs', '.js', '.mjs', '.cjs', '.ts', '.tsx', '.css', '.html', '.wgsl'];
export const BASELINE_PATH = join(dirname(fileURLToPath(import.meta.url)), 'file-size-baseline.json');

/** Physical lines, with a single trailing newline ignored so an editor's final break does not add a line. */
export function countLines(text) {
  const normalized = text.replace(/\r\n/g, '\n');
  const body = normalized.endsWith('\n') ? normalized.slice(0, -1) : normalized;
  return body === '' ? 0 : body.split('\n').length;
}

/** Pure check: `files` is `[{ path, lines }]`, `baseline` maps a path to the line budget it was granted. */
export function evaluate({ files, baseline = {}, maxLines = MAX_LINES }) {
  const oversized = [];
  const grown = [];
  const stale = [];
  for (const { path, lines } of files) {
    const budget = baseline[path];
    if (budget === undefined) {
      if (lines > maxLines) oversized.push({ path, lines, limit: maxLines });
      continue;
    }
    if (lines > budget) grown.push({ path, lines, limit: budget });
    else if (lines <= maxLines) stale.push({ path, lines, limit: budget });
  }
  const byPath = (a, b) => a.path.localeCompare(b.path);
  return {
    oversized: oversized.sort(byPath),
    grown: grown.sort(byPath),
    stale: stale.sort(byPath),
    ok: oversized.length === 0 && grown.length === 0 && stale.length === 0,
  };
}

const bullets = (items) => items.map((item) => `  - ${item.path}: ${item.lines} lines (limit ${item.limit})`).join('\n');

export function formatReport(result, maxLines = MAX_LINES) {
  const sections = [];
  if (result.oversized.length) {
    sections.push(`files over ${maxLines} lines, split each responsibility into its own file:\n${bullets(result.oversized)}`);
  }
  if (result.grown.length) {
    sections.push(`grandfathered files that grew, extract the new code instead:\n${bullets(result.grown)}`);
  }
  if (result.stale.length) {
    sections.push(`files now under ${maxLines} lines that still have a baseline entry, run --update:\n${bullets(result.stale)}`);
  }
  return sections.join('\n\n');
}

const gitList = (root, args) =>
  execFileSync('git', args, { cwd: root, encoding: 'utf8' })
    .split('\n')
    .map((line) => line.trim())
    .filter(Boolean);

export function listSourcePaths(root) {
  const isSource = (path) => SOURCE_EXTENSIONS.some((extension) => path.endsWith(extension));
  const paths = [...gitList(root, ['ls-files']), ...gitList(root, ['ls-files', '--others', '--exclude-standard'])];
  return [...new Set(paths.filter(isSource).map((path) => path.replace(/\\/g, '/')))].sort();
}

export function readSources(root) {
  return listSourcePaths(root).map((path) => ({ path, lines: countLines(readFileSync(join(root, path), 'utf8')) }));
}

export function readBaseline(path = BASELINE_PATH) {
  return existsSync(path) ? JSON.parse(readFileSync(path, 'utf8')) : {};
}

export function writeBaseline(files, path = BASELINE_PATH) {
  const baseline = Object.fromEntries(
    files
      .filter((file) => file.lines > MAX_LINES)
      .sort((a, b) => a.path.localeCompare(b.path))
      .map((file) => [file.path, file.lines]),
  );
  writeFileSync(path, `${JSON.stringify(baseline, null, 2)}\n`);
  return baseline;
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  const root = process.cwd();
  const files = readSources(root);
  if (process.argv.includes('--update')) {
    const baseline = writeBaseline(files);
    console.log(`file-size: baseline updated with ${Object.keys(baseline).length} files over ${MAX_LINES} lines`);
    process.exit(0);
  }
  const baseline = readBaseline();
  const result = evaluate({ files, baseline });
  const report = formatReport(result);
  const summary = `ok: ${files.length} source files, ${Object.keys(baseline).length} grandfathered over ${MAX_LINES} lines (none grew)`;
  console[result.ok ? 'log' : 'error'](report || summary);
  process.exit(result.ok ? 0 : 1);
}
