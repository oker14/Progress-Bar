# Render and input model (browser build)

Physics is **fixed at 240 Hz in every mode**. Render rate, extrapolation and input timing never change what the simulation computes; the state hash is identical in all settings.

## Render FPS
- Rendering runs on `requestAnimationFrame` (the browser's display rate) with an optional **render FPS cap**: Display rate, 30, 60, 90, 120, 144, 165, 240, 360.
- Any cap from 1 to 100000 can be typed into the custom box; the dropdown has presets up to 1000 and "Unlimited". A browser still shows at most one frame per display refresh, so caps above it behave like "Unlimited". The native build has no such limit (vsync off).
- Above 240 fps there are fewer than one physics tick per frame; extrapolation is what makes every frame a distinct position (virtual-time tests at 480 and 1000 fps: 240 ticks/s, every frame a new, strictly increasing position). Without extrapolation those frames would repeat the last tick (an effective 240 fps).
- Ticks per frame adapt automatically (e.g. 8 at 30 fps, 4 at 60, ~1.7 at 144). Measured in headless Chrome: 240.0 Hz physics at 30 and 60 fps.
- A stall longer than 100 ms resyncs instead of fast-forwarding (no spiral of death).

## Frame extrapolation (visual only)
`Sim::extrapolate(ticks)` in the core predicts the cube position a fractional number of ticks after the last real tick (exact gravity model; clamped to the floor). The renderer draws that instead of the raw state.
- **Off** — draw the last simulated tick.
- **Smooth** (default) — extrapolate to the current time (at most one tick, ~4 ms): removes tick-boundary stutter at high refresh rates.
- **Predict** — extrapolate to the estimated moment the frame is displayed (~1 frame ahead): lowest perceived latency, but can briefly overshoot at a landing or a just-pressed jump.
Extrapolation never feeds back into the sim (tested: it leaves the state hash untouched, and in free flight it equals real stepping to ~1e-14 units). Once objects exist (step 4), the renderer must clamp predicted positions against solids.

## Click-between-frames (on by default)
- **On:** an input event is applied at its own timestamp. Ticks before the click run with the old input, ticks after with the new one. A click 7.3 ms into a frame lands on tick 26, the tick at/after 107.3 ms, regardless of frame rate.
- **Off (vanilla GD behaviour):** input is sampled once at frame start and every tick of that frame sees it, so a click is treated as happening up to a frame *earlier* than it really did, and timing depends on frame rate.
- Known limit: an event that is delivered after its frame already simulated past its timestamp (main-thread stall) is applied at the next tick instead of rewinding.

## How it is tested
All in the page's **Self-tests** panel, on virtual time so they are deterministic: extrapolation equals real stepping; CBF on lands on the click's tick; CBF off applies from the frame's first tick; 240 ticks/s at 30/60/144/240 fps.
Not testable in headless Chrome: real display refresh above 60 Hz, and real touch/keyboard hardware latency.
