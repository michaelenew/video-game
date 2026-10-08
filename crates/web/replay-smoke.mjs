// The page saves a replay, and the replay is a file the harness can judge.
//
//   ./scripts/web-smoke.sh            # runs this after smoke.mjs
//   node crates/web/replay-smoke.mjs http://127.0.0.1:8080/ target/web-smoke
//
// The page is loaded, left to play a few seconds against the dummy, and `Y`
// is pressed: the game finishes its tape and hands the browser a download
// (`platform::save_replay`, the browser half). The download is saved beside
// the screenshots and read back: it has to be a replay, of the fight the page
// was started in, with frames on it. `scripts/web-smoke.sh` then runs
// `cargo run -p hunt --bin replay` over it, which is the proof that matters --
// this build reproduces, bit for bit, what the page recorded.
import { createRequire } from 'node:module';
import { execSync } from 'node:child_process';
import { readFileSync } from 'node:fs';

const root = execSync('npm root -g').toString().trim();
const { chromium } = createRequire(root + '/')('playwright');

const [url, prefix = 'target/web-smoke'] = process.argv.slice(2);
if (!url) {
  console.error('usage: node crates/web/replay-smoke.mjs <url> [prefix]');
  process.exit(2);
}
const browser = await chromium.launch({
  args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist'],
});
const context = await browser.newContext({ viewport: { width: 900, height: 700 }, acceptDownloads: true });
const errors = [];
const page = await context.newPage();
page.on('console', (m) => m.type() === 'error' && errors.push(`console: ${m.text()}`));
page.on('pageerror', (e) => errors.push(`page: ${e.message}`));
const logs = [];
page.on('console', (m) => logs.push(m.text()));
await page.goto(url, { waitUntil: 'load', timeout: 120_000 });
// Let it fight for a while: a tape of nothing proves little.
await page.waitForTimeout(20_000);
await page.locator('canvas').focus();
const download = page.waitForEvent('download', { timeout: 30_000 });
await page.keyboard.press('KeyY');
const file = `${prefix}.replay`;
let name = '(no download)';
try {
  const d = await download;
  name = d.suggestedFilename();
  await d.saveAs(file);
} catch (e) {
  errors.push(`Y did not download a replay: ${e.message}`);
}
await page.waitForTimeout(1_000);
await browser.close();

if (!errors.length) {
  const text = readFileSync(file, 'utf8');
  const lines = text.split('\n');
  console.log(`downloaded ${name}: ${lines.length} lines, header:`);
  for (const l of lines.slice(0, 6)) console.log(`  ${l}`);
  if (lines[0] !== 'replay 1') errors.push(`the download does not start with "replay 1": ${lines[0]}`);
  if (!lines.some((l) => l.startsWith('start '))) errors.push('the download has no start line');
  if (!lines.some((l) => l.startsWith('end '))) errors.push('the download has no end line');
  // A frame line may stand for a run of identical frames: `... x149`.
  const frames = lines
    .filter((l) => /^[0-9a-f]{4} /.test(l))
    .reduce((n, l) => n + (Number((l.match(/ x(\d+)$/) ?? [])[1]) || 1), 0);
  console.log(`${frames} frames on the tape`);
  if (frames < 10) errors.push(`only ${frames} frames: the page barely ran`);
  const said = logs.find((l) => l.includes('replay saved'));
  console.log(`the page said: ${said ?? '(nothing about the save)'}`);
  if (!said) errors.push('the console never said the replay was saved');
}
if (errors.length) {
  console.error(`${errors.length} problem(s):`);
  for (const e of errors.slice(0, 10)) console.error(`  ${e}`);
  process.exit(1);
}
console.log(`replay saved from the page: ${file}`);
