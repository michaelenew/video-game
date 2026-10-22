import { createRequire } from 'node:module';
import { execSync } from 'node:child_process';
const root = execSync('npm root -g').toString().trim();
const { chromium } = createRequire(root + '/')('playwright');
const [url, mode] = process.argv.slice(2);
const browser = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist'] });
const context = await browser.newContext({ viewport: { width: 900, height: 700 } });
const open = async (q) => { const p = await context.newPage(); p.on('console', m => m.type()==='error' && console.log('console', m.text())); await p.goto(url + q, { waitUntil: 'load', timeout: 120000 }); await p.waitForTimeout(25000); return p; };
const a = await open('?board=tabs');
if (mode === 'room') { await a.locator('canvas').focus(); await a.keyboard.press('Escape'); await a.waitForTimeout(3000); const b = await a.locator('canvas').boundingBox(); await a.mouse.move(b.x+70,b.y+78); await a.waitForTimeout(1200); await a.mouse.down(); await a.waitForTimeout(150); await a.mouse.up(); await a.waitForTimeout(4000); console.log('a at', await a.evaluate(() => location.href)); }
const b = await open('?board=tabs');
const t = Date.now();
try { await b.screenshot({ path: 'target/zz-two-' + mode + '.png', timeout: 60000 }); console.log(mode, 'second tab answers', Date.now()-t, 'ms'); } catch (e) { console.log(mode, 'second tab HUNG'); }
await browser.close();
