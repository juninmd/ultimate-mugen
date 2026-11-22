const fs = require('fs');
const path = require('path');

const out = process.env.WEB_QA_OUT;
const strict = process.env.WEB_QA_STRICT === '1';
const fatalText = /panicked|PanicHookInfo|unreachable executed|gpu error:|renderbufferStorageMultisample/;
const ignoredPageError = /register_plugin/;
const ignoredRequest = /favicon|sw\.js/;
const maxProblemShots = 4;

if (out) {
  fs.mkdirSync(out, { recursive: true });
  const consoleLog = path.join(out, 'console.log');
  const networkLog = path.join(out, 'network.log');
  const problems = [];
  const started = Date.now();
  const stamp = () => `+${((Date.now() - started) / 1000).toFixed(1)}s`;
  const append = (file, line) => fs.appendFileSync(file, `${stamp()} ${line}\n`);
  const seen = new Map();
  let problemShots = 0;

  const shot = async (page, name) => {
    if (page.isClosed()) return;
    await page.screenshot({ path: path.join(out, name), timeout: 8000 }).catch(() => {});
  };

  const pageId = page => {
    if (!seen.has(page)) seen.set(page, seen.size + 1);
    return seen.get(page);
  };

  const report = (page, kind, text, fatal) => {
    const line = `${kind}: ${text.slice(0, 600)}`;
    append(consoleLog, `[problem] ${line}`);
    if (!fatal) return;
    problems.push(line);
    if (problemShots < maxProblemShots) {
      problemShots += 1;
      shot(page, `problem-${problemShots}.png`);
    }
  };

  const watchPage = page => {
    if (seen.has(page)) return;
    const id = pageId(page);
    page.on('console', message => {
      const text = message.text();
      append(consoleLog, `[p${id}] ${message.type()}: ${text.slice(0, 800)}`);
      if (fatalText.test(text)) report(page, 'panic', text, true);
    });
    page.on('pageerror', error => {
      const text = `${error.message}${error.stack ? `\n${error.stack}` : ''}`;
      report(page, 'pageerror', text, !ignoredPageError.test(error.message));
    });
    page.on('crash', () => report(page, 'crash', 'Chromium page crashed', true));
    page.on('requestfailed', request => {
      if (!ignoredRequest.test(request.url())) append(networkLog, `FAILED ${request.url()} ${request.failure()?.errorText || ''}`);
    });
    page.on('response', response => {
      if (response.status() >= 400 && !ignoredRequest.test(response.url())) append(networkLog, `${response.status()} ${response.url()}`);
    });
  };

  const finalShots = async context => {
    await Promise.all(context.pages().map(page => shot(page, `final-${pageId(page)}.png`)));
  };

  const watchContext = context => {
    if (context.__umeGuard) return;
    context.__umeGuard = true;
    context.pages().forEach(watchPage);
    context.on('page', watchPage);
    const close = context.close.bind(context);
    context.close = async (...args) => {
      await finalShots(context);
      return close(...args);
    };
  };

  const watchBrowser = browser => {
    const newContext = browser.newContext.bind(browser);
    browser.newContext = async (...args) => {
      const context = await newContext(...args);
      watchContext(context);
      return context;
    };
    const newPage = browser.newPage.bind(browser);
    browser.newPage = async (...args) => {
      const page = await newPage(...args);
      watchContext(page.context());
      watchPage(page);
      return page;
    };
    const close = browser.close.bind(browser);
    browser.close = async (...args) => {
      await Promise.all(browser.contexts().map(finalShots));
      return close(...args);
    };
    return browser;
  };

  const { chromium } = require('playwright');
  const launch = chromium.launch.bind(chromium);
  chromium.launch = async (...args) => watchBrowser(await launch(...args));

  process.on('exit', code => {
    const summary = { exitCode: code, strict, problems };
    fs.writeFileSync(path.join(out, 'guard.json'), JSON.stringify(summary, null, 2));
    if (strict && problems.length && !code) process.exitCode = 1;
  });
}
