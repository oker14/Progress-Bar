# Geometry Clone

A Geometry Dash-style cube runner built as a **test bed for physics tuning and, later, AI agents**.
Plain JavaScript, no build step, no dependencies. Separate from the "Progress Bar" game in the repo root.

## Run
- Play: open `index.html` in a browser. Space / Up arrow / click / tap to jump (hold to keep jumping).
- Tests: `node "Geometry Clone/test/sim.test.js"` from the repo root (needs Node only).

## Layout
| File | Role |
|---|---|
| `src/physics.js` | Deterministic fixed-step simulation (60 Hz). All constants live in `CONST`. No DOM. |
| `src/level.js` | Level data (`spike`, `block`). Add levels here. |
| `src/env.js` | Gym-style `GeometryEnv` (`reset()`, `step(action)`) — the AI interface. |
| `src/render.js` | Canvas drawing. Read-only view of the state. |
| `src/main.js` | Input + game loop for humans. |
| `test/sim.test.js` | Jump height, determinism, and a DFS solver proving Level 1 is beatable. |

See `HANDOFF.md` for status, design decisions and the roadmap.
