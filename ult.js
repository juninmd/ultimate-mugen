const { execSync } = require('child_process');
const load = (rev, f) => JSON.parse(execSync(`git show ${rev}:assets/chars/${f}.json`, { maxBuffer: 256 * 1024 * 1024 }).toString());
const cases = [['npc_brute','ult_bulldozer'],['npc_colossus','ult_avalanche'],['npc_razor','ult_veil']];
for (const [f, id] of cases) {
  const pr = load('HEAD', f), ms = load('origin/master', f);
  const pm = pr.moves.find(m => m.id === id);
  const mm = ms.moves.find(m => m.id === id);
  console.log(`\n=== ${f} ${id} ===`);
  console.log('MASTER:', JSON.stringify(mm));
  console.log('PR    :', JSON.stringify(pm));
  const prUlt = pr.moves.filter(m => (m.id||'').startsWith('ult_'));
  console.log('PR ults:', JSON.stringify(prUlt.map(m => ({id:m.id,kind:m.kind,cost:m.cost,input:m.input,anim:m.anim}))));
}
