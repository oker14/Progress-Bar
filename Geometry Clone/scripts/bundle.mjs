// Builds dist/geometry.html: one self-contained file (WASM inlined as base64).
// Usage: node scripts/bundle.mjs
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const wasm = readFileSync(join(root, 'target/wasm32-unknown-unknown/release/gd_wasm.wasm'));
const html = readFileSync(join(root, 'web/template.html'), 'utf8')
  .replace('__WASM_B64__', wasm.toString('base64'));
mkdirSync(join(root, 'dist'), { recursive: true });
writeFileSync(join(root, 'dist/geometry.html'), html);
console.log(`dist/geometry.html  ${(html.length / 1024).toFixed(1)} KiB (wasm ${wasm.length} bytes)`);
