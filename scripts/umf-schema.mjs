#!/usr/bin/env node
// Validates UMF fighters and kits against docs/schema/umf-2.schema.json with a small in-repo JSON Schema subset
// (no dependency): $ref, type, enum, const, required, properties, additionalProperties, items, min/max, pattern,
// oneOf, anyOf, if/then/else.
//   node scripts/umf-schema.mjs                 check the repository fighters, kits and examples
//   node scripts/umf-schema.mjs a.json b.json   check specific files (e.g. a pack from mugen-assets)
import { readFileSync, readdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');
export const SCHEMA_PATH = join(ROOT, 'docs/schema/umf-2.schema.json');

const kind = (v) => (v === null ? 'null' : Array.isArray(v) ? 'array' : Number.isInteger(v) ? 'integer' : typeof v);
const isType = (v, t) => (t === 'number' ? typeof v === 'number' : kind(v) === t);
const escape = (key) => String(key).replace(/~/g, '~0').replace(/\//g, '~1');

function resolve(root, ref) {
  if (!ref.startsWith('#/')) throw new Error(`unsupported $ref ${ref}`);
  return ref.slice(2).split('/').reduce((node, key) => node[key.replace(/~1/g, '/').replace(/~0/g, '~')], root);
}

function check(root, schema, value, path, errors) {
  if (schema.$ref) return check(root, resolve(root, schema.$ref), value, path, errors);
  const fail = (message) => errors.push({ path: path || '/', message });
  if (schema.type) {
    const types = [].concat(schema.type);
    if (!types.some((t) => isType(value, t))) return fail(`expected ${types.join(' or ')}, got ${kind(value)}`);
  }
  if (schema.enum && !schema.enum.some((e) => JSON.stringify(e) === JSON.stringify(value))) fail(`${JSON.stringify(value)} is not one of ${JSON.stringify(schema.enum)}`);
  if ('const' in schema && JSON.stringify(schema.const) !== JSON.stringify(value)) fail(`expected ${JSON.stringify(schema.const)}`);
  if (typeof value === 'number') {
    if (schema.minimum !== undefined && value < schema.minimum) fail(`${value} is below ${schema.minimum}`);
    if (schema.maximum !== undefined && value > schema.maximum) fail(`${value} is above ${schema.maximum}`);
    if (schema.exclusiveMinimum !== undefined && value <= schema.exclusiveMinimum) fail(`${value} must be above ${schema.exclusiveMinimum}`);
  }
  if (typeof value === 'string') {
    if (schema.minLength !== undefined && value.length < schema.minLength) fail('string is too short');
    if (schema.pattern && !new RegExp(schema.pattern).test(value)) fail(`${JSON.stringify(value)} does not match ${schema.pattern}`);
  }
  if (Array.isArray(value)) {
    if (schema.minItems !== undefined && value.length < schema.minItems) fail(`needs at least ${schema.minItems} items`);
    if (schema.maxItems !== undefined && value.length > schema.maxItems) fail(`allows at most ${schema.maxItems} items`);
    if (schema.items) value.forEach((item, i) => check(root, schema.items, item, `${path}/${i}`, errors));
  }
  if (kind(value) === 'object') {
    for (const key of schema.required ?? []) if (!(key in value)) fail(`missing required key "${key}"`);
    const known = schema.properties ?? {};
    for (const [key, item] of Object.entries(value)) {
      const at = `${path}/${escape(key)}`;
      if (key in known) check(root, known[key], item, at, errors);
      else if (schema.additionalProperties === false) errors.push({ path: at, message: 'unknown key' });
      else if (schema.additionalProperties) check(root, schema.additionalProperties, item, at, errors);
    }
  }
  const branches = (list) => list.map((branch) => { const sub = []; check(root, branch, value, path, sub); return sub; });
  if (schema.oneOf || schema.anyOf) {
    const results = branches(schema.oneOf ?? schema.anyOf);
    const passing = results.filter((r) => r.length === 0).length;
    if (passing === 0) errors.push(...results.reduce((best, r) => (r.length < best.length ? r : best)));
    else if (schema.oneOf && passing > 1) fail('matches more than one alternative');
  }
  if (schema.if) {
    const probe = [];
    check(root, schema.if, value, path, probe);
    const next = probe.length === 0 ? schema.then : schema.else;
    if (next) check(root, next, value, path, errors);
  }
}

export function validate(value, schema = JSON.parse(readFileSync(SCHEMA_PATH, 'utf8'))) {
  const errors = [];
  check(schema, schema, value, '', errors);
  return errors;
}

export function repoFiles() {
  const list = (dir, skip = []) => readdirSync(join(ROOT, dir)).filter((n) => n.endsWith('.json') && !skip.includes(n)).map((n) => join(ROOT, dir, n));
  return [...list('assets/chars', ['index.json']), ...list('crates/native/src/kits'), ...list('docs/examples/umf')];
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  const files = process.argv.length > 2 ? process.argv.slice(2) : repoFiles();
  const schema = JSON.parse(readFileSync(SCHEMA_PATH, 'utf8'));
  let bad = 0;
  for (const file of files) {
    const errors = validate(JSON.parse(readFileSync(file, 'utf8')), schema);
    for (const e of errors.slice(0, 20)) console.error(`${file}: ${e.path}: ${e.message}`);
    bad += errors.length;
  }
  if (bad) { console.error(`umf-schema: ${bad} error(s) in ${files.length} file(s)`); process.exit(1); }
  console.log(`umf-schema: ${files.length} file(s) valid`);
}
