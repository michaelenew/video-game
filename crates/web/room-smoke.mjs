// Two tabs of one headless Chromium open the same room and must meet.
//
//   ./scripts/web-smoke.sh            # runs this after the plain page load
//   node crates/web/room-smoke.mjs http://127.0.0.1:8080/ target/room-smoke
//
// The room is `?board=tabs`: a BroadcastChannel rather than the public
// brokers, because a test should not depend on somebody else's server (and a
// sandbox may not reach one). Everything after the board is the real thing --
// the hello, the build and world check, the offer and the answer, a WebRTC
// data channel between the two pages, and GGRS running a match over it. Both
// tabs have to say which player they are, and they have to disagree: the tab
// that opened the room first is player one.
//
// Chromium hides local addresses behind generated `.local` names, which a
// container cannot resolve; the flag below turns that off so the two pages
// can reach each other on this machine.
import { createRequire } from 'node:module';
import { execSync } from 'node:child_process';

const root = execSync('npm root -g').toString().trim();
const { chromium } = createRequire(root + '/')('playwright');

const [url, prefix = 'target/room-smoke'] = process.argv.slice(2);
if (!url) {
  console.error('usage: node crates/web/room-smoke.mjs <url> [screenshot-prefix]');
  process.exit(2);
}

const browser = await chromium.launch({
  args: [
    '--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist',
    '--disable-features=WebRtcHideLocalIpsWithMdns',
  ],
});
// Small, because two games share one software GPU here: at the smoke test's
// full size each draws about two frames a second, and the meeting and the
// match only move when a frame does.
const context = await browser.newContext({ viewport: { width: 520, height: 420 } });
const room = 'smoke' + Math.random().toString(36).slice(2, 8);
// ROOM_VIA=broker=ws://127.0.0.1:9001 meets through a broker instead: the
// whole public path, against one running on this machine.
const via = process.env.ROOM_VIA || 'board=tabs';
const link = `${url}${url.includes('?') ? '&' : '?'}room=${room}&${via}`;
console.log(`room link: ${link}`);

const errors = [];
// Everything the two pages said, printed if they did not meet: the meeting
// logs each step, and a WebRTC failure is a warning rather than an error.
const said = [];
async function open(name) {
  const page = await context.newPage();
  page.on('console', (m) => {
    if (m.type() === 'error') errors.push(`${name} console: ${m.text()}`);
    if (m.type() === 'warning' || m.type() === 'log') said.push(`${name} ${m.type()}: ${m.text()}`);
  });
  page.on('pageerror', (e) => errors.push(`${name} page: ${e.message}`));
  page.on('requestfailed', (r) => errors.push(`${name} request: ${r.url()} ${r.failure()?.errorText}`));
  await page.goto(link, { waitUntil: 'load', timeout: 120_000 });
  return page;
}

const status = (page) => page.evaluate(() => document.getElementById('online-status')?.textContent ?? '');

// One, then the other once the first is up and waiting: the order is the
// thing being checked.
const first = await open('first');
const deadline = Date.now() + 150_000;
let alone = '';
while (Date.now() < deadline && !(alone = await status(first)).includes('Waiting')) {
  await first.waitForTimeout(1_000);
}
console.log(`first tab, alone: ${alone || '(nothing)'}`);
const second = await open('second');

let a = '', b = '';
while (Date.now() < deadline) {
  [a, b] = [await status(first), await status(second)];
  if (a.includes('Online') && b.includes('Online')) break;
  await first.waitForTimeout(1_000);
}
// A few seconds of match, so a desync or a disconnect has time to show.
await first.waitForTimeout(5_000);
[a, b] = [await status(first), await status(second)];

await first.screenshot({ path: `${prefix}-first.png` });
await second.screenshot({ path: `${prefix}-second.png` });
await browser.close();

console.log(`first tab:  ${a}`);
console.log(`second tab: ${b}`);
console.log(`screenshots: ${prefix}-first.png ${prefix}-second.png`);
if (!a.includes('player one')) errors.push(`the first tab should be player one; it says: ${a}`);
if (!b.includes('player two')) errors.push(`the second tab should be player two; it says: ${b}`);
if (errors.length) {
  console.error('what the pages said:');
  for (const line of said.slice(-30)) console.error(`  ${line}`);
  console.error(`${errors.length} problem(s):`);
  for (const e of errors.slice(0, 10)) console.error(`  ${e}`);
  process.exit(1);
}
console.log('two tabs met and are playing');
