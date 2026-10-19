// Parity check: native hash == WASM-in-Node hash == WASM-in-Chrome-from-file:// hash.
// Usage (from "Geometry Clone/"): node scripts/parity.mjs
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { createRequire } from 'node:module';
import { execSync } from 'node:child_process';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const TICKS = 100000;

const native = execFileSync('cargo', ['run', '-q', '--release', '-p', 'gd-cli', '--', 'parity', String(TICKS)],
  { cwd: root }).toString().trim();

const { instance } = await WebAssembly.instantiate(
  readFileSync(join(root, 'target/wasm32-unknown-unknown/release/gd_wasm.wasm')), {});
const w = instance.exports;
w.gd_reset(0, 0, 1); w.gd_run_parity(0, TICKS);
const node = (BigInt(w.gd_hash_hi(0) >>> 0) << 32n | BigInt(w.gd_hash_lo(0) >>> 0)).toString(16).padStart(16, '0');

let chrome = '(skipped: playwright not found)';
try {
  const req = createRequire(join(execSync('npm root -g').toString().trim(), 'x.js'));
  const { chromium } = req('playwright');
  const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH || undefined });
  const page = await browser.newPage();
  await page.goto(pathToFileURL(join(root, 'dist/geometry.html')).href);
  await page.waitForFunction(() => window.__GD_RESULT__, null, { timeout: 15000 });
  const r = await page.evaluate(() => window.__GD_RESULT__);
  chrome = r.error ? 'ERROR ' + r.error : r.hash;
  await browser.close();
} catch (e) { if (!/Cannot find (module|package)/.test(String(e))) chrome = 'ERROR ' + e.message; }

console.log({ native, node, chrome });
const ok = native === node && (chrome.startsWith('(skipped') || chrome === native);
console.log(ok ? 'PARITY OK' : 'PARITY FAIL');
process.exit(ok ? 0 : 1);
