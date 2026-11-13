const { execSync } = require('child_process');
const files = ['npc_brute', 'npc_colossus', 'npc_phantom', 'npc_razor', 'npc_spitfire'];
const base = execSync('git merge-base HEAD origin/master').toString().trim();
const load = (rev, f) => JSON.parse(execSync(`git show ${rev}:assets/chars/${f}.json`, { maxBuffer: 256 * 1024 * 1024 }).toString());

function shapes(a, b, path, out) {
  const ta = Array.isArray(a) ? 'array' : typeof a;
  const tb = Array.isArray(b) ? 'array' : typeof b;
  if (ta !== tb) { out.push(`${path}: tipo ${ta} -> ${tb}`); return; }
  if (ta === 'object' && a && b) {
    for (const k of new Set([...Object.keys(a), ...Object.keys(b)])) {
      if (!(k in a)) out.push(`${path}.${k}: +novo`);
      else if (!(k in b)) out.push(`${path}.${k}: -removido`);
      else shapes(a[k], b[k], `${path}.${k}`, out);
    }
  } else if (ta === 'array') {
    if (a.length !== b.length) out.push(`${path}: array ${a.length} -> ${b.length}`);
  } else if (a !== b) out.push(`${path}: ${JSON.stringify(a)} -> ${JSON.stringify(b)}`);
}

for (const f of files) {
  console.log(`\n=== ${f} ===`);
  const b = load(base, f), pr = load('HEAD', f), ms = load('origin/master', f);
  const a = [], m = [];
  shapes(b, pr, f, a);
  shapes(b, ms, f, m);
  console.log(`-- mudancas do PR (${a.length}) --`);
  console.log(a.slice(0, 40).join('\n') + (a.length > 40 ? `\n... +${a.length - 40}` : ''));
  console.log(`-- mudancas do master (${m.length}) --`);
  console.log(m.slice(0, 40).join('\n') + (m.length > 40 ? `\n... +${m.length - 40}` : ''));
}
