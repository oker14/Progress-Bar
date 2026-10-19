# Step 2 — Project skeleton

## Layout
| Path | Role |
|---|---|
| `core/` (`gd-core`) | The physics. `#![no_std]`, zero dependencies, no I/O, no randomness. Everything else wraps this. |
| `wasm/` (`gd-wasm`) | Thin C-ABI exports (`gd_reset`, `gd_step`, `gd_run_parity`, `gd_hash_*`...). A fixed pool of 64 sims; no wasm-bindgen. |
| `cli/` (`gd-cli`, binary `gd`) | Native runner: `gd parity [ticks]`, `gd bench [ticks]`. Will grow into the training/bench harness. |
| `web/template.html` | Page template. `scripts/bundle.mjs` inlines the WASM as base64 -> `dist/geometry.html`. |
| `scripts/parity.mjs` | Proves native == WASM(Node) == Chrome opened from `file://`. |

## Determinism rules (enforced by review + parity test)
1. Core uses only `+ - * /` and comparisons on floats. No `sin/cos/pow/exp`/`sqrt` unless via an explicit, tested implementation.
2. No FMA, no fast-math (Rust doesn't contract by default; do not enable target features that change this).
3. Fixed tick = 1/240 s. Input is supplied by the caller as a pure function of the tick; the core never reads a clock or RNG.
4. Every field that influences the future must be included in `Sim::state_hash`.
5. Version differences live only in `version.rs` tables (`PhysicsVersion`). V21 returns `None` until filled in.

## Current state of the sim
`sim.rs` is a **scaffold**: speed + cube gravity/jump from spec sections 3-4, flat ground at y = 0, no objects. It exists to prove the pipeline. Step 3 replaces it with validated cube physics.

## Results (this machine, 2026-10)
- Native scaffold throughput: ~336 M ticks/s single thread (trivial physics; will drop as the sim grows).
- WASM size: 7 KB; `dist/geometry.html`: 11 KB.
- Parity hash after 100,000 ticks: `5284f536a4cc5b87` identical in native, WASM-in-Node and Chrome opened from `file://`. The `file://` risk flagged in the plan is **resolved**: a single HTML with inlined WASM works with no server.

## Known limitations / next
- Parity script only exercises the scaffold; it must be re-run (and golden hashes recorded) as each step lands.
- Hash is FNV-1a over exact f64 bits; if a future change is intentional, update the golden hash in the commit that changes it.
- Training-side concerns (batched envs, observations, Python bindings) are step 8, not here.
