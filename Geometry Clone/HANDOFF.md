# HANDOFF — Geometry Clone

**Read this first.** It is written for an AI coding assistant (and the human) picking this project up in a new session or a new repository. It says where everything is, what is done, what is decided, and exactly what to do next.

---

## 1. What this project is
A **Geometry Dash 2.2-accurate physics clone**, built to (a) be tick-accurate, (b) run extremely fast, and (c) later train a **reinforcement-learning agent that learns the physics from vision**, not from hand-computed formulas. Visuals come last.

**Two deliverables from ONE physics core:**
| | Home build ("program") | School build (Chromebook) |
|---|---|---|
| Owner's machine | Windows 11, Ryzen 5 3600, 32 GB DDR4, **RX 5700 XT** | Chromebook, **Intel N4500, 4 GB RAM** |
| Purpose | Exact physics, huge simulation throughput, RL training | Play, watch the AI, test physics |
| Form | Native Rust binary + Python bindings (later) | **One double-clickable `geometry.html`** (WASM inlined as base64), works offline from `file://`, no server, no install |
| GPU note | The 5700 XT is RDNA1: **no official ROCm/PyTorch-GPU on Windows**. Plan on **CPU training** (small CNN, low-res tile-grid observation) or Rust-native training (`burn`/`candle`). Decide at step 9. | n/a |

Physics versions: **2.2 first.** 2.1 must be selectable per level later; every version-dependent constant already goes through `PhysicsVersion` in `core/src/version.rs` (V21 currently returns `None`).

## 2. Where this code lives
- **Current location:** GitHub repo `oker14/Progress-Bar`, branch **`claude/nice-albattani-fa532b`**, folder **`Geometry Clone/`**.
- The repo root contains an **unrelated old game ("Progress Bar": `index.html`, `js/`, `style.css`). Do not touch it.** Everything for this project is inside `Geometry Clone/`.
- The owner intends to **move this folder into its own new GitHub repository.** To keep history, from a clone of `oker14/Progress-Bar` run:
  ```bash
  git checkout claude/nice-albattani-fa532b
  git subtree split -P "Geometry Clone" -b geometry-clone-only
  mkdir ../geometry-clone && cd ../geometry-clone && git init -b main
  git pull ../Progress-Bar geometry-clone-only
  git remote add origin https://github.com/<owner>/<new-repo>.git && git push -u origin main
  ```
  (No history wanted? Just copy the folder contents to the new repo root.) After the move, **this folder IS the repo root**: all paths below are relative to it, and `CLAUDE.md` is picked up automatically by Claude Code.
- Do not assume the commit/branch above exists in the new repo. Check `git log` and `docs/` to see how far things have got.

## 3. Hard rules (from the owner)
1. **Step by step, every step is testable and shows visible progress.** After each step: the browser page must demonstrate it, with self-tests; give the owner the built `dist/geometry.html`. Never leave a step as "plumbing only".
2. **Do not copy or reuse other people's code.** Researching numbers/behaviour (jump values, tick rates, hitboxes) is fine and expected. Write our own implementation. Public GD reimplementations seen so far: `camila314/gdp` (decompiled 2.2 snippets, behaviour reference only), `camila314/pathfinder` gd-sim (constants; not licensed for redistribution), `Open-GD/OpenGD` (GPL-3.0, unmaintained), `opstic/gdclone`, `prevter/OpenHack`, `geode-sdk/bindings`. Keep any clone of these **outside the repo**.
3. **Physics is always fixed at 240 Hz** in every mode. Render FPS is independent; extrapolation is visual only and never feeds back into the sim.
4. **Determinism is sacred:** native, WASM-in-Node and WASM-in-Chrome must produce identical state hashes. See section 6.
5. **Chromebook performance matters** (N4500/4 GB): no per-frame allocation in the browser hot path, no DOM churn, no heavy dependencies. Keep `dist/geometry.html` small (currently ~33 KB).
6. Be honest about what is verified vs inferred. Docs use confidence tags: **[V]** two sources agree, **[R]** one source, **[?]** unknown.
7. Keep the commit trail clean and push to the designated branch; do not open pull requests unless asked.

