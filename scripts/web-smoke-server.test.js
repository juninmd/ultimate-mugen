const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const http = require('node:http');
const os = require('node:os');
const path = require('node:path');
const { staticServer } = require('./web-smoke-server');

const types = { '.html': 'text/html', '.txt': 'text/plain' };

async function withServer(run) {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'ume-web-smoke-'));
  const root = path.join(temporary, 'dist');
  const sibling = path.join(temporary, 'dist-private');
  fs.mkdirSync(root);
  fs.mkdirSync(sibling);
  fs.mkdirSync(path.join(root, 'assets'));
  fs.symlinkSync(sibling, path.join(root, 'assets', 'linked'), process.platform === 'win32' ? 'junction' : 'dir');
  fs.writeFileSync(path.join(root, 'index.html'), 'game');
  fs.writeFileSync(path.join(root, 'assets', 'index.html'), 'assets');
  fs.writeFileSync(path.join(sibling, 'secret.txt'), 'secret');
  const server = staticServer(root, types);
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  try {
    await run(server.address().port);
  } finally {
    await new Promise((resolve, reject) => server.close(error => error ? reject(error) : resolve()));
    fs.rmSync(temporary, { recursive: true, force: true });
  }
}

function request(port, pathname) {
  return new Promise((resolve, reject) => {
    http.get({ hostname: '127.0.0.1', port, path: pathname }, response => {
      const chunks = [];
      response.on('data', chunk => chunks.push(chunk));
      response.on('end', () => resolve({ status: response.statusCode, body: Buffer.concat(chunks).toString() }));
    }).on('error', reject);
  });
}

test('serves web build files from the local smoke server', async () => {
  await withServer(async port => {
    assert.deepEqual(await request(port, '/'), { status: 200, body: 'game' });
    assert.deepEqual(await request(port, '/assets/'), { status: 200, body: 'assets' });
  });
});

test('does not disclose sibling files whose directory shares the build prefix', async () => {
  await withServer(async port => {
    const response = await request(port, '/../dist-private/secret.txt');
    assert.equal(response.status, 404);
    assert.notEqual(response.body, 'secret');
  });
});

test('does not follow a link from the build directory to a sibling directory', async () => {
  await withServer(async port => {
    const response = await request(port, '/assets/linked/secret.txt');
    assert.equal(response.status, 404);
    assert.notEqual(response.body, 'secret');
  });
});

test('rejects encoded traversal and malformed escaped paths', async () => {
  await withServer(async port => {
    assert.equal((await request(port, '/%2e%2e/dist-private/secret.txt')).status, 404);
    assert.equal((await request(port, '/bad%2')).status, 404);
    assert.equal((await request(port, '/%00bad')).status, 404);
  });
});
