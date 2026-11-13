const { execSync } = require('child_process');
const files = ['npc_brute', 'npc_colossus', 'npc_phantom', 'npc_razor', 'npc_spitfire'];
const base = execSync('git merge-base HEAD origin/master').toString().trim();
const load = (rev, f) => JSON.parse(execSync(`git show ${rev}:assets/chars/${f}.json`, { maxBuffer: 256 * 1024 * 1024 }).toString());
for (const f of files) {
  const b = load(base, f), pr = load('HEAD', f), ms = load('origin/master', f);
  const id = m => m.id || m.name;
  const bid = b.moves.map(id), mid = ms.moves.map(id), pid = pr.moves.map(id);
  const addedMaster = mid.filter(x => !bid.includes(x));
  const addedPr = pid.filter(x => !bid.includes(x));
  console.log(`\n${f}`);
  console.log(`  master adicionou: ${JSON.stringify(addedMaster)}`);
  console.log(`  PR adicionou (${addedPr.length}): ${JSON.stringify(addedPr)}`);
  console.log(`  master posicao: ${mid.indexOf(addedMaster[0])}/${mid.length}`);
  console.log(`  PR posicao do ultimo move: ${pid.length}`);
  const extra = ms.moves.find(m => id(m) === addedMaster[0]);
  console.log(`  ult master: kind=${extra.kind} cost=${extra.cost} input=${extra.input} anim=${extra.anim}`);
  console.log(`  pr tem anim ${extra.anim}? ${!!pr.anims[extra.anim]}`);
}
