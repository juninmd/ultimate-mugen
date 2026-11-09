// Automated Headless Gameplay Recording & GIF/Screenshot Capture Tool
// Drives web build via Playwright Chromium, records WebM, extracts HD shots,
// and transposes into Bayer-dithered GIF via ffmpeg.
const fs = require('fs');
const path = require('path');
const os = require('os');
const { execSync } = require('child_process');
const { staticServer } = require('./web-smoke-server');

function parseArgs(argv) {
  const opts = {
    out: path.resolve('docs/evidence/recordings'),
    gif: 'gameplay.gif',
    duration: 6,
    dist: path.resolve(__dirname, '..', 'web', 'dist'),
    shots: true,
  };
  for (let i = 2; i < argv.length; i++) {
    if (argv[i] === '--out' && argv[i + 1]) opts.out = path.resolve(argv[++i]);
    else if (argv[i] === '--gif' && argv[i + 1]) opts.gif = argv[++i];
    else if (argv[i] === '--duration' && argv[i + 1]) opts.duration = parseFloat(argv[++i]);
    else if (argv[i] === '--dist' && argv[i + 1]) opts.dist = path.resolve(argv[++i]);
    else if (argv[i] === '--no-shots') opts.shots = false;
  }
  return opts;
}

const types = {
  '.html': 'text/html',
  '.js': 'text/javascript',
  '.wasm': 'application/wasm',
  '.svg': 'image/svg+xml',
  '.json': 'application/json',
  '.webmanifest': 'application/manifest+json',
  '.zip': 'application/zip',
};

async function driveRecording(opts) {
  fs.mkdirSync(opts.out, { recursive: true });
  const server = staticServer(opts.dist, types);
  await new Promise(ok => server.listen(0, '127.0.0.1', ok));
  const port = server.address().port;
  const url = `http://127.0.0.1:${port}/index.html`;

  const tmpVideoDir = fs.mkdtempSync(path.join(os.tmpdir(), 'ume-rec-'));
  const { chromium } = require('playwright');
  const browser = await chromium.launch({
    args: [
      '--no-sandbox',
      '--disable-setuid-sandbox',
      '--use-gl=angle',
      '--use-angle=swiftshader',
      '--enable-webgl',
      '--ignore-gpu-blocklist',
      '--enable-unsafe-swiftshader',
    ],
  });

  const context = await browser.newContext({
    viewport: { width: 1280, height: 720 },
    recordVideo: { dir: tmpVideoDir, size: { width: 1280, height: 720 } },
  });
  const page = await context.newPage();

  console.log(`Loading game from ${url}...`);
  await page.goto(url);
  await page.locator('#boot').waitFor({ state: 'detached', timeout: 45000 }).catch(() => {});
  await page.waitForTimeout(10000);
  await page.locator('#glcanvas').focus();

  const frames = n => page.evaluate(n => new Promise(done => {
    const step = i => (i ? requestAnimationFrame(() => step(i - 1)) : done());
    step(n);
  }), n);

  const key = async (k, wait = 1000) => {
    await page.keyboard.down(k);
    await frames(2);
    await page.keyboard.up(k);
    await frames(5);
    await page.waitForTimeout(wait);
  };

  const snap = async (name) => {
    if (!opts.shots) return;
    const p = path.join(opts.out, `${name}.png`);
    await page.screenshot({ path: p });
    console.log(`Saved screenshot: ${p}`);
  };

  await key('Enter');
  await snap('01_menu');

  for (let i = 0; i < 3; i++) await key('ArrowDown', 400);
  await key('Enter', 1500);
  await snap('02_select');

  await key('Enter', 1200);
  await key('ArrowRight', 400);
  await key('Enter', 1200);
  await snap('03_stage');

  await key('Enter', 1200);
  await page.waitForTimeout(5000);
  await snap('04_fight_start');

  const fightEnd = Date.now() + opts.duration * 1000;
  const fightActions = ['KeyD', 'KeyS', 'KeyA', 'KeyW', 'KeyJ', 'KeyK', 'KeyU', 'KeyI'];
  let actionIdx = 0;

  while (Date.now() < fightEnd) {
    const act = fightActions[actionIdx % fightActions.length];
    await page.keyboard.down(act);
    await page.waitForTimeout(120);
    await page.keyboard.up(act);
    await page.waitForTimeout(200);
    actionIdx++;
  }

  await snap('05_fight_action');

  await page.close();
  await context.close();
  await browser.close();
  server.close();

  const videoFiles = fs.readdirSync(tmpVideoDir).filter(f => f.endsWith('.webm'));
  if (videoFiles.length > 0 && opts.gif) {
    const webmPath = path.join(tmpVideoDir, videoFiles[0]);
    const gifPath = path.join(opts.out, opts.gif);
    const filter = 'fps=20,scale=480:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=128[p];[s1][p]paletteuse=dither=bayer:bayer_scale=3';
    console.log(`Generating Bayer-dithered GIF: ${gifPath}...`);
    execSync(`ffmpeg -y -i "${webmPath}" -vf "${filter}" "${gifPath}"`, { stdio: 'inherit' });
    console.log(`GIF generated: ${gifPath}`);
  }

  fs.rmSync(tmpVideoDir, { recursive: true, force: true });
  console.log(`Capture complete! Files saved to: ${opts.out}`);
}

if (require.main === module) {
  const opts = parseArgs(process.argv);
  driveRecording(opts).catch(err => {
    console.error('Recording failed:', err);
    process.exit(1);
  });
}

module.exports = { parseArgs, driveRecording };
