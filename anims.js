const { execSync } = require('child_process');
const load = (rev, f) => JSON.parse(execSync(`git show ${rev}:assets/chars/${f}.json`, { maxBuffer: 256 * 1024 * 1024 }).toString());
const cases = [
  ['npc_brute', 'ult_bulldozer', 'titanSuplex', 'titanSuplexUlt'],
  ['npc_colossus', 'ult_avalanche', 'avalanche', 'avalancheUlt'],
  ['npc_phantom', 'ult_shadowslash', 'shadowslash', 'shadowslashUlt'],
  ['npc_razor', 'ult_veil', 'veil', null],
  ['npc_spitfire', 'ult_cometstorm', 'stormSuper', 'stormSuperUlt'],
];
function animInfo(c, name) {
  const a = c.anims && c.anims[name];
  if (!a) return 'AUSENTE';
  const files = new Set();
  let frames = 0;
  for (const f of a.frames || []) {
    frames++;
    const img = f.img || f.file || f.image || f.sprite;
    if (img) files.add(img);
  }
  return `frames=${frames} imgs=${files.size} ${[...files].slice(0,4).join(',')}`;
}
for (const [f, id, mA, pA] of cases) {
  const pr = load('HEAD', f), ms = load('origin/master', f);
  console.log(`\n=== ${f} ===`);
  console.log(`  master ult ${id} anim=${mA} -> master: ${animInfo(ms, mA)}`);
  console.log(`  master ult anim no PR: ${animInfo(pr, mA)}`);
  if (pA) {
    const pm = pr.moves.find(m => (m.id||'').startsWith('ult_'));
    console.log(`  PR ult ${pm && pm.id} anim=${pm && pm.anim} -> PR: ${animInfo(pr, pA)}`);
  }
  console.log(`  forms: master=${JSON.stringify((ms.forms||[]).length)} pr=${JSON.stringify((pr.forms||[]).length)}`);
  const prIds = pr.moves.map(m => m.id);
  const msIds = ms.moves.map(m => m.id);
  console.log(`  moves master=${msIds.length} pr=${prIds.length}  sobreposicao=${prIds.filter(x => msIds.includes(x)).length}`);
}
