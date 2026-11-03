import { test } from 'node:test';
import assert from 'node:assert/strict';
import { PLATFORMS, assetName, buildManifest, parseAsset, verifyManifest } from './release-manifest.mjs';

const repo = 'o/r';
const assets = PLATFORMS.map((p) => ({ name: assetName('0.3.0', p), sha256: `hash-${p}`, size: 10 }));

test('asset names round-trip and reject strangers', () => {
  assert.deepEqual(parseAsset('ultimate-mugen-v0.3.0-windows-x86_64.zip'), { version: '0.3.0', platform: 'windows-x86_64' });
  assert.equal(parseAsset('ultimate-mugen-v0.3.0-windows-x86_64.tar.gz'), null);
  assert.equal(parseAsset('latest.json'), null);
  assert.equal(parseAsset('ultimate-mugen-v0.3.0-linux-x86_64.tar.gz.sha256'), null);
});

test('manifest has one https entry per platform with hashes', () => {
  const m = buildManifest({ version: '0.3.0', repo, assets, notes: '## Features\n', date: '2026-11-03T00:00:00Z' });
  assert.equal(m.version, '0.3.0');
  assert.equal(m.platforms['linux-x86_64'].url, 'https://github.com/o/r/releases/download/v0.3.0/ultimate-mugen-v0.3.0-linux-x86_64.tar.gz');
  assert.equal(m.platforms['macos-aarch64'].sha256, 'hash-macos-aarch64');
});

test('manifest refuses an incomplete platform set or a version mix', () => {
  assert.throws(() => buildManifest({ version: '0.3.0', repo, assets: assets.slice(1) }), /missing platform assets: linux-x86_64/);
  assert.throws(() => buildManifest({ version: '0.4.0', repo, assets }), /does not match version/);
});

test('verify passes on consistent assets and flags every kind of drift', () => {
  const m = buildManifest({ version: '0.3.0', repo, assets });
  const ok = { dir: '.', version: '0.3.0', repo, has: () => true, hashOf: (f) => `hash-${parseAsset(f).platform}` };
  assert.deepEqual(verifyManifest(m, ok), []);
  assert.match(verifyManifest(m, { ...ok, hashOf: () => 'tampered' })[0], /sha256 mismatch/);
  assert.match(verifyManifest(m, { ...ok, has: (f) => !f.includes('macos') })[0], /macos-aarch64: asset .* not found/);
  assert.match(verifyManifest(m, { ...ok, version: '0.3.1' })[0], /manifest version 0.3.0 != 0.3.1/);
  assert.match(verifyManifest(m, { ...ok, repo: 'evil/fork' }).join('\n'), /url .* !=/);
  const broken = { ...m, platforms: { ...m.platforms } };
  delete broken.platforms['windows-x86_64'];
  assert.match(verifyManifest(broken, ok)[0], /missing platform windows-x86_64/);
});
