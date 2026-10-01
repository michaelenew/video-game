// Load the published browser build in headless Chromium and say what it did.
//
//   ./scripts/web-smoke.sh            # builds if needed, serves, runs this
//   node crates/web/smoke.mjs http://127.0.0.1:8080/ target/web-smoke.png
//
// It is the browser-side half of `build-game.sh`'s validation: that script
// proves a browser *can compile* the module, and this proves the page then
// starts -- the module loads, the canvas gets a frame, the controls panel came
// out of the manual -- with no console error and no failed request. It exits
// non-zero on any of those, and prints the first few so the report can name
// them. The screenshot it leaves is the same evidence a person would give.
//
// Playwright comes from the global npm root (`./scripts/setup-tools.sh
// browser` puts it there) and finds its own Chromium; a machine that pins one
// elsewhere sets PLAYWRIGHT_BROWSERS_PATH, as the cloud containers do.
import { createRequire } from 'node:module';
import { execSync } from 'node:child_process';

const root = execSync('npm root -g').toString().trim();
const { chromium } = createRequire(root + '/')('playwright');

const [url, shot] = process.argv.slice(2);
if (!url) {
  console.error('usage: node crates/web/smoke.mjs <url> [screenshot.png]');
  process.exit(2);
}

const browser = await chromium.launch({
  // A software GPU: the page has to draw somewhere, and a headless runner
  // has no hardware.
  args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist'],
});
const page = await browser.newPage({ viewport: { width: 1280, height: 760 } });
const errors = [];
page.on('console', (m) => {
  if (m.type() === 'error') errors.push(`console: ${m.text()}`);
});
page.on('pageerror', (e) => errors.push(`page: ${e.message}`));
page.on('requestfailed', (r) => errors.push(`request: ${r.url()} ${r.failure()?.errorText}`));

// `?dev` is `--dev`: the Oven and the overlay open, which exercises more of
// the page than the plain load does.
// A URL that already asks for something (`?hunt=gnawers`) gets `&dev`.
await page.goto(url + (url.includes('?') ? '&dev' : '?dev'), { waitUntil: 'load', timeout: 120_000 });
// The module is twenty-odd megabytes and compiles on load; give it time to
// start and draw.
await page.waitForTimeout(25_000);

const canvas = await page.evaluate(() => {
  const c = document.querySelector('canvas');
  return c ? { width: c.width, height: c.height } : null;
});
const text = await page.evaluate(() => document.body.innerText);
// The controls panel is generated from the manual, so a binding the game has
// and the page does not describe is a build that did not run the manual.
const expected = ['Mouse side buttons', 'F and R', 'Fissure', 'Updraft'];
const missing = expected.filter((k) => !text.includes(k));
if (!canvas) errors.push('no canvas on the page');
if (missing.length) errors.push(`controls panel is missing: ${missing.join(', ')}`);

if (shot) await page.screenshot({ path: shot });
await browser.close();

console.log(`canvas: ${canvas ? `${canvas.width}x${canvas.height}` : 'none'}`);
console.log(`controls panel: ${expected.length - missing.length} of ${expected.length} expected entries`);
if (shot) console.log(`screenshot: ${shot}`);
if (errors.length) {
  console.error(`${errors.length} problem(s):`);
  for (const e of errors.slice(0, 10)) console.error(`  ${e}`);
  process.exit(1);
}
console.log('the page starts clean');
