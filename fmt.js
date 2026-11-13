const { execSync } = require('child_process');
for (const f of ['npc_brute', 'npc_razor']) {
  const raw = execSync(`git show HEAD:assets/chars/${f}.json`, { maxBuffer: 256 * 1024 * 1024 }).toString();
  const body = raw.replace(/\r?\n$/, '');
  console.log(`\n${f}: len=${raw.length} crlf=${raw.includes('\r\n')} fim=${JSON.stringify(raw.slice(-4))}`);
  for (const indent of [1, 2, '\t']) {
    const out = JSON.stringify(JSON.parse(raw), null, indent);
    console.log(`  indent=${JSON.stringify(indent)} igual=${out === body.replace(/\r\n/g, '\n')} / ${out === body}`);
  }
}
