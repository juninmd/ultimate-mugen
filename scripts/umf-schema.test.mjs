import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { repoFiles, validate } from './umf-schema.mjs';

const kaito = () => JSON.parse(readFileSync(new URL('../assets/chars/kaito.json', import.meta.url), 'utf8'));
const paths = (data) => validate(data).map((e) => e.path);

test('every fighter, kit and example in the repository satisfies the schema', () => {
  const files = repoFiles();
  assert.ok(files.length >= 25, `only ${files.length} files found`);
  for (const file of files) assert.deepEqual(validate(JSON.parse(readFileSync(file, 'utf8'))), [], file);
});

test('a misspelled key is rejected instead of silently ignored by the loader', () => {
  const fighter = kaito();
  fighter.stats.lyfe = 900;
  fighter.moves[0].hitz = [];
  assert.deepEqual(paths(fighter).sort(), ['/moves/0/hitz', '/stats/lyfe']);
});

test('enums and ranges are enforced', () => {
  const fighter = kaito();
  fighter.moves[1].kind = 'spcial';
  fighter.color = 'red';
  fighter.stats.life = 0;
  assert.deepEqual(paths(fighter).sort(), ['/color', '/moves/1/kind', '/stats/life']);
});

test('a standalone fighter needs its art, animations and moves; an extending one does not', () => {
  assert.ok(validate({ format: 'umf/2', id: 'mini' }).length > 0);
  assert.deepEqual(validate({ format: 'umf/2', id: 'mini', extends: 'kaito', moves: [{ id: 'gale', name: 'Gale' }] }), []);
});

test('a move without its identity is rejected', () => {
  const fighter = kaito();
  delete fighter.moves[0].id;
  assert.deepEqual(paths(fighter), ['/moves/0']);
});

test('single and list forms of projectile, shake and teleport both validate', () => {
  const fighter = kaito();
  const move = { id: 'x', name: 'X', input: 'a', kind: 'normal', anim: 'jab' };
  const projectile = { at: 3, anim: 'orb', speed: [4, 0], offset: [10, -20], damage: 30 };
  fighter.moves = [{ ...move, projectile }, { ...move, id: 'y', projectile: [projectile, projectile], teleport: [2, { at: 4, mode: 'behind' }], shake: { time: 5 } }];
  assert.deepEqual(validate(fighter), []);
});
