const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { chromium } = require('playwright');

const baseUrl = process.argv.find(value => value.startsWith('http')) || 'https://juninmd.github.io/ultimate-mugen/';
const candidateDir = process.env.UME_CANDIDATE_WEB_DIR;
const outputDir = path.resolve('.tmp/live-web/harness');
const executablePath = process.env.UME_BROWSER_EXECUTABLE;
const base = new URL(baseUrl);
const stagePattern = /\/assets\/stages\/[^/?]+\.png(?:\?|$)/i;
const packPattern = /\/assets\/chars\/sprites\/kaito\.pack(?:\?|$)/i;

fs.mkdirSync(outputDir, { recursive: true });

async function installCandidate(page) {
  if (!candidateDir) return;
  for (const [name, type] of [['ultimate-mugen.js', 'text/javascript'], ['ultimate-mugen_bg.wasm', 'application/wasm']]) {
    await page.route(`**/${name}*`, route => route.fulfill({ path: path.join(candidateDir, name), contentType: type }));
  }
}

async function openPage(context, name, query = '', configure) {
  const page = await context.newPage();
  const errors = [];
  const responses = [];
  const finishedPacks = new Set();
  page.on('pageerror', error => errors.push(error.message));
  page.on('response', response => responses.push({ url: response.url(), status: response.status() }));
  page.on('requestfinished', request => { if (request.url().includes('/assets/chars/sprites/') && request.url().endsWith('.pack')) finishedPacks.add(request.url()); });
  await installCandidate(page);
  if (configure) await configure(page);
  await page.goto(new URL(query, base).href, { waitUntil: 'domcontentloaded', timeout: 90000 });
  return { page, errors, responses, finishedPacks, name };
}

async function selectKaito(page) {
  await page.waitForTimeout(2800);
  await page.locator('#glcanvas').focus();
  for (const key of ['Enter', 'ArrowDown', 'Enter']) {
    await page.keyboard.down(key);
    await page.waitForTimeout(180);
    await page.keyboard.up(key);
    await page.waitForTimeout(350);
  }
  await page.waitForTimeout(900);
}

async function screenshot(page, name) {
  const file = path.join(outputDir, name);
  await page.screenshot({ path: file });
  return fs.readFileSync(file).toString('base64');
}

async function mismatchFraction(page, firstShot, secondShot, crop) {
  return page.evaluate(async ({ a, b, rect }) => {
    const decode = async value => {
      const image = new Image();
      image.src = `data:image/png;base64,${value}`;
      await image.decode();
      const canvas = document.createElement('canvas');
      canvas.width = rect.width;
      canvas.height = rect.height;
      const context = canvas.getContext('2d', { willReadFrequently: true });
      context.drawImage(image, rect.x, rect.y, rect.width, rect.height, 0, 0, rect.width, rect.height);
      return context.getImageData(0, 0, rect.width, rect.height).data;
    };
    const [first, second] = await Promise.all([decode(a), decode(b)]);
    let different = 0;
    for (let index = 0; index < first.length; index += 4) {
      if (Math.abs(first[index] - second[index]) > 16 || Math.abs(first[index + 1] - second[index + 1]) > 16 || Math.abs(first[index + 2] - second[index + 2]) > 16) different++;
    }
    return different / (first.length / 4);
  }, { a: firstShot, b: secondShot, rect: crop });
}

async function failureNoticeBounds(page, image) {
  return page.evaluate(async data => {
    const bitmap = await createImageBitmap(await (await fetch(`data:image/png;base64,${data}`)).blob());
    const canvas = document.createElement('canvas');
    canvas.width = bitmap.width;
    canvas.height = bitmap.height;
    const context = canvas.getContext('2d', { willReadFrequently: true });
    context.drawImage(bitmap, 0, 0);
    const pixels = context.getImageData(0, bitmap.height - 36, bitmap.width, 36).data;
    let count = 0;
    let minX = bitmap.width;
    let maxX = -1;
    for (let y = 0; y < 36; y++) {
      for (let x = 0; x < bitmap.width; x++) {
        const index = (y * bitmap.width + x) * 4;
        const [red, green, blue] = pixels.slice(index, index + 3);
        if (green > 160 && red < 190 && blue > 80 && green - red > 40 && green - blue > 30) {
          count++;
          minX = Math.min(minX, x);
          maxX = Math.max(maxX, x);
        }
      }
    }
    return { count, minX, maxX };
  }, image);
}

