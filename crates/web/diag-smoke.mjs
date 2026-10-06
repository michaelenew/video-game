// A room that cannot form says why: in the console, and with F10 in the menu.
//
//   ./scripts/web-smoke.sh            # runs this after menu-smoke
//   node crates/web/diag-smoke.mjs http://127.0.0.1:8080/ target/diag-smoke
//
// The room is pointed at a broker where nothing listens (`?broker=` on a port
// nobody uses), so it fails the same way on every machine, with or without
// the internet -- the way the public brokers failed the first time two real
// people tried. Then:
//
// - the console must hold the meeting's report, unasked: the failure, and the
//   broker's own story ending in the close code;
// - F10 must switch dev mode on mid-session, and say so;
// - the Esc menu, in dev mode, shows the same details (the screenshot).
import { createRequire } from 'node:module';
import { execSync } from 'node:child_process';

const root = execSync('npm root -g').toString().trim();
const { chromium } = createRequire(root + '/')('playwright');

const [url, prefix = 'target/diag-smoke'] = process.argv.slice(2);
if (!url) {
  console.error('usage: node crates/web/diag-smoke.mjs <url> [screenshot-prefix]');
  process.exit(2);
}
const broker = 'ws://127.0.0.1:9/mqtt';
const browser = await chromium.launch({
  args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist'],
});
const page = await browser.newPage({ viewport: { width: 900, height: 700 } });
const logged = [];
const errors = [];
page.on('console', (m) => logged.push(m.text()));
page.on('pageerror', (e) => errors.push(`page: ${e.message}`));
const join = `${url}${url.includes('?') ? '&' : '?'}room=diagno&broker=${encodeURIComponent(broker)}#key=abcdefghjkmnpqrstuvwxyz234`;
await page.goto(join, { waitUntil: 'load', timeout: 120_000 });

const deadline = Date.now() + 90_000;
while (Date.now() < deadline && !logged.some((l) => l.startsWith('meeting failed'))) {
  await page.waitForTimeout(1_000);
}
await page.locator('canvas').focus();
await page.keyboard.press('F10');
await page.waitForTimeout(2_000);
await page.keyboard.press('Escape');
await page.waitForTimeout(4_000);
await page.screenshot({ path: `${prefix}.png` });
await browser.close();

const report = logged.slice(logged.findIndex((l) => l.startsWith('meeting failed')));
console.log('console after the failure:');
for (const line of report.slice(0, 20)) console.log(`  ${line}`);
const want = [
  'meeting failed: Could not reach any meeting point',
  `meeting point ${broker}: down after`,
  'opening a WebSocket',
  'code 1006',
  'friend: not heard from',
  'sealed room, topic',
  'dev mode on (F10)',
];
for (const w of want) {
  if (!logged.some((l) => l.includes(w))) errors.push(`the console never said: ${w}`);
}
console.log(`screenshot: ${prefix}.png`);
if (errors.length) {
  console.error(`${errors.length} problem(s):`);
  for (const e of errors.slice(0, 10)) console.error(`  ${e}`);
  process.exit(1);
}
console.log('a room that could not form said why');
