import { readFileSync, writeFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

const WIDTH = 200;

function compactFrame(frame) {
  const keys = Object.keys(frame);
  const plain = keys.length === 2 && typeof frame.pose === "string" && Number.isInteger(frame.ticks);
  return plain && !frame.pose.includes("\n") ? `${frame.pose}*${frame.ticks}` : frame;
}

export function toUmf2(fighter) {
  const out = { ...fighter, format: "umf/2" };
  if (out.anims) {
    out.anims = Object.fromEntries(
      Object.entries(out.anims).map(([name, anim]) => [
        name,
        Array.isArray(anim.frames) ? { ...anim, frames: anim.frames.map(compactFrame) } : anim,
      ]),
    );
  }
  return out;
}

export function layout(value, indent = 0) {
  const flat = JSON.stringify(value);
  if (flat === undefined || indent * 2 + flat.length <= WIDTH || typeof value !== "object" || value === null) return flat;
  const pad = " ".repeat((indent + 1) * 2);
  const end = " ".repeat(indent * 2);
  if (Array.isArray(value)) {
    return `[\n${value.map((item) => pad + layout(item, indent + 1)).join(",\n")}\n${end}]`;
  }
  const rows = Object.entries(value).map(([key, item]) => `${pad}${JSON.stringify(key)}: ${layout(item, indent + 1)}`);
  return `{\n${rows.join(",\n")}\n${end}}`;
}

export function migrate(text) {
  return `${layout(toUmf2(JSON.parse(text)))}\n`;
}

if (process.argv[1] && process.argv[1].endsWith("umf2-migrate.mjs")) {
  const dir = process.argv[2] ?? "assets/chars";
  for (const name of readdirSync(dir)) {
    if (!name.endsWith(".json") || name === "index.json") continue;
    const path = join(dir, name);
    writeFileSync(path, migrate(readFileSync(path, "utf8")));
  }
}
