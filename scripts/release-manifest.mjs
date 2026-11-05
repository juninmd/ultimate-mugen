#!/usr/bin/env node
// Builds and verifies the `latest.json` update manifest that the in-game updater reads.
//   build : release-manifest.mjs --dir dist --version 1.2.3 --repo o/r --notes notes.md   (writes dist/latest.json + dist/SHA256SUMS.txt)
//   verify: release-manifest.mjs --verify downloaded/latest.json --dir downloaded --version 1.2.3 --repo o/r
import { createHash } from 'node:crypto';
import { readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

export const PLATFORMS = ['linux-x86_64', 'windows-x86_64', 'macos-aarch64'];
// extension is tied to the platform: zip on Windows, tar.gz elsewhere
const ASSET = /^ultimate-mugen-v(\d+\.\d+\.\d+)-(windows-x86_64(?=\.zip$)|linux-x86_64(?=\.tar\.gz$)|macos-aarch64(?=\.tar\.gz$))\.(?:zip|tar\.gz)$/;
const BINARY = /^ultimate-mugen-v(\d+\.\d+\.\d+)-(windows-x86_64|linux-x86_64|macos-aarch64)(\.exe)?$/;

export const assetName = (version, platform) => `ultimate-mugen-v${version}-${platform}.${platform.startsWith('windows') ? 'zip' : 'tar.gz'}`;
export const binaryName = (version, platform) => `ultimate-mugen-v${version}-${platform}${platform.startsWith('windows') ? '.exe' : ''}`;
export const assetUrl = (repo, version, name) => `https://github.com/${repo}/releases/download/v${version}/${name}`;
export const sha256 = (path) => createHash('sha256').update(readFileSync(path)).digest('hex');

export function parseAsset(name) {
  const m = ASSET.exec(name);
  return m ? { version: m[1], platform: m[2] } : null;
}

export function parseBinary(name) {
  const m = BINARY.exec(name);
  if (!m) return null;
  if (m[3] !== '.exe' && m[2].startsWith('windows')) return null;
  if (m[3] === '.exe' && !m[2].startsWith('windows')) return null;
  return { version: m[1], platform: m[2] };
}

function sortAssets(assets) {
  const archives = new Map();
  const binaries = new Map();
  for (const a of assets) {
    const binary = parseBinary(a.name);
    if (binary) {
      if (binaries.has(binary.platform)) throw new Error(`duplicate binary asset: ${a.name}`);
      binaries.set(binary.platform, { ...binary, ...a });
      continue;
    }
    const archive = parseAsset(a.name);
    if (!archive) throw new Error(`unexpected asset name: ${a.name}`);
    if (archives.has(archive.platform)) throw new Error(`duplicate archive asset: ${a.name}`);
    archives.set(archive.platform, { ...archive, ...a });
  }
  return { archives, binaries };
}

export function buildManifest({ version, repo, assets, notes = '', date = new Date().toISOString() }) {
  const { archives, binaries } = sortAssets(assets);
  const platforms = {};
  for (const platform of PLATFORMS) {
    const archive = archives.get(platform);
    if (!archive) throw new Error(`missing platform assets: ${platform}`);
    if (archive.version !== version) throw new Error(`asset ${archive.name} does not match version ${version}`);
    const binary = binaries.get(platform);
    if (!binary) throw new Error(`missing binary asset for ${platform}`);
    if (binary.version !== version) throw new Error(`asset ${binary.name} does not match version ${version}`);
    platforms[platform] = {
      url: assetUrl(repo, version, archive.name), sha256: archive.sha256, size: archive.size,
      binary: { url: assetUrl(repo, version, binary.name), sha256: binary.sha256, size: binary.size },
    };
  }
  return { version, notes: notes.trim(), pub_date: date, platforms };
}

/** Returns a list of problems (empty = consistent). `hashOf(file)` is injectable for tests. */
export function verifyManifest(manifest, { dir, version, repo, hashOf = (f) => sha256(join(dir, f)), has = (f) => { try { return statSync(join(dir, f)).isFile(); } catch { return false; } } }) {
  const problems = [];
  if (manifest.version !== version) problems.push(`manifest version ${manifest.version} != ${version}`);
  for (const platform of PLATFORMS) {
    const entry = manifest.platforms?.[platform];
    if (!entry) { problems.push(`missing platform ${platform}`); continue; }
    const archive = assetName(version, platform);
    if (entry.url !== assetUrl(repo, version, archive)) problems.push(`${platform}: url ${entry.url} != ${assetUrl(repo, version, archive)}`);
    if (!has(archive)) { problems.push(`${platform}: asset ${archive} not found`); continue; }
    if (hashOf(archive) !== entry.sha256) problems.push(`${platform}: sha256 mismatch for ${archive}`);
    const binary = binaryName(version, platform);
    if (!entry.binary) { problems.push(`${platform}: missing binary entry`); continue; }
    if (entry.binary.url !== assetUrl(repo, version, binary)) problems.push(`${platform}: binary url ${entry.binary.url} != ${assetUrl(repo, version, binary)}`);
    if (!has(binary)) { problems.push(`${platform}: binary ${binary} not found`); continue; }
    if (hashOf(binary) !== entry.binary.sha256) problems.push(`${platform}: sha256 mismatch for binary ${binary}`);
  }
  return problems;
}

function main(argv) {
  const opt = (n) => { const i = argv.indexOf(n); return i >= 0 ? argv[i + 1] : null; };
  const need = (n) => opt(n) ?? (() => { throw new Error(`missing ${n}`); })();
  const [dir, version, repo] = [need('--dir'), need('--version'), need('--repo')];
  const verify = opt('--verify');
  if (verify) {
    const problems = verifyManifest(JSON.parse(readFileSync(verify, 'utf8')), { dir, version, repo });
    if (problems.length) { console.error(problems.map((p) => `- ${p}`).join('\n')); process.exit(1); }
    console.log(`manifest ok: v${version} (${PLATFORMS.length} platforms, with binaries)`);
    return;
  }
  const assets = readdirSync(dir).filter((f) => parseAsset(f) || parseBinary(f)).sort()
    .map((name) => ({ name, sha256: sha256(join(dir, name)), size: statSync(join(dir, name)).size }));
  const notesFile = opt('--notes');
  const manifest = buildManifest({ version, repo, assets, notes: notesFile ? readFileSync(notesFile, 'utf8') : '' });
  writeFileSync(join(dir, 'latest.json'), JSON.stringify(manifest, null, 2) + '\n');
  writeFileSync(join(dir, 'SHA256SUMS.txt'), assets.map((a) => `${a.sha256}  ${a.name}`).join('\n') + '\n');
  console.log(`wrote latest.json for v${version}: ${assets.map((a) => a.name).join(', ')}`);
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  try { main(process.argv.slice(2)); } catch (e) { console.error(`release-manifest: ${e.message}`); process.exit(1); }
}
