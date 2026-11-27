import test from "node:test";
import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { layout, migrate, toUmf2 } from "./umf2-migrate.mjs";

const dir = "assets/chars";
const fighters = readdirSync(dir).filter((name) => name.endsWith(".json") && name !== "index.json");

test("every shipped fighter is canonical umf/2", () => {
  assert.ok(fighters.length >= 21);
  for (const name of fighters) {
    const text = readFileSync(join(dir, name), "utf8").replace(/\r\n/g, "\n");
    assert.equal(JSON.parse(text).format, "umf/2", name);
    assert.equal(migrate(text), text, `${name} is not in the canonical layout, run node scripts/umf2-migrate.mjs`);
  }
});

test("plain frames become pose*ticks and richer frames stay objects", () => {
  const out = toUmf2({
    format: "umf/1",
    anims: { idle: { frames: [{ pose: "a", ticks: 4 }, { pose: "b", ticks: 2, hit: { part: "fist" } }], loop: 0 } },
  });
  assert.equal(out.format, "umf/2");
  assert.deepEqual(out.anims.idle.frames, ["a*4", { pose: "b", ticks: 2, hit: { part: "fist" } }]);
  assert.equal(out.anims.idle.loop, 0);
});

test("layout keeps short values on one line and expands long ones", () => {
  assert.equal(layout({ a: [1, 2] }), '{"a":[1,2]}');
  const long = { list: Array.from({ length: 80 }, (_, i) => `item${i}`) };
  assert.ok(layout(long).split("\n").length > 3);
  assert.deepEqual(JSON.parse(layout(long)), long);
});
