import { test } from 'node:test';
import assert from 'node:assert/strict';
import { PLATFORMS, assetName, binaryName, buildManifest, parseAsset, parseBinary, verifyManifest } from './release-manifest.mjs';

const repo = 'o/r';
const archives = PLATFORMS.map((p) => ({ name: assetName('0.3.0', p), sha256: `hash-${p}`, size: 10 }));
const binaries = PLATFORMS.map((p) => ({ name: binaryName('0.3.0', p), sha256: `bin-${p}`, size: 20 }));
const assets = [...archives, ...binaries];
const hashOf = (f) => {
  const binary = parseBinary(f);
  return binary ? `bin-${binary.platform}` : `hash-${parseAsset(f).platform}`;
};

test('asset names round-trip and reject strangers', () => {
  assert.deepEqual(parseAsset('ultimate-mugen-v0.3.0-windows-x86_64.zip'), { version: '0.3.0', platform: 'windows-x86_64' });
  assert.equal(parseAsset('ultimate-mugen-v0.3.0-windows-x86_64.tar.gz'), null);
  assert.equal(parseAsset('latest.json'), null);
  assert.equal(parseAsset('ultimate-mugen-v0.3.0-linux-x86_64.tar.gz.sha256'), null);
});

test('the raw updater binary is named per platform', () => {
  assert.equal(binaryName('0.3.0', 'windows-x86_64'), 'ultimate-mugen-v0.3.0-windows-x86_64.exe');
  assert.equal(binaryName('0.3.0', 'linux-x86_64'), 'ultimate-mugen-v0.3.0-linux-x86_64');
  assert.deepEqual(parseBinary('ultimate-mugen-v0.3.0-windows-x86_64.exe'), { version: '0.3.0', platform: 'windows-x86_64' });
  assert.deepEqual(parseBinary('ultimate-mugen-v0.3.0-macos-aarch64'), { version: '0.3.0', platform: 'macos-aarch64' });
  assert.equal(parseBinary('ultimate-mugen-v0.3.0-windows-x86_64'), null);
  assert.equal(parseBinary('ultimate-mugen-v0.3.0-linux-x86_64.exe'), null);
  assert.equal(parseBinary('ultimate-mugen-v0.3.0-linux-x86_64.tar.gz'), null);
});

test('manifest has one https entry and one binary per platform with hashes', () => {
  const m = buildManifest({ version: '0.3.0', repo, assets, notes: '## Features\n', date: '2026-11-03T00:00:00Z' });
  assert.equal(m.version, '0.3.0');
  assert.equal(m.platforms['linux-x86_64'].url, 'https://github.com/o/r/releases/download/v0.3.0/ultimate-mugen-v0.3.0-linux-x86_64.tar.gz');
  assert.equal(m.platforms['macos-aarch64'].sha256, 'hash-macos-aarch64');
  assert.deepEqual(m.platforms['windows-x86_64'].binary, {
    url: 'https://github.com/o/r/releases/download/v0.3.0/ultimate-mugen-v0.3.0-windows-x86_64.exe',
    sha256: 'bin-windows-x86_64',
    size: 20,
  });
});

test('manifest refuses an incomplete platform set or a version mix', () => {
  assert.throws(() => buildManifest({ version: '0.3.0', repo, assets: assets.slice(1) }), /missing platform assets: linux-x86_64/);
  assert.throws(() => buildManifest({ version: '0.4.0', repo, assets }), /does not match version/);
  assert.throws(() => buildManifest({ version: '0.3.0', repo, assets: archives }), /missing binary asset for linux-x86_64/);
  assert.throws(() => buildManifest({ version: '0.3.0', repo, assets: [...assets, binaries[0]] }), /duplicate binary asset/);
});

test('verify passes on consistent assets and flags every kind of drift', () => {
  const m = buildManifest({ version: '0.3.0', repo, assets });
  const ok = { dir: '.', version: '0.3.0', repo, has: () => true, hashOf };
  assert.deepEqual(verifyManifest(m, ok), []);
  assert.match(verifyManifest(m, { ...ok, hashOf: () => 'tampered' })[0], /sha256 mismatch/);
  assert.match(verifyManifest(m, { ...ok, has: (f) => !f.includes('macos') })[0], /macos-aarch64: asset .* not found/);
  assert.match(verifyManifest(m, { ...ok, version: '0.3.1' })[0], /manifest version 0.3.0 != 0.3.1/);
  assert.match(verifyManifest(m, { ...ok, repo: 'evil/fork' }).join('\n'), /url .* !=/);
  assert.match(verifyManifest(m, { ...ok, has: (f) => !f.endsWith('.exe') && !f.includes('macos') }).join('\n'), /binary .* not found|asset .* not found/);
  const broken = { ...m, platforms: { ...m.platforms } };
  delete broken.platforms['windows-x86_64'];
  assert.match(verifyManifest(broken, ok)[0], /missing platform windows-x86_64/);
  const noBinary = { ...m, platforms: { ...m.platforms, 'windows-x86_64': { ...m.platforms['windows-x86_64'] } } };
  delete noBinary.platforms['windows-x86_64'].binary;
  assert.match(verifyManifest(noBinary, ok)[0], /windows-x86_64: missing binary entry/);
  const tampered = { ...m, platforms: { ...m.platforms, 'windows-x86_64': { ...m.platforms['windows-x86_64'], binary: { ...m.platforms['windows-x86_64'].binary, sha256: 'deadbeef' } } } };
  assert.match(verifyManifest(tampered, ok).join('\n'), /sha256 mismatch for binary/);
});
