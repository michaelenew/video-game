// The Esc menu makes a room, and another tab joins it from the menu.
//
//   ./scripts/web-smoke.sh            # runs this after room-smoke
//   node crates/web/menu-smoke.mjs http://127.0.0.1:8080/ target/menu-smoke
//
// No link is typed into an address bar here. The first tab presses Escape and
// clicks Create a room; the link it shows is read back from the address bar
// (where the game also puts it) and pasted into the second tab's Join box,
// which the second tab reaches the same way. Both tabs must then be in the
// match, the first as player one. The rooms are `?board=tabs`, for the same
// reason as room-smoke.mjs.
//
// The menu is drawn by egui into the canvas, so its buttons have no DOM: they
// are clicked by position. The positions are the menu's own layout (anchored
// at 18,150 in the canvas, `crates/game/src/menu.rs`); if that layout moves,
// so do these.
import { createRequire } from 'node:module';
import { execSync } from 'node:child_process';

const root = execSync('npm root -g').toString().trim();
const { chromium } = createRequire(root + '/')('playwright');

const [url, prefix = 'target/menu-smoke'] = process.argv.slice(2);
if (!url) {
  console.error('usage: node crates/web/menu-smoke.mjs <url> [screenshot-prefix]');
  process.exit(2);
}
const browser = await chromium.launch({
  args: [
    '--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist',
    '--disable-features=WebRtcHideLocalIpsWithMdns',
  ],
});
const context = await browser.newContext({ viewport: { width: 900, height: 700 } });
await context.grantPermissions(['clipboard-read', 'clipboard-write'], { origin: new URL(url).origin });
const errors = [];
const page = async (name) => {
  const p = await context.newPage();
  p.on('console', (m) => m.type() === 'error' && errors.push(`${name} console: ${m.text()}`));
  p.on('pageerror', (e) => errors.push(`${name} page: ${e.message}`));
  await p.goto(`${url}${url.includes('?') ? '&' : '?'}board=tabs`, { waitUntil: 'load', timeout: 120_000 });
  await p.waitForTimeout(25_000);
  return p;
};
const status = (p) => p.evaluate(() => document.getElementById('online-status')?.textContent ?? '');
// Where the canvas is, so a position in the menu is a position on the page.
const at = async (p, x, y) => {
  const box = await p.locator('canvas').boundingBox();
  return [box.x + x, box.y + y];
};
// A click with the pointer settled first: this page draws two frames a second
// in software, and egui has to see the pointer over a button before the press.
// Held well under egui's 0.8 s, past which a press is a drag, not a click.
const click = async (p, [x, y]) => {
  await p.mouse.move(x, y);
  await p.waitForTimeout(1_200);
  await p.mouse.down();
  await p.waitForTimeout(150);
  await p.mouse.up();
  await p.waitForTimeout(1_200);
};
const menu = async (p) => {
  await p.locator('canvas').focus();
  await p.keyboard.press('Escape');
  await p.waitForTimeout(3_000);
};

const first = await page('first');
await menu(first);
await first.screenshot({ path: `${prefix}-menu.png` });
await click(first, await at(first, 70, 228));
await first.waitForTimeout(4_000);
const link = await first.evaluate(() => location.href);
await first.screenshot({ path: `${prefix}-room.png` });
console.log(`first tab made: ${link}`);
// Copy link: the button under the link, and then the clipboard holds it.
await click(first, await at(first, 54, 318));
const copied = await first.evaluate(() => navigator.clipboard.readText()).catch((e) => `(unreadable: ${e})`);
console.log(`clipboard after Copy link: ${copied}`);
if (copied !== link) errors.push(`Copy link put ${JSON.stringify(copied)} on the clipboard, not the link`);
if (!link.includes('room=') || !link.includes('#key=')) errors.push(`no room in the address bar after Create: ${link}`);

const second = await page('second');
await menu(second);
await click(second, await at(second, 160, 304));
await second.keyboard.type(link, { delay: 5 });
await second.waitForTimeout(1_500);
await click(second, await at(second, 316, 304));

const deadline = Date.now() + 150_000;
let a = '', b = '';
while (Date.now() < deadline) {
  [a, b] = [await status(first), await status(second)];
  if (a.includes('Online') && b.includes('Online')) break;
  await first.waitForTimeout(1_000);
}
await second.screenshot({ path: `${prefix}-joined.png` });
await browser.close();
console.log(`first tab:  ${a}`);
console.log(`second tab: ${b}`);
console.log(`screenshots: ${prefix}-menu.png ${prefix}-room.png ${prefix}-joined.png`);
if (!a.includes('player one')) errors.push(`the tab that made the room should be player one; it says: ${a}`);
if (!b.includes('player two')) errors.push(`the tab that joined should be player two; it says: ${b}`);
if (errors.length) {
  console.error(`${errors.length} problem(s):`);
  for (const e of errors.slice(0, 10)) console.error(`  ${e}`);
  process.exit(1);
}
console.log('a room made in the menu was joined from the menu');
