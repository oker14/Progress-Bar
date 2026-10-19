# Step 1 — Physics Spec (GD 2.2 first, versioned for 2.1 later)

Status: **DRAFT v0.1**. Every item carries a confidence tag:
- **[V]** verified: read from reconstructed 2.2 game code *and* cross-checked against a known number
- **[R]** read from reconstructed code, not yet cross-checked
- **[?]** unknown / not found yet — must be researched or measured before implementing

## 0. Sources and rules of use

| Source | What it gave | Notes |
|---|---|---|
| `camila314/gdp` (GitHub, public) | Reconstructed 2.2 `GJBaseGameLayer::update`, `PlayerObject::updateJump`, `updateTimeMod`, `boostPlayer`, `checkSnapJumpToObject`, `collidedWithObjectInternal`, `collidedWithSlopeInternal` | Community reverse engineering of the game binary. Used **only as a behavioural reference**. |
| GD wiki (via web search snippets) | Cube jump height 2.1333 blocks | Cross-check value. The wiki host itself is blocked in this environment (see section 8). |

**Rule: we do not copy that code.** We read it to learn *behaviour*, write the behaviour down here in our own words, and implement independently. The reference clone lives outside this repo (`/home/user/camila314/gdp`) and is never committed.

## 1. Units and conventions
- 1 block = **30 units**. Positions are in units. [R]
- Velocities in the reference are expressed **per 1/60 s "frame"**, not per second. Convert: `units/s = v * 60`. [V]
- Gravity-flipped play is the same math multiplied by `flipMod = -1`. [R]

## 2. The tick loop (the most important part for accuracy)
Per rendered frame with delta time `dt` (seconds): [R]
1. `stepCount = max(1, floor(dt * 240 / min(timewarp, 1)))`
2. Each step: `physicsDelta = dt / stepCount`, `physicsSecond = dt * 60 / stepCount`
3. A fixed 240 Hz tick therefore has `physicsSecond = 0.25`. **Our core always runs fixed 240 Hz steps** (stepCount = 1); the browser just runs N ticks per frame.
4. Per step order (relevant parts): process queued input/commands -> move-trigger actions -> `player.update(physicsSecond)` -> `checkCollisions(player, physicsSecond)` -> `updateSpecial` -> end/win check -> rotation update.
5. `updateJump(dt)` (all vertical velocity logic) runs inside `player.update`, **before** position integration.

Open: the exact body of `PlayerObject::update` (position integration, horizontal movement) is **not** in the reference. [?] Must be inferred from tests in step 3/7.

## 3. Speed table (cube; `updateTimeMod`) [R]
`m_playerSpeed` is the speed portal enum, not a plain multiplier.

| Portal | playerSpeed | speedMultiplier | yStart (jump vel) | gravity |
|---|---|---|---|---|
| 0.5x | 0.7 | 5.98 | 10.62 | 0.940199 |
| 1x | 0.9 | 5.77 | 11.18 | 0.958199 |
| 2x | 1.1 | 5.87 | 11.42 | 0.957199 |
| 3x | 1.3 | 6.00 | 11.23 | 0.961199 |
| 4x | 1.6 | 6.00 | 11.23 | 0.961199 |

- Horizontal speed = `playerSpeed * speedMultiplier` units per 60 Hz frame. At 1x: `0.9 * 5.77 = 5.193` -> **10.386 blocks/s**, matching the known 10.3854 value. [V]
- Ball, ship-family and spider use gravity constant **0.9582** regardless of speed. [R]

## 4. Cube jump and gravity [R], numerics [V]
- Jump (grounded + jump buffered): `yVelocity = flipMod * yStart * sizeMod` where `sizeMod = 1.0` (mini: 0.8). Robot uses `yStart * 0.5`.
- Airborne gravity: `yVelocity -= gravity * gravityMod * physicsSecond * flipMod * modeFactor`, `modeFactor = 1.0` cube, `0.6` ball/spider/swing, `0.9` robot.
- Terminal fall velocity: **-15** (clamp each step; `+15` when upside down).
- **Cross-check (V):** simulating `yStart = 11.18`, `g = 0.958199`, `dt = 0.25`, explicit Euler (velocity first, then position) gives apex **2.128 blocks** vs the wiki's **2.1333**; air time 93 ticks (0.3875 s). The ~0.005 gap is within the integration-order uncertainty from section 2 and will be closed in step 3.

## 5. Other vehicles (jump/gravity layer only) [R]
- **Ship/UFO/Wave/Swing** are "flying" modes; velocity caps `±8/sizeMod` (ship) with `-6.4/sizeMod` falling cap for mini differences; accelerating/decelerating factors 0.8 / 1.2 (and 0.4 / 0.5 depending on a "falling bugged" flag) are in `updateJump`. Details to be transcribed per mode in step 6.
- **Ball**: jump flips gravity and multiplies Y velocity by 0.6.
- **UFO**: ring-jump tap gives 7.0 (mini 8.0) times size factor, only if it exceeds current velocity.
- **Wave**: velocity is set directly to `±8 * flipMod` or `±playerSpeed * speedMultiplier` depending on input. Full details in step 6.
- **Swing**: gravity factor 0.4 (normal) / 0.6 (mini); tap flips gravity and keeps 0.8x velocity.

## 6. Collision (high-level findings) [R]
- Player rect and object rect are axis-aligned; slopes use a separate path with `playerRadius / cos(slopeAngle)`.
- **Snap-up threshold** (how far inside a block's top you can be and still be "landed"): **10** units normally, **15** when scaled, **5** in platformer; flying modes use **6** (non-platformer). Gravity-flipped negates. This is the key rule behind landing-on-edges behaviour.
- Hitbox sizes (player, spike, orb, pad, portal, block): **[?]** not in the reference files. Needed before step 4. Known community values will be collected and cross-checked.
- "Stair snap" (`checkSnapJumpToObject`): a cube-only correction that nudges X when stepping up/down regular stair patterns; per-speed offset tables are in the reference and will be transcribed in step 5 (threshold 1-2 units).

## 7. 2.1 vs 2.2 [?]
- 2.2 physics is fixed-timestep and refresh-rate independent (240 Hz sub-steps). 2.1 ran physics per rendered frame with delta-time-scaled movement and is known to behave differently across FPS.
- A confirmed per-feature difference list is **not** collected yet. Plan: design every constant/behaviour as `PhysicsVersion`-tagged data; fill V21 later (requires 2.1-era sources).

## 8. Gaps and blockers
1. **Network policy blocks the GD wiki hosts** (`geometrydash.wiki.gg`, `geometry-dash.fandom.com`) and other write-ups (`deepwiki.com`, `byteatatime.dev`). GitHub works. To unblock: allow these hosts in the environment's network settings.
2. Missing from the reference: `PlayerObject::update` (position integration), `checkCollisions`, hitbox sizes, orb/pad/portal effect code, trigger logic.
3. Ground truth needed: tick-by-tick recordings (practice-mode / replay bot data), not just formulas. A formula can look right and still differ by one tick.

## 9. Proposed Step 2 inputs
Everything here will be encoded as data tables in `core/` (versioned by `PhysicsVersion`), with a test per row of section 3 and section 4's cross-check as the first regression test.
