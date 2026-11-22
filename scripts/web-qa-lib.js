const fs = require('fs');
const path = require('path');

const manifestPath = path.join(__dirname, 'web-qa.json');

const loadManifest = file => JSON.parse(fs.readFileSync(file || manifestPath, 'utf8'));

const shardsOf = manifest => [...new Set(manifest.scenarios.map(s => s.shard))];

const select = (manifest, { shard, only } = {}) => {
  const names = only ? new Set(only.split(',')) : null;
  return manifest.scenarios.filter(s => (!shard || s.shard === shard) && (!names || names.has(s.name)));
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

module.exports = { loadManifest, shardsOf, select, expandArgs, tail, problemLines, summaryMarkdown };
