// A page opens a sealed room; the desktop game joins it with --join.
//
//   ./scripts/room-desktop.sh          # sets up the broker, the page and a display
//
// The real thing end to end, on one machine: the page in headless Chromium,
// the desktop build under Xvfb, the two meeting through a real MQTT broker
// (mosquitto, on this machine, because a test should not depend on a public
// one), sealing their notes with the link's key, and playing over WebRTC --
// the page's own on one end, str0m on the other. The page opens the room
// first, so it must be player one and the desktop player two, and neither may
// report a desync.
import { createRequire } from 'node:module';
import { execSync, spawn } from 'node:child_process';
import { createWriteStream } from 'node:fs';

const root = execSync('npm root -g').toString().trim();
const { chromium } = createRequire(root + '/')('playwright');

const [link, prefix = 'target/room-desktop', game = './target/debug/game'] = process.argv.slice(2);
if (!link) {
  console.error('usage: node crates/web/room-desktop.mjs <link> [screenshot-prefix] [game]');
  process.exit(2);
}

const browser = await chromium.launch({
  args: [
    '--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist',
    // The page's local address, unhidden: see room-smoke.mjs.
    '--disable-features=WebRtcHideLocalIpsWithMdns',
  ],
});
const page = await browser.newPage({ viewport: { width: 520, height: 420 } });
const errors = [];
const said = [];
page.on('console', (m) => {
  if (m.type() === 'error') errors.push(`page console: ${m.text()}`);
  else said.push(`page ${m.type()}: ${m.text()}`);
});
page.on('pageerror', (e) => errors.push(`page: ${e.message}`));
console.log(`link: ${link}`);
await page.goto(link, { waitUntil: 'load', timeout: 120_000 });

const status = () => page.evaluate(() => document.getElementById('online-status')?.textContent ?? '');
const deadline = Date.now() + 180_000;
let alone = '';
while (Date.now() < deadline && !(alone = await status()).includes('Waiting')) {
  await page.waitForTimeout(1_000);
}
console.log(`page, alone: ${alone || '(nothing)'}`);

// The desktop, given the link exactly as a friend would paste it.
const log = [];
const out = createWriteStream(`${prefix}-game.log`);
const desktop = spawn(game, ['--join', link], { stdio: ['ignore', 'pipe', 'pipe'] });
for (const stream of [desktop.stdout, desktop.stderr]) {
  stream.on('data', (chunk) => {
    out.write(chunk);
    for (const line of chunk.toString().split('\n')) if (line.trim()) log.push(line.trim());
  });
}
const desktopSays = (text) => log.some((l) => l.includes(text));

let page_ = '';
while (Date.now() < deadline) {
  page_ = await status();
  if (page_.includes('Online') && desktopSays('online: you are player')) break;
  if (desktop.exitCode !== null) break;
  await page.waitForTimeout(1_000);
}
// Some seconds of match, so a desync or a disconnect has time to show.
await page.waitForTimeout(10_000);
page_ = await status();

await page.screenshot({ path: `${prefix}-page.png` });
try {
  execSync(`import -window root ${prefix}-desktop.png`);
} catch (e) {
  errors.push(`could not grab the desktop's screen: ${e.message}`);
}
desktop.kill();
await browser.close();

const seat = log.find((l) => l.includes('online: you are player')) ?? '(never seated)';
console.log(`page:    ${page_}`);
console.log(`desktop: ${seat}`);
console.log(`screenshots: ${prefix}-page.png ${prefix}-desktop.png; desktop log: ${prefix}-game.log`);
if (!page_.includes('player one')) errors.push(`the page opened the room, so it should be player one; it says: ${page_}`);
if (!seat.includes('player 2')) errors.push(`the desktop joined second, so it should be player 2; it says: ${seat}`);
if (page_.includes('DESYNC') || desktopSays('DESYNC')) errors.push('the two games disagreed: DESYNC');
if (page_.includes('disconnected') || desktopSays('peer disconnected')) errors.push('the connection dropped during the match');
if (errors.length) {
  console.error('what the page said:');
  for (const line of said.slice(-15)) console.error(`  ${line}`);
  console.error('what the desktop said:');
  for (const line of log.slice(-15)) console.error(`  ${line}`);
  console.error(`${errors.length} problem(s):`);
  for (const e of errors.slice(0, 10)) console.error(`  ${e}`);
  process.exit(1);
}
console.log('a desktop joined a page\'s sealed room and they are playing');
