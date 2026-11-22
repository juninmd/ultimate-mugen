const fs = require('fs');
const path = require('path');
const { spawn } = require('child_process');
const { loadManifest, select, expandArgs, tail, problemLines, summaryMarkdown } = require('./web-qa-lib');

const opt = (name, fallback) => {
  const i = process.argv.indexOf(`--${name}`);
  return i > 0 ? process.argv[i + 1] : fallback;
};
const dist = path.resolve(opt('dist', 'web/dist'));
const outRoot = path.resolve(opt('out', 'web-qa-report'));
const jobs = Math.max(1, Number(opt('jobs', 1)));
const manifest = loadManifest();
const scenarios = select(manifest, { shard: opt('shard'), only: opt('only') });
const guard = path.join(__dirname, 'web-guard.js');

const run = scenario => new Promise(done => {
  const out = path.join(outRoot, scenario.name);
  fs.mkdirSync(out, { recursive: true });
  const log = fs.createWriteStream(path.join(out, 'output.log'));
  const env = {
    ...process.env,
    ...scenario.env,
    WEB_QA_OUT: out,
    WEB_QA_STRICT: scenario.expectFatal ? '0' : '1',
    WEB_SMOKE_SHOTS: path.join(out, 'shots'),
    NODE_OPTIONS: `${process.env.NODE_OPTIONS || ''} --require ${guard}`.trim(),
  };
  const args = [path.join(__dirname, scenario.script), ...expandArgs(scenario.args, { dist, out })];
  const started = Date.now();
  const child = spawn(process.execPath, args, { env, stdio: ['ignore', 'pipe', 'pipe'] });
  let timedOut = false;
  const timer = setTimeout(() => { timedOut = true; child.kill('SIGKILL'); }, scenario.timeoutMs || manifest.defaultTimeoutMs);
  child.stdout.pipe(log, { end: false });
  child.stderr.pipe(log, { end: false });
  child.on('close', (code, signal) => {
    clearTimeout(timer);
    log.end();
    const output = fs.readFileSync(path.join(out, 'output.log'), 'utf8');
    const consoleFile = path.join(out, 'console.log');
    const consoleText = fs.existsSync(consoleFile) ? fs.readFileSync(consoleFile, 'utf8') : '';
    const ok = code === 0 && !timedOut;
    const result = { name: scenario.name, ok, code, signal, timedOut, ms: Date.now() - started, problems: problemLines(consoleText), tail: tail(output, 25) };
    console.log(`${ok ? 'PASS' : 'FAIL'} ${scenario.name} (${(result.ms / 1000).toFixed(0)}s)`);
    done(result);
  });
});

(async () => {
  if (!scenarios.length) { console.error('no scenarios selected'); process.exit(2); }
  if (!fs.existsSync(path.join(dist, 'index.html'))) { console.error(`web build not found in ${dist}`); process.exit(2); }
  fs.mkdirSync(outRoot, { recursive: true });
  const results = [];
  const queue = [...scenarios];
  await Promise.all(Array.from({ length: Math.min(jobs, queue.length) }, async () => {
    while (queue.length) results.push(await run(queue.shift()));
  }));
  results.sort((a, b) => a.name.localeCompare(b.name));
  const markdown = summaryMarkdown(results, { shard: opt('shard') });
  fs.writeFileSync(path.join(outRoot, 'summary.md'), markdown);
  fs.writeFileSync(path.join(outRoot, 'summary.json'), JSON.stringify(results, null, 2));
  if (process.env.GITHUB_STEP_SUMMARY) fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, markdown);
  console.log(markdown);
  process.exit(results.every(r => r.ok) ? 0 : 1);
})();