async function waitForPack(test, status = 200) {
  const existing = test.responses.find(response => packPattern.test(response.url) && response.status === status);
  if (existing) return existing;
  const response = await test.page.waitForResponse(item => packPattern.test(item.url()) && item.status() === status, { timeout: 60000 });
  return { url: response.url(), status: response.status() };
}

async function waitForFinishedPacks(test, count) {
  const deadline = Date.now() + 90000;
  while (test.finishedPacks.size < count && Date.now() < deadline) await test.page.waitForTimeout(250);
  assert.ok(test.finishedPacks.size >= count, `only ${test.finishedPacks.size} sprite packs finished; expected ${count}`);
}

async function testDelayedPack(browser) {
  const referenceContext = await browser.newContext({ viewport: { width: 1280, height: 720 }, serviceWorkers: 'block' });
  const reference = await openPage(referenceContext, 'reference');
  await waitForFinishedPacks(reference, 16);
  await selectKaito(reference.page);
  const referenceImage = await screenshot(reference.page, 'normal-reference.png');
  const testContext = await browser.newContext({ viewport: { width: 1280, height: 720 }, serviceWorkers: 'block' });
  let release;
  let notify;
  const intercepted = new Promise(resolve => { notify = resolve; });
  const gate = new Promise(resolve => { release = resolve; });
  const test = await openPage(testContext, 'delayed', '', page => page.route('**/assets/chars/sprites/kaito.pack', async route => { notify(); await gate; await route.continue(); }));
  const stageRequests = [];
  test.page.on('request', request => { if (stagePattern.test(request.url())) stageRequests.push(request.url()); });
  const response = test.page.waitForResponse(item => packPattern.test(item.url()), { timeout: 60000 });
  await Promise.race([intercepted, test.page.waitForTimeout(60000).then(() => { throw new Error('Kaito pack request was not intercepted'); })]);
  await selectKaito(test.page);
  const before = await screenshot(test.page, 'delayed-before.png');
  release();
  await response;
  const deadline = Date.now() + 30000;
  let mismatch = 1;
  let after;
  while (mismatch >= 0.1 && Date.now() < deadline) {
    await test.page.waitForTimeout(1000);
    after = await screenshot(test.page, 'delayed-after.png');
    mismatch = await mismatchFraction(test.page, after, referenceImage, { x: 355, y: 120, width: 64, height: 70 });
  }
  const beforeMismatch = await mismatchFraction(test.page, before, referenceImage, { x: 355, y: 120, width: 64, height: 70 });
  await fs.promises.writeFile(path.join(outputDir, 'sprite-comparison.json'), JSON.stringify({ beforeMismatch, afterMismatch: mismatch }, null, 2));
  assert.ok(beforeMismatch > 0.1, `delayed preview unexpectedly matches fresh reference before release (${beforeMismatch})`);
  assert.ok(mismatch < 0.1, `Kaito roster image differs from fresh reference after pack release (${mismatch})`);
  assert.deepEqual(test.errors, [], 'delayed pack flow raised browser page errors');
  await Promise.all([referenceContext.close(), testContext.close()]);
  return { beforeMismatch, afterMismatch: mismatch };
}

