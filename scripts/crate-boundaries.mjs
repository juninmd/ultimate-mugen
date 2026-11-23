#!/usr/bin/env node
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');

export const FORBIDDEN = {
  formats: ['ume-engine', 'ume-render', 'ume-shell', 'ume-netplay', 'ume-market', 'wgpu', 'winit', 'ggrs'],
  engine: ['ume-native', 'ume-render', 'ume-shell', 'ume-netplay', 'ume-market', 'wgpu', 'winit', 'ggrs'],
  native: ['ume-render', 'ume-shell', 'ume-netplay', 'ume-market', 'wgpu', 'winit', 'ggrs'],
  render: ['ume-engine', 'ume-shell', 'ume-netplay', 'ume-market', 'winit', 'ggrs'],
  shell: ['ume-engine', 'ume-netplay', 'ume-market', 'ggrs'],
  netplay: ['ume-render', 'ume-shell', 'ume-market', 'wgpu', 'winit'],
  model3d: ['ume-engine', 'ume-native', 'ume-render', 'ume-shell', 'ume-netplay', 'ume-market', 'ume-formats', 'wgpu', 'winit', 'ggrs'],
  market: ['ume-engine', 'ume-render', 'ume-shell', 'ume-netplay', 'wgpu', 'winit', 'ggrs'],
};

export function dependencyNames(toml) {
  const names = new Set();
  let inDependencies = false;
  for (const raw of toml.split(/\r?\n/)) {
    const line = raw.trim();
    const section = /^\[(.+)\]$/.exec(line);
    if (section) {
      inDependencies = /(^|\.)(dependencies|build-dependencies)$/.test(section[1]);
      const named = /(?:^|\.)(?:dependencies|build-dependencies)\.([A-Za-z0-9_-]+)$/.exec(section[1]);
      if (named) names.add(named[1]);
      continue;
    }
    if (!inDependencies) continue;
    const entry = /^([A-Za-z0-9_-]+)\s*=/.exec(line);
    if (entry) names.add(entry[1]);
  }
  return names;
}

export function violations(crate, toml, forbidden = FORBIDDEN) {
  const names = dependencyNames(toml);
  return (forbidden[crate] ?? []).filter((name) => names.has(name)).map((name) => ({ crate, dependency: name }));
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  const found = Object.keys(FORBIDDEN).flatMap((crate) => violations(crate, readFileSync(join(ROOT, 'crates', crate, 'Cargo.toml'), 'utf8')));
  for (const { crate, dependency } of found) console.error(`crate boundary: crates/${crate} must not depend on ${dependency}`);
  if (found.length > 0) process.exit(1);
  console.log(`crate boundaries ok (${Object.keys(FORBIDDEN).length} crates checked)`);
}
