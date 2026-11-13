const { execSync } = require('child_process');
const fs = require('fs');
const files = ['npc_brute', 'npc_colossus', 'npc_phantom', 'npc_razor', 'npc_spitfire'];
const load = (rev, f) => JSON.parse(execSync(`git show ${rev}:assets/chars/${f}.json`, { maxBuffer: 256 * 1024 * 1024 }).toString());
const isUlt = m => (m.id || '').startsWith('ult_');

for (const f of files) {
  const pr = load('HEAD', f);
  const ms = load('origin/master', f);
  const msUlt = ms.moves.find(isUlt);
  const prUlt = pr.moves.find(isUlt);
  if (!prUlt) {
    const at = Math.min(ms.moves.findIndex(isUlt), pr.moves.length);
    pr.moves.splice(at, 0, msUlt);
    console.log(`${f}: sem ult no PR -> adicionado ${msUlt.id} em ${at}`);
  } else {
    console.log(`${f}: mantido ult do PR ${prUlt.id} (master tinha ${msUlt.id})`);
  }
  fs.writeFileSync(`assets/chars/${f}.json`, JSON.stringify(pr, null, 1) + '\n');
}