## 4. Repository map
```
Cargo.toml            workspace (core, wasm, cli); release profile = lto, opt 3, panic=abort
core/   (gd-core)     THE PHYSICS. #![no_std], zero dependencies, no I/O, no RNG
  src/sim.rs          Sim: step(held), extrapolate(ticks), state_hash(), jump_profile(), parity_input()
  src/version.rs      PhysicsVersion {V22, V21(stub)}, Speed, per-speed constants table
  src/hash.rs         FNV-1a 64 over exact bits
wasm/   (gd-wasm)     C-ABI exports for the browser (no wasm-bindgen). Pool of 64 sims.
cli/    (gd-cli)      native binary `gd`: `gd parity [ticks]`, `gd bench [ticks]`
web/template.html     the whole browser app (HTML+CSS+JS). __WASM_B64__/__GOLDEN_HASH__/__PARITY_TICKS__ are substituted at bundle time
scripts/bundle.mjs    builds dist/geometry.html (single file, WASM inlined)
scripts/parity.mjs    native == WASM(Node) == Chrome(file://) == golden.json   (+ `--update`)
scripts/golden.json   expected parity hash. Update ONLY on intentional physics changes.
dist/geometry.html    committed built artifact (so it can be downloaded from GitHub for the Chromebook)
docs/01-physics-spec.md        step 1 spec (confidence-tagged; has a correction banner)
docs/02-architecture.md        step 2 layout + determinism rules
docs/03-cube-physics.md        step 3 results, integration order, open question
docs/render-input.md           render FPS cap, extrapolation, click-between-frames
docs/research/step3-research.md  hitboxes, speed/jump tables, replay formats, clone survey
```

## 5. Progress against the 10-step plan
| # | Step | Status |
|---|---|---|
| 1 | Spec and reference data | **Done (draft).** `docs/01-physics-spec.md` + `docs/research/step3-research.md`. No tick-level ground truth exists publicly. |
| 2 | Project skeleton (core/wasm/cli, single-file bundle, parity) | **Done.** `file://` single-file WASM verified in Chrome. |
| 3 | Cube physics, all 5 speeds | **Done.** Ticks match the reference model exactly; see caveats in section 8. |
| — | Browser extras (not in original plan) | **Done:** Chromebook-tuned renderer, timestamped input, **click-between-frames (default on)**, **frame extrapolation (Off/Smooth)**, render FPS cap 1–100000/unlimited, perf HUD, low-quality mode. |
| 4 | Level format + collisions (blocks, spikes, saws, slopes, spatial grid) | **NEXT** |
| 5 | Pads, orbs, portals (gravity, size, speed, mode), orb buffering | todo |
| 6 | Other modes: ship, ball, UFO, wave, robot, spider (+swing) | todo |
| 7 | Validation harness: replay comparison, fuzzing, determinism, recorded ground truth | todo |
| 8 | Env API + vision observations + parallel batched envs | todo |
| 9 | RL training (PPO + small CNN on vision), curriculum, held-out levels | todo |
| 10 | Chromebook bundle with trained network, human + watch-the-AI modes, then visuals | todo |
2.1 physics tables: fill in late (design already supports it).

## 6. How to build, test, verify
Prereqs (home PC): Rust (`rustup`), `rustup target add wasm32-unknown-unknown`, Node 18+. Playwright + Chromium only for the Chrome part of the parity check (optional; the script skips it if missing).
```bash
cargo test -p gd-core                                   # 10 unit tests
cargo run --release -p gd-cli -- bench 200000000        # raw throughput
cargo build --release -p gd-wasm --target wasm32-unknown-unknown
node scripts/bundle.mjs                                 # -> dist/geometry.html
node scripts/parity.mjs                                 # must print PARITY OK
node scripts/parity.mjs --update                        # ONLY after an intentional physics change; then re-run bundle.mjs and commit golden.json + dist
```
In the page, click **Self-tests** (they also run on load). Current page tests: parity vs native, 1x speed, jump table for 5 speeds, published jump height, determinism, 2.1-unavailable, extrapolation exactness, click-between-frames on/off, 240 ticks/s at 30/60/144/240/480/1000 fps.
**Definition of done for any change:** `cargo test` green, `parity.mjs` prints PARITY OK, page self-tests all PASS with no console errors, `dist/geometry.html` rebuilt and committed, docs updated.

