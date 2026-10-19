# Step 3 — Cube physics (flat ground)

## What changed vs the step-2 scaffold
The research pass found the one thing the scaffold got wrong: **vertical motion runs on a 0.9 time scale**.
Position advances by `yVel * 0.9 * dt` and gravity pulls by `g * 0.9 * dt` (`dt = 0.25` 60 Hz-frames per 240 Hz tick).
The scaffold used 1.0, which still gave the right apex (2.13 blocks) but the wrong duration (93 ticks instead of 103).
Horizontal speed was already right (`player_speed * speed_mult` already includes the factor).

## Implemented (`core/src/sim.rs`)
Per 240 Hz tick, in this order:
1. **Move** with last tick's velocity: `x += speed * 0.25`, `y += yVel * 0.9 * 0.25`
2. **Floor:** if `y <= 0` and not rising -> snap to 0, grounded
3. **Input:** grounded + held -> `yVel = yStart` (jump); grounded + not held -> `yVel = 0`
4. **Gravity** if airborne: `yVel -= g * 0.9 * 0.25`, clamped to >= -15

Consequences that fall out of this order (all tested): a jump decided on tick N first moves the cube on tick N+1;
holding the button re-jumps on the landing tick with no ground contact in between; resting on the floor is exactly still.

## Results
| Speed | apex (blk) | air ticks | distance (blk) | reference |
|---|---|---|---|---|
| 0.5x | 1.960 | 100 | 3.49 | 1.955 / 100 / 3.49 |
| 1x | 2.132 | 103 | 4.46 | 2.128 / 103 / 4.457 |
| 2x | 2.228 | 106 | 5.70 | 2.232 / 106 / 5.70 |
| 3x | 2.145 | 103 | 6.69 | 2.147 / 103 / 6.69 |
| 4x | 2.145 | 103 | 8.24 | 2.147 / 103 / 8.24 |

Ticks match **exactly** for all five speeds. 1x apex is 2.1323 blocks vs the published 2.1333.

## How honest is "matches the reference"?
- The reference table comes from a *different implementation* of the same published constants (see `docs/research/step3-research.md`), not from recordings of the real game. Agreement proves our integrator matches the model, and the model reproduces the two published heights (cube 2.1333, mini 1.3583) to < 0.01 blocks. It does **not** prove tick-exactness against the real game.
- **Open: one-tick phase.** The most careful public sim moves *then* accelerates (what we do). A 2.1-era reimplementation accelerates *then* moves. The real 2.2 `PlayerObject::update` is not public. Settle it with a recording from the real game (GDR2 + xdBot "frame fixes" give per-frame positions, or log player position per physics step with a mod).
- **Not modelled:** the sim we compared to rounds velocity to 3 decimals (a float-emulation quirk): apex 2.128 vs our 2.132. Could become an optional "faithful" mode if recordings show it matters.

## Tooling added
- `scripts/golden.json` holds the expected parity hash. `node scripts/parity.mjs` fails if native/WASM/Chrome or the golden file disagree. After an *intentional* physics change run `node scripts/parity.mjs --update`, then `node scripts/bundle.mjs`, and commit both.
- The browser build self-tests include the full jump table above, run on the WASM build.

## Not done yet (by design)
Objects and collisions (step 4), pads/orbs/portals (step 5), other vehicles (step 6), mini cube, gravity flip. The hitbox numbers gathered for step 4 are in `docs/research/step3-research.md` section 2.
