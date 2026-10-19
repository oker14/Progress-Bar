# Geometry Clone

Deterministic Geometry Dash physics (2.2 first, 2.1 later), built to be:
- **Accurate** — fixed 240 Hz tick, version-tagged constants (see `docs/`)
- **Fast** — Rust `no_std` core, no allocation in the tick path, built for RL training
- **Portable** — the same core runs natively (home PC) and as WASM in a single double-clickable HTML file (Chromebook)

Progress is tracked per step; see `docs/`.

| Doc | Content |
|---|---|
| `docs/01-physics-spec.md` | Physics spec with confidence tags and open questions |
| `docs/02-architecture.md` | Step 2: layout, rules, how to build and test |
| `docs/03-cube-physics.md` | Step 3: cube physics on flat ground, results, open questions |
| `docs/render-input.md` | Render FPS cap, frame extrapolation, click-between-frames |
| `docs/research/step3-research.md` | Research notes: integration order, hitboxes, reference tables, replay formats |

## Quick start (home PC)
```
cargo test -p gd-core                 # unit tests
cargo run --release -p gd-cli -- bench 200000000
rustup target add wasm32-unknown-unknown
cargo build --release -p gd-wasm --target wasm32-unknown-unknown
node scripts/bundle.mjs               # -> dist/geometry.html (copy this ONE file to the Chromebook)
node scripts/parity.mjs               # native == WASM(Node) == Chrome(file://)
```
