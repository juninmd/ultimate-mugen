const { execSync } = require('child_process');
const fs = require('fs');
const files = ['npc_brute', 'npc_colossus', 'npc_phantom', 'npc_razor', 'npc_spitfire'];
const load = (rev, f) => JSON.parse(execSync(`git show ${rev}:assets/chars/${f}.json`, { maxBuffer: 256 * 1024 * 1024 }).toString());
const isUlt = m => (m.id || '').startsWith('ult_');

for (const f of files) {
  const pr = load('HEAD', f);
  const ms = load('origin/master', f);
  const msUlts = ms.moves.filter(isUlt);
  if (msUlts.length !== 1) throw new Error(`${f}: master tem ${msUlts.length} ults`);
  const msIdx = ms.moves.findIndex(isUlt);

  const prUltIds = pr.moves.filter(isUlt).map(m => m.id);
  const moves = pr.moves.filter(m => !isUlt(m));
  const at = Math.min(msIdx, moves.length);
  moves.splice(at, 0, msUlts[0]);
  pr.moves = moves;

  fs.writeFileSync(`assets/chars/${f}.json`, JSON.stringify(pr, null, 1) + '\n');
  console.log(`${f}: removidos [${prUltIds}], inserido ${msUlts[0].id} no indice ${at}; moves=${moves.length}`);
}
