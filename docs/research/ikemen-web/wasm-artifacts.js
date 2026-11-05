// Reads a WebAssembly engine artifact (gzip or raw) and prints what the module needs and exports,
// plus the Go build info and the web-backend files named by the Go symbol table.
// Usage: node wasm-artifacts.js <runtime-engine.bin | engine.wasm>
const fs = require('fs');
const zlib = require('zlib');

const path = process.argv[2];
if (!path) {
  console.error('usage: node wasm-artifacts.js <runtime-engine.bin|engine.wasm>');
  process.exit(2);
}
let buf = fs.readFileSync(path);
if (buf[0] === 0x1f && buf[1] === 0x8b) buf = zlib.gunzipSync(buf);
if (buf.subarray(0, 4).toString('latin1') !== '\0asm') {
  console.error('not a WebAssembly module');
  process.exit(1);
}
console.log(`module bytes: ${buf.length}`);

const mod = new WebAssembly.Module(buf);
const byModule = new Map();
for (const i of WebAssembly.Module.imports(mod)) {
  if (!byModule.has(i.module)) byModule.set(i.module, []);
  byModule.get(i.module).push(`${i.name}:${i.kind}`);
}
for (const [name, list] of byModule) {
  console.log(`\n[imports "${name}"] ${list.length}`);
  for (const item of list) console.log('  ' + item);
}
const exports_ = WebAssembly.Module.exports(mod);
console.log('\n[exports] ' + exports_.map(e => `${e.name}:${e.kind}`).join(' '));

const text = buf.toString('latin1');
const build = text.match(/ikemen-GO[\x20-\x7e]{0,80}/);
if (build) console.log('\n[build info] ' + build[0].trim());

const files = new Set();
for (const m of text.matchAll(/github\.com\/ikemen-engine\/Ikemen-GO\/src\/[A-Za-z0-9_]+\.go/g)) files.add(m[0].split('/src/')[1]);
console.log('\n[go source files named by the symbol table] ' + [...files].sort().join(' '));
