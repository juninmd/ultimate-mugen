const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');

function loader(fetch, expire = false) {
  let cleared = false;
  let timeout;
  const scope = {
    window: {}, document: { baseURI: 'https://example.test/game/index.html' },
    URL, Uint8Array, AbortController, fetch,
    setTimeout: fn => { timeout = fn; return 1; },
    clearTimeout: () => { cleared = true; },
  };
  vm.runInNewContext(fs.readFileSync(path.join(__dirname, '../web/assets.js'), 'utf8'), scope);
  const result = scope.window.ume_fetch_asset('assets/stages/dojo.png');
  if (expire) timeout();
  return { result, cleared: () => cleared };
}

test('fetches the selected asset relative to the deployed page', async () => {
  const run = loader(async (url) => {
    assert.equal(url.href, 'https://example.test/game/assets/stages/dojo.png');
    return { ok: true, arrayBuffer: async () => new Uint8Array([1, 2, 3]).buffer };
  });
  assert.deepEqual(Array.from(await run.result), [1, 2, 3]);
  assert.equal(run.cleared(), true);
});

test('rejects HTTP errors and clears the timeout', async () => {
  const run = loader(async () => ({ ok: false, status: 404 }));
  await assert.rejects(run.result, /HTTP 404/);
  assert.equal(run.cleared(), true);
});

test('aborts stalled downloads and clears the timeout', async () => {
  const run = loader((url, { signal }) => new Promise((resolve, reject) => {
    signal.addEventListener('abort', () => reject(new Error('aborted')));
  }), true);
  await assert.rejects(run.result, /aborted/);
  assert.equal(run.cleared(), true);
});