## 7. Key technical decisions (and why)
- **Rust core, no dependencies, `#![no_std]`** → compiles to a 7 KB WASM and to native with identical behaviour. Floats: only `+ - * /` and comparisons in the core (IEEE-exact on both targets). **No sin/cos/pow/exp** unless implemented by hand and tested. No FMA/fast-math.
- **Units:** 1 block = 30 units. `y` is the cube's bottom edge, floor top = 0 (the real game offsets by 90/105; translate at the edges if ever needed).
- **Tick model (step 3):** one tick = 1/240 s = 0.25 "60 Hz frames" (`TICK_FRAMES`). x moves `player_speed*speed_mult*0.25` units/tick. **Vertical motion uses an extra 0.9 time scale**: `y += yVel*0.9*0.25`, gravity `yVel -= g*0.9*0.25`, clamp `yVel >= -15`. Order per tick: **move with last tick's velocity → floor/landing → input/jump → gravity.** A jump chosen on tick N first moves the cube on tick N+1. Holding re-jumps on the landing tick with no ground contact.
- **Speed table (2.2, from `core/src/version.rs`):** 0.5x ps .7 sm 5.98 yStart 10.62 g .940199 · 1x ps .9 sm 5.77 yStart 11.18 g .958199 · 2x ps 1.1 sm 5.87 y 11.42 g .957199 · 3x/4x ps 1.3/1.6 sm 6.0 y 11.23 g .961199. Blocks/s: 8.372, 10.386, 12.914, 15.6, 19.2. 1x jump: apex 2.132 blocks, 103 ticks, 4.46 blocks long.
- **Input model:** physics ticks are driven by wall time AND input timestamps (`Pipeline` class in `web/template.html`). With click-between-frames ON an input lands on the tick at/after its own timestamp; OFF = vanilla (sampled at frame start). Known limit: an event delivered after its frame already simulated past its timestamp is applied at the next tick (no rewind).
- **Extrapolation:** `Sim::extrapolate(ticks)` in the core (exact gravity model, floor-clamped, pure). Modes Off/**Smooth**(default). A "Predict to display time" mode was **removed on purpose** (can draw frames that never happen).
- **Browser single file:** WASM inlined as base64 so Chrome's `file://` restrictions don't matter. Don't introduce `fetch`/ES-module loading of sibling files.
- **Render:** opaque canvas (`alpha:false, desynchronized:true`), cached static background, one batched grid path, ring-buffer trail, stats text updated ~8 Hz only when changed, fixed-height panel (no layout shifts). Measured in headless Chrome with 6x CPU throttle: 60 fps, ~0.3 ms draw. Real-hardware numbers are shown by the in-page perf HUD: **ask the owner to report them from the N4500.**

## 8. Known gaps and open questions (be honest about these)
1. **No public tick-by-tick ground truth.** The "reference" jump table in the tests comes from a *different implementation of the same published constants*, not recordings of the real game. It proves we match the model, not the game, to the tick. Published cross-checks: cube jump 2.1333 blocks, mini 1.3583. To settle it, record the real game: GDR2 macros + xdBot "frame fixes" (per-frame positions) or a Geode mod logging `m_position` per physics step (needs a 2.2 install — the owner may have one).
2. **One-tick phase uncertainty:** we move-then-accelerate (matches the most careful public sim). A 2.1-era reimplementation accelerates-then-moves. The real 2.2 `PlayerObject::update` / `checkCollisions` are not public.
3. **Velocity rounding quirk** (the reference sim rounds yVel to 3 decimals; apex 2.128 vs ours 2.132). Not modelled; consider an optional "faithful" mode if recordings show it matters.
4. **Hitboxes are 2.1-era** (from extracted object data); no 2.2 table found. Player hitbox: outer 30x30 (mini 18x18); inner box 7.5x7.5 (0.25×) for normal, 0.4× for mini in 2.2 per OpenHack (an older sim used 9x9 — conflict noted). Spike 6x12, ground spike 9x10.8, small spike 6x5.6, saws are circles r 32.3/21.6/11.78, orb 36x36, pads 25x4 (yellow)/25x5 (pink)/25x6 (blue)/29x7 (red), portals 25x75 (gravity), 34x86 (vehicle), 31x90 (size), speed portals 33–69 x 44–56. Full table with roles in `docs/research/step3-research.md` §2. Verify against a hitbox-view on 2.2 when possible.
5. **Block collision rule (from research, [R]):** touching a block with the *inner* hitbox kills; otherwise if the player's bottom is within **10 units** of the block top and `vy <= 0` → snap on top; a side hit that misses the inner box does nothing; hazards are tested after blocks in the same tick. Cube-only stair "snap-up" X nudges exist (`checkSnapJumpToObject` behaviour; offsets per speed in `docs/01-physics-spec.md`/research). Orbs: one-frame coyote on the hitbox, need a buffered click; pads fire on touch.
6. **2.1 vs 2.2 differences are not catalogued.** 2.2 runs 240 Hz sub-steps; 2.1 ran per frame with delta scaling. Needs sources before the V21 tables can be filled.
7. The GD wiki (`geometrydash.wiki.gg`) was unreachable from the cloud sandbox (Cloudflare). **On the owner's local machine it should work in a browser — use it** for hitbox/orb/pad pages and cross-checks (2.2 "Hitbox", "Jump pad", "Orbs", "Portals", "Speed" pages).

## 9. What to work on next — STEP 4: level format and collisions
Goal: blocks, spikes, saws and the death/landing rules, **playable and visible in the browser** with a hand-built test level and tests.
1. **Level data format:** compact, deterministic, version-tagged (`PhysicsVersion` in the header). Object types: block (30x30, solid), spike (6x12 kill box, orientations), small/ground spike variants, saw (circle), half slab, and a placeholder for slopes. Static array/arena, **no allocation in the tick path** (training needs millions of steps/s).
2. **Spatial lookup:** uniform grid or sorted-by-x with a moving window; only objects near the player are tested.
3. **Collision rules in `Sim::step`** following section 8 item 5: solid landing with the 10-unit snap, inner-hitbox death, hazard test last, floor/ceiling. Add `dead`/`won` state, a level end x, and **include every new field in `state_hash`**.
4. **Core API** for later steps: `Sim::new(version, speed, &level)`, `reset()`, cheap `Copy`/`clone` state for batching. Keep `extrapolate` correct near solids (clamp predicted position against landings; never draw through a block).
5. **Browser:** draw objects, death flash, restart on press, a hand-made level covering: flat run with spikes, single block step, 2-block stair, gap-less platforms, a saw. Show hitboxes via a toggle (very useful for validation). Extend self-tests (e.g. "dies on first spike if no input", "clears spike with correct jump", "lands on block top", "dies hitting block side with inner box", **a solver (DFS over jump/no-jump decisions, branching only on grounded ticks) proving the level is beatable**).
6. Update `parity_input`/golden hash if needed (intentional change → `parity.mjs --update`), docs `docs/04-collisions.md`, rebuild `dist/geometry.html`, and hand the file to the owner.
Then continue with step 5 (pads/orbs/portals) in the same style.

## 10. Environment notes (cloud sandbox this was built in; may differ locally)
- Network was restricted by policy; GitHub and crates.io worked, many other hosts did not. The WebFetch tool had broken DNS; `curl` through the shell worked for allowed hosts.
- Playwright Chromium lives at `/opt/pw-browsers/chromium` in that sandbox. `scripts/parity.mjs` finds Playwright via the *global* npm root and accepts `CHROMIUM_PATH`. On the owner's PC install Playwright locally (`npm i -D playwright`) or let the script skip the Chrome leg.
- Headless Chrome there is capped near 60 fps: real >60 Hz behaviour and real input latency have **not** been measured. Ask the owner to try `dist/geometry.html` on the Chromebook and report the HUD numbers (render fps, draw ms, input→sim ms, worst frame).
- The prototype from the first attempt was deleted on request (it is in git history before commit `9e5c30d`); do not resurrect it.

## 11. Suggested first actions for the next session
1. `git log --oneline | head`, read `docs/03-cube-physics.md` and this file's sections 5–9.
2. Run the verification commands in section 6 and confirm everything is green before changing anything.
3. Start step 4 as described in section 9, keep each commit small and testable, and send the owner the rebuilt `dist/geometry.html` when it demonstrates something new.
