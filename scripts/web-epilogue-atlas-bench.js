const fs = require('fs');
const path = require('path');
const { staticServer } = require('./web-smoke-server');
const { chromium } = require('playwright');

const dir = path.resolve(process.argv[2] || path.join(__dirname, '..', 'web', 'dist'));
const shots = process.argv[3];
const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.json': 'application/json', '.zip': 'application/zip' };
const server = staticServer(dir, types);

const run = async () => {
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const browser = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-webgl', '--ignore-gpu-blocklist', '--enable-unsafe-swiftshader'] });
  try {
    const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
    await page.goto(`http://127.0.0.1:${server.address().port}/index.html?e2e=1&e2e_epilogue=1`);
    await page.locator('#boot').waitFor({ state: 'detached', timeout: 45000 }).catch(() => {});
    await page.waitForTimeout(12000);
    await page.locator('#glcanvas').focus();
    if (shots) fs.mkdirSync(shots, { recursive: true });
    const state = () => page.evaluate(() => JSON.parse(globalThis.__ume_state || 'null'));
    const frame = () => page.evaluate(() => new Promise(resolve => requestAnimationFrame(resolve)));
    const enter = async () => {
      await page.keyboard.down('Enter');
      await frame();
      await page.keyboard.up('Enter');
      await page.waitForTimeout(800);
    };
    const reach = async target => {
      const deadline = Date.now() + 90000;
      while (((await state())?.epilogue ?? -1) < target && Date.now() < deadline) await enter();
      if (((await state())?.epilogue ?? -1) < target) throw new Error(`could not reach epilogue page ${target}`);
    };
    await reach(0);
    await page.waitForTimeout(2500);
    if (shots) await page.screenshot({ path: path.join(shots, 'page-0.png') });
    const samples = [];
    await page.evaluate(() => {
      globalThis.__atlas_samples = [];
      const sample = () => {
        const current = JSON.parse(globalThis.__ume_state || 'null');
        if (current?.epilogue >= 0 && current.gpu) globalThis.__atlas_samples.push({ page: current.epilogue, ...current.gpu });
        requestAnimationFrame(sample);
      };
      requestAnimationFrame(sample);
    });
    for (let target = 1; target <= 4; target += 1) {
      await reach(target);
      if (target === 1) {
        await page.waitForTimeout(2500);
        if (shots) await page.screenshot({ path: path.join(shots, 'page-1.png') });
      }
    }
    samples.push(...await page.evaluate(() => globalThis.__atlas_samples));
    const result = {
      frames: samples.length,
      upload_operations: samples.reduce((sum, sample) => sum + (sample.up || 0), 0),
      upload_kb: samples.reduce((sum, sample) => sum + (sample.kb || 0), 0),
      atlas_resets: [...new Set(samples.map(sample => sample.resets))],
      atlas_generations: [...new Set(samples.map(sample => sample.tresets))],
    };
    console.log(JSON.stringify(result));
  } finally {
    await browser.close();
    server.close();
  }
};

run().catch(error => { console.error(error); process.exit(1); });
