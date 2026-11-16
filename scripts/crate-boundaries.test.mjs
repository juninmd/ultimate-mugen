import assert from 'node:assert/strict';
import test from 'node:test';
import { dependencyNames, violations } from './crate-boundaries.mjs';

test('dependencyNames reads runtime and build dependency tables only', () => {
  const toml = '[package]\nname = "x"\n[dependencies]\nume-formats = { path = "../formats" }\nserde = "1"\n[target.\'cfg(unix)\'.dependencies]\nlibc = "0"\n[dev-dependencies]\ntempfile = "3"\n[dependencies.winit]\nversion = "0.30"\n';
  assert.deepEqual([...dependencyNames(toml)].sort(), ['libc', 'serde', 'ume-formats', 'winit']);
});

test('a forbidden dependency is reported with its crate', () => {
  const toml = '[dependencies]\nwinit = "0.30"\nume-formats = { path = "../formats" }\n';
  assert.deepEqual(violations('render', toml), [{ crate: 'render', dependency: 'winit' }]);
  assert.deepEqual(violations('shell', toml), []);
});

test('the real workspace respects every boundary', async () => {
  const { readFileSync } = await import('node:fs');
  const { FORBIDDEN } = await import('./crate-boundaries.mjs');
  for (const crate of Object.keys(FORBIDDEN)) {
    assert.deepEqual(violations(crate, readFileSync(new URL(`../crates/${crate}/Cargo.toml`, import.meta.url), 'utf8')), []);
  }
});
