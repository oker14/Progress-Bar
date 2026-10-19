# Handoff — Geometry Clone

## Status (v0)
Working: cube mode only, auto-run at 10.4 tiles/s, jump, spikes, solid blocks (land on top, die on sides/underside),
one 120-tile level, human play in browser, headless env wrapper, passing tests.
Not done: other game modes (ship, ball, wave...), orbs/pads/portals, practice mode, rotation/animation, sound, level editor.

## Key decisions
- **Simulation is fully separate from rendering/input.** `physics.js` and `env.js` run in plain Node. Never import DOM code into them.
- **Deterministic**: fixed `DT = 1/60`, no `Math.random`, no wall-clock. Same input sequence => same result (tested). This matters for AI training and replays.
- **Units = tiles**, y up, floor top at y = 0. Player is a 1x1 cube.
- **Collision**: vertical axis first (landing snaps to block tops), then horizontal (any side overlap kills). Spike hitbox is smaller than its sprite (0.3 x 0.5), as in GD.
- **Tuning values** (`CONST`): SPEED 10.4, GRAVITY 137, JUMP_VEL 25.2 -> apex ~2.1 tiles, ~21 ticks airtime (~3.6 tiles horizontal). JUMP_VEL is slightly higher than the analytic value to compensate for the semi-implicit Euler step; if you change the integrator, retune and re-run the test.
- Holding jump while grounded re-jumps immediately (GD behaviour).

## AI interface
```js
const env = new GeometryEnv(level, { frameSkip: 1, maxTicks: 3600 });
let obs = env.reset();
const { obs, reward, done, info } = env.step(action); // action 0 = release, 1 = hold
```
- obs (18 floats): `[y/4, vy/JUMP_VEL, onGround, ...3 upcoming objects x (dx/20, y/4, w/4, h/4, isSpike)]`, padded when fewer than 3 remain.
- reward: distance advanced, +100 on win, -10 on death. `info`: `{x, progress, dead, won}`.
- Input only matters when grounded, so an agent really makes one decision per grounded tick. `frameSkip` can reduce decision count.

## Suggested next steps
1. Physics feel: tune constants against real GD footage/numbers; add cube rotation (visual only).
2. More objects: yellow pads/orbs, portals (gravity flip, ship), moving speed portals. Extend `step()` and the obs vector, keep determinism.
3. More levels, then a simple level format (JSON) and editor.
4. AI: start with a tabular/neuroevolution agent in JS (no deps), or export the env to Python (reimplement `physics.js` or run via Node child process / WebSocket) for stable-baselines3. `test/sim.test.js` already has a DFS solver usable as an oracle/expert for imitation learning.
5. Replay system: record `(tick, held)` inputs; determinism guarantees exact playback.

## Gotchas
- Level 1 was originally unwinnable (triple spike was too wide for a single jump); the solver test now guards against unbeatable levels — run it after editing levels.
- The repo root is an unrelated game (Progress Bar). Don't touch it from here.