async function testFightReadiness(browser) {
  const context = await browser.newContext({ viewport: { width: 1280, height: 720 }, serviceWorkers: 'block' });
  const page = await context.newPage();
  const errors = [];
  const responses = [];
  page.on('pageerror', error => errors.push(error.message));
  let release;
  let notify;
  const gate = new Promise(resolve => { release = resolve; });
  const intercepted = new Promise(resolve => { notify = resolve; });
  const stages = [];
  page.on('request', request => { if (stagePattern.test(request.url())) stages.push(request.url()); });
  await page.route('**/assets/chars/sprites/kaito.pack', async route => { notify(); await gate; await route.continue(); });
  page.on('response', response => responses.push({ url: response.url(), status: response.status() }));
  await installCandidate(page);
  await page.goto(new URL('?p1=kaito&p2=shion&luta=1&modo=treino', base).href, { waitUntil: 'domcontentloaded', timeout: 90000 });
  await Promise.race([intercepted, page.waitForTimeout(60000).then(() => { throw new Error('Deep-link Kaito pack request was not intercepted'); })]);
  await page.waitForTimeout(1200);
  assert.equal(stages.length, 0, 'fight requested full stage art before Kaito sprites were ready');
  release();
  await waitForPack({ page, responses });
  await page.waitForFunction(() => document.querySelector('#glcanvas')?.width > 0, { timeout: 30000 });
  await page.waitForTimeout(3500);
  await screenshot(page, 'fight-after-assets.png');
  assert.ok(stages.length > 0, 'fight did not request stage art after Kaito pack release');
  assert.deepEqual(errors, [], 'deep-link fight raised browser page errors');
  await page.context().close();
  return { stageRequestsAfterRelease: stages.length };
}

async function testFailureAndRecovery(browser) {
  const failedContext = await browser.newContext({ viewport: { width: 1280, height: 720 }, serviceWorkers: 'block' });
  const stages = [];
  const failed = await openPage(failedContext, 'forced-503', '?p1=kaito&p2=shion&luta=1&modo=treino', async page => {
    page.on('request', request => { if (stagePattern.test(request.url())) stages.push(request.url()); });
    await page.route('**/assets/chars/sprites/kaito.pack', route => route.fulfill({ status: 503, body: 'simulated sprite pack failure' }));
  });
  const failedResponse = await waitForPack(failed, 503);
  const failedStatus = failedResponse.status;
  await failed.page.waitForTimeout(2500);
  const failureImage = await screenshot(failed.page, 'forced-503.png');
  const notice = await failureNoticeBounds(failed.page, failureImage);
  assert.equal(failedStatus, 503, 'forced Kaito pack failure was not observed');
  assert.ok(notice.count > 10 && notice.minX >= 12 && notice.maxX < 1268, `failure notice is missing or clipped: ${JSON.stringify(notice)}`);
  assert.equal(stages.length, 0, 'fight requested stage art after Kaito pack failure');
  assert.deepEqual(failed.errors, [], 'pack failure raised an uncaught browser page error');
  await failedContext.close();
  const recoveryContext = await browser.newContext({ viewport: { width: 1280, height: 720 }, serviceWorkers: 'block' });
  const recoveryStages = [];
  const recovery = await openPage(recoveryContext, 'fresh-recovery', '?p1=kaito&p2=shion&luta=1&modo=treino', page => page.on('request', request => { if (stagePattern.test(request.url())) recoveryStages.push(request.url()); }));
  await waitForPack(recovery);
  await recovery.page.waitForTimeout(3500);
  await screenshot(recovery.page, 'fresh-recovery.png');
  assert.ok(recoveryStages.length > 0, 'fresh context did not resume loading stage art');
  assert.deepEqual(recovery.errors, [], 'fresh context recovery raised browser page errors');
  await recoveryContext.close();
  return { forcedPackStatus: failedStatus, notice, recovery: 'HTTP 200' };
}

(async () => {
  const browser = await chromium.launch({ headless: true, executablePath, args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-webgl', '--ignore-gpu-blocklist', '--enable-unsafe-swiftshader'] });
  const mode = candidateDir ? `candidate (${path.resolve(candidateDir)}; local WASM/JS, not deployed)` : 'published website';
  const results = { target: base.href, mode };
  try {
    results.delayedPack = await testDelayedPack(browser);
    results.fightReadiness = await testFightReadiness(browser);
    results.failureRecovery = await testFailureAndRecovery(browser);
    console.log(JSON.stringify(results, null, 2));
    await fs.promises.writeFile(path.join(outputDir, 'result.json'), JSON.stringify(results, null, 2));
  } finally {
    await browser.close();
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
