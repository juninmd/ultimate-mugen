const fs = require('fs');
const path = require('path');

const manifestPath = path.join(__dirname, 'web-qa.json');

const loadManifest = file => JSON.parse(fs.readFileSync(file || manifestPath, 'utf8'));

const shardsOf = manifest => [...new Set(manifest.scenarios.map(s => s.shard))];

const weightsPath = path.join(__dirname, 'web-qa-weights.json');
const DEFAULT_WEIGHT = 60;

const loadWeights = file => {
  try { return JSON.parse(fs.readFileSync(file || weightsPath, 'utf8')); } catch { return {}; }
};

const partition = (scenarios, count, weights = {}) => {
  const bins = Array.from({ length: count }, () => ({ load: 0, items: [] }));
  const w = s => weights[s.name] ?? DEFAULT_WEIGHT;
  const ordered = [...scenarios].sort((a, b) => w(b) - w(a) || a.name.localeCompare(b.name));
  for (const s of ordered) {
    const bin = bins.reduce((best, b) => (b.load < best.load ? b : best));
    bin.items.push(s);
    bin.load += w(s);
  }
  return bins.map(b => b.items);
};

const select = (manifest, { shard, only, part, weights } = {}) => {
  const names = only ? new Set(only.split(',')) : null;
  let list = manifest.scenarios.filter(s => (!shard || s.shard === shard) && (!names || names.has(s.name)));
  if (part) {
    const [index, count] = part.split('/').map(Number);
    if (!(index >= 1 && count >= index)) throw new Error(`bad --part ${part}, expected i/N`);
    list = partition(list, count, weights ?? loadWeights())[index - 1];
  }
  return list;
};

const expandArgs = (args, vars) => (args || []).map(a => a.replace(/\{(\w+)\}/g, (_, k) => vars[k] ?? ''));

const tail = (text, lines) => text.split('\n').filter(Boolean).slice(-lines);

const problemLines = text => text.split('\n').filter(l => /\[problem\]/.test(l)).slice(0, 12);

const summaryMarkdown = (results, meta = {}) => {
  const failed = results.filter(r => !r.ok);
  const head = [`# Web QA${meta.shard ? ` (${meta.shard})` : ''}: ${failed.length ? `${failed.length} of ${results.length} failed` : `${results.length} passed`}`, ''];
  const rows = ['| Scenario | Result | Time | Evidence |', '|---|---|---|---|'];
  for (const r of results) rows.push(`| ${r.name} | ${r.ok ? 'pass' : 'FAIL'} | ${(r.ms / 1000).toFixed(0)}s | ${r.ok ? '' : `${r.name}/output.log, console.log, *.png`} |`);
  const details = [];
  for (const r of failed) {
    details.push('', `## ${r.name}`, `exit ${r.code}${r.signal ? ` (${r.signal})` : ''}${r.timedOut ? ', timed out' : ''}`);
    if (r.problems.length) details.push('', 'Problems:', '```', ...r.problems, '```');
    details.push('', 'Last output:', '```', ...r.tail, '```');
  }
  return [...head, ...rows, ...details, ''].join('\n');
};

module.exports = { loadManifest, loadWeights, partition, shardsOf, select, expandArgs, tail, problemLines, summaryMarkdown };
