// Web perf probe: serves web/dist, drives the menus in headless Chromium and prints main-thread self time + slow frames.
// usage: NODE_PATH=$(npm root -g) node scripts/web-profile.js Enter Enter Enter
const { chromium } = require('playwright');
const http = require('http'), fs = require('fs'), path = require('path');
const root = process.env.DIST || require('path').resolve(__dirname, '../web/dist');
const types = {'.html':'text/html','.js':'text/javascript','.wasm':'application/wasm','.svg':'image/svg+xml','.webmanifest':'application/json'};
const srv = http.createServer((q,r)=>{ let p = q.url.split('?')[0]; if(p==='/')p='/index.html'; const f=path.join(root,p); fs.readFile(f,(e,d)=>{ if(e){r.writeHead(404);r.end();return;} r.writeHead(200,{'content-type':types[path.extname(f)]||'application/octet-stream'}); r.end(d);});}).listen(8098);
(async()=>{
  const b = await chromium.launch({args:['--use-gl=angle','--use-angle=swiftshader','--enable-unsafe-swiftshader','--ignore-gpu-blocklist']});
  const pg = await b.newPage({viewport:{width:1280,height:720}});
  pg.on('console', m=>console.log('console:', m.text().slice(0,300)));
  pg.on('pageerror', e=>console.log('pageerror:', e.message.slice(0,300)));
  await pg.addInitScript(()=>{
    window.__ft=[]; let last=performance.now();
    const loop=t=>{ window.__ft.push([Math.round(t),+(t-last).toFixed(1)]); last=t; requestAnimationFrame(loop); }; requestAnimationFrame(loop);
  });
  const cdp = await pg.context().newCDPSession(pg);
  await cdp.send('Profiler.enable'); await cdp.send('Profiler.setSamplingInterval',{interval:500});
  const t0=Date.now();
  await pg.goto('http://localhost:8098/');
  await pg.waitForTimeout(6000);
  await cdp.send('Profiler.start');
  const steps = process.argv.slice(2);
  await pg.mouse.click(640,360);
  for (const spec of steps) { const [k,rep='1']=spec.split('*'); for(let r=0;r<+rep;r++){ await pg.keyboard.down(k); await pg.waitForTimeout(rep==='1'?300:80); await pg.keyboard.up(k); await pg.waitForTimeout(rep==='1'?3500:120);}  await pg.screenshot({path:`/tmp/p_${k}_${Date.now()%100000}.png`}); }
  const {profile} = await cdp.send('Profiler.stop');
  const self = {}; const dt = profile.timeDeltas; const byId={}; profile.nodes.forEach(n=>byId[n.id]=n);
  profile.samples.forEach((id,i)=>{ const n=byId[id]; const k=n.callFrame.functionName||'(anon)'; self[k]=(self[k]||0)+dt[i]/1000; });
  const busy = Object.entries(self).filter(([k])=>k!=='(idle)'&&k!=='(program)'&&k!=='(garbage collector)').reduce((a,[,v])=>a+v,0); console.log('BUSY_MS', busy.toFixed(0));
  const top = Object.entries(self).sort((a,b)=>b[1]-a[1]).slice(0,25);
  console.log(top.map(([k,v])=>v.toFixed(0)+'ms '+k).join('\n'));
  const ft = await pg.evaluate(()=>window.__ft);
  const big = ft.filter(x=>x[1]>150).slice(0,30); console.log('frames>150ms:', JSON.stringify(big));
  await b.close(); srv.close();
})();
