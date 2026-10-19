# Geometry Clone — instructions for AI assistants

Start with **HANDOFF.md** (project goal, where files live, decisions, status, and the next step to work on).

Non-negotiables:
- Physics is fixed at **240 Hz** and **deterministic**. Native, WASM(Node) and WASM(Chrome) must give identical hashes: run `node scripts/parity.mjs` (prints PARITY OK) after any core change; `--update` only for intentional physics changes.
- The core (`core/`) is `#![no_std]`, dependency-free, float math limited to `+ - * /` and comparisons, no allocation in the tick path.
- Do **not** copy other people's code. Researching numbers and behaviour is fine; write our own implementation.
- Every step must be testable and visibly demonstrable in `dist/geometry.html` (single file, WASM inlined, works from `file://`). Rebuild it with `node scripts/bundle.mjs` and commit it.
- Chromebook target (Intel N4500, 4 GB): no per-frame allocation or DOM churn in the browser path.
- Be honest about what is verified vs inferred; tag facts [V]/[R]/[?] in docs.
