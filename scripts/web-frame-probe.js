// Per-phase frame-time probe for the web build. usage: NODE_PATH=$(npm root -g) node scripts/web-frame-probe.js <outDir> "Enter" "ArrowRight*20" ...
const { chromium } = require('playwright');
const http = require('http'), fs = require('fs'), path = require('path');
const root = process.env.DIST || path.resolve(__dirname, '../web/dist');
const types = {'.html':'text/html','.js':'text/javascript','.wasm':'application/wasm','.svg':'image/svg+xml','.webmanifest':'application/json'};
const srv = http.createServer((q,r)=>{ let p = q.url.split('?')[0]; if(p==='/')p='/index.html'; const f=path.join(root,p); fs.readFile(f,(e,d)=>{ if(e){r.writeHead(404);r.end();return;} r.writeHead(200,{'content-type':types[path.extname(f)]||'application/octet-stream'}); r.end(d);});}).listen(8099);
const pct = (a,p)=>a.length?a.slice().sort((x,y)=>x-y)[Math.min(a.length-1,Math.floor(a.length*p))]:0;
(async()=>{
  const out = process.argv[2]; fs.mkdirSync(out,{recursive:true});
  const b = await chromium.launch({args:['--use-gl=angle','--use-angle=swiftshader','--enable-unsafe-swiftshader','--ignore-gpu-blocklist']});
  const pg = await b.newPage({viewport:{width:1280,height:720}});
  pg.on('console', m=>{ const t=m.text(); if(t.includes('[perf]')) console.log(t.slice(0,200)); });
  pg.on('pageerror', e=>console.log('pageerror:', e.message.slice(0,300)));
  await pg.addInitScript(()=>{
    window.__ft=[]; window.__lt=[]; let last=performance.now();
    const loop=t=>{ window.__ft.push([t,t-last]); last=t; requestAnimationFrame(loop); }; requestAnimationFrame(loop);
    new PerformanceObserver(l=>l.getEntries().forEach(e=>window.__lt.push([e.startTime,e.duration]))).observe({entryTypes:['longtask']});
  });
  await pg.goto('http://localhost:8099/?perf=1');
  await pg.waitForTimeout(8000);
  await pg.mouse.click(640,360);
  const phases = process.argv.slice(3);
  const rows = [];
  for (const [n,spec] of phases.entries()) {
    const [key,rep='1',wait='2500'] = spec.split('*');
    const t0 = await pg.evaluate(()=>performance.now());
    for (let i=0;i<+rep;i++){ await pg.keyboard.down(key); await pg.waitForTimeout(80); await pg.keyboard.up(key); await pg.waitForTimeout(120); }
    await pg.waitForTimeout(+wait);
    const t1 = await pg.evaluate(()=>performance.now());
    const {ft,lt} = await pg.evaluate(()=>({ft:window.__ft,lt:window.__lt}));
    const f = ft.filter(x=>x[0]>=t0&&x[0]<=t1).map(x=>x[1]);
    const l = lt.filter(x=>x[0]>=t0&&x[0]<=t1).map(x=>x[1]);
    rows.push({phase:`${n}:${spec}`,frames:f.length,p50:+pct(f,.5).toFixed(0),p95:+pct(f,.95).toFixed(0),max:+Math.max(0,...f).toFixed(0),longTasks:l.length,longTaskMs:+l.reduce((a,c)=>a+c,0).toFixed(0),maxLongTask:+Math.max(0,...l).toFixed(0)});
    await pg.screenshot({path:path.join(out,`${n}.png`)});
  }
  console.table(rows);
  fs.writeFileSync(path.join(out,'frames.json'),JSON.stringify(rows,null,1));
  await b.close(); srv.close();
})();
