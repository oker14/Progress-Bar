# GD cube physics: integration, hitboxes, references (step 3)

Tags: [V]=two independent sources agree, [R]=one source, [?]=uncertain.
Behaviour/numbers only; no code copied.
Local clones are in `.../scratchpad/repos/`.
Short names used below:
- PF = camila314/pathfinder, `gd-sim/` (sim for GD 1.x to partial 2.1; commit Mar 2026, README says "full support up to 1.7, partial up to 1.9").
- GDP = camila314/gdp (2.2 decomp snippets).
- OGD = Open-GD/OpenGD (unmaintained re-implementation, roughly 2.1 era).
- GDC = opstic/gdclone (Rust/Bevy, 2024; object data covers ids <= 1911, so 2.1-era).
- OH = prevter/OpenHack (2.2 hitbox renderer).
- GEO = geode-sdk/bindings 2.2074.
- FAN = geometry-dash.fandom.com MediaWiki API (the only wiki reachable).

geometrydash.wiki.gg is Cloudflare-blocked, including Playwright/Chromium ("Just a moment" challenge never clears).
The fandom wiki has no hitbox/physics pages (stub wiki).
GitHub's search API is blocked by the proxy, so repos were found via WebSearch and guessed names.

## 1. Per-tick integration (cube)

Tick = 1/240 s. GDP update (GJBaseGameLayer_update.cpp):
- `stepCount = max(1, delta*240 / min(timewarp,1))`.
- `physicsSecond = delta*60/stepCount`, so one 240 Hz tick gives dt = 0.25 in "60 Hz-frame units" [R, GDP].
- Per step the order is: `player->update(dt)`, then `checkCollisions(player, dt)`, then `updateSpecial`/`updateRotation` [R, GDP].
- x/y collision order inside checkCollisions is not in GDP (PlayerObject::update and checkCollisions are missing).

x advance: `x += speed * dt` every tick.
- Speed in units/s: 0.7x 251.16008, 1x 311.58009, 2x 387.42014, 3x 468.00014, 4x 576.00020 [R, PF player_speeds].
- Per tick that is 1.0465, 1.2983, 1.6143, 1.95, 2.4 units.
- OGD does the same thing in 60 Hz-frame terms: `velX = dt * xVel * playerSpeed` [R, OGD PlayerObject.cpp::update].

y advance: `y += yVel * 0.9 * dt` (dt = physicsSecond).
- So yVel is in "units per 60 Hz frame / 0.9". Equivalently a velocity stored as units/s = yVel*54 and advanced by 1/240 per tick.
- Evidence: PF uses `pos.y += vel*dt` with dt = 1/240 and jump 603.72 = 11.180032*54.
- OGD: `dtSlow = dt*0.9; velY = dtSlow * yVel` [V, PF + OGD].
- GDC uses a `*0.9` slowed delta too [R].

Gravity: for the cube, `yVel -= g * dt * 0.9 * flipMod`, with g = m_gravity (0.958199 at 1x) [V, GDP updateJump float_b=0.9 + PF accel table].
- PF accel in units/s²: 0.7x -2747.52, 1x -2794.1082, 2x -2786.4, 3x -2799.36, 4x -2799.36 [R].
- Check: 0.958199*60*60*0.81 = 2794.1, so the 0.9 enters twice (once in the velocity scale, once in the gravity step).
- Per tick at 1x: dv = -0.21559 yVel units, i.e. -11.642 units/s.

Clamp: `yVel = max(yVel, -15)` [V, GDP lines ~200/459 + OGD `max(m_dYVel,-15)` + GDC VELOCITY_LIMIT 15]. PF has -810 = -15*54.

Order of operations, as modelled by the most careful sim (PF, which was validated against the game for old versions) [R]:
1. preCollision: `x += vx*dt; y += vy*dt` using the velocity from the end of the previous tick; frame++.
2. Collisions run: sort order is non-block triggers (portals, pads, orbs) first, then blocks in reverse list order, then hazards last (prio 0/1/2).
3. postCollision: floor/ceiling snap, then vehicle.update (jump if grounded and input), then `v += a*dt`, then rounding, then clamp.

So a jump set at tick N applies `v = jump + a*dt` immediately, and the first y movement happens at tick N+1.
- OGD (2.1) instead calls updateJump (gravity and jump) before moving in the same update. That is equivalent up to a one-tick phase shift [?].
- The exact 2.2 order cannot be verified without PlayerObject::update. For tick-exactness, test against recorded replays.

Rounding quirk (PF) [R]:
- After most operations the velocity (in yVel units, i.e. v/54) is rounded to 3 decimals of its fractional part: `n = v/54; frac = round((n - trunc(n))*1000)/1000`.
- Ball is excluded.
- Effect is tiny: apex 63.968 -> 63.847 units at 1x.
- It looks like float-truncation emulation. Probably skip it, or add it as an optional "faithful" mode.

Ground: floor handling.
- OGD: cube centre y = 105 (mini 99). Level object y is stored +90, so floor top y = 90 in world units and a 30-unit cube rests at y = 105 [R, OGD PlayLayer.cpp::checkCollisions and line 392 `+90`].
- Cube touching floor: `if y < 105: y = 105; hitGround()`. Block landing: `y = block top + 15` (+0 for non-mini) [R, OGD/PF].
- PF works in editor coordinates: floor = 0, cube centre y = 15; its README says real GD is offset by 105 [R]. These agree.
- Landing: PF snaps when `playerBottom <= floor && v <= 0 && !velocityOverride` and sets grounded.
- While grounded and not clicking: `setVelocity(0)` each tick (falls off blocks one frame faster than naive).
- Coyote/buffer: PF has cube coyote frames (upside-down cube jumps if coyote < 10 frames) and orb input buffering. Details are in the PF source (Vehicle.cpp cube, Orb.cpp).

Block collision rule for cube [R, PF Block.cpp; OGD agrees on the 10-unit tolerance]:
- Overlapping block with the inner hitbox, or inner hitbox only: death (`innerHitbox.intersects(block)`).
- Else if `blockTop - playerBottom <= 10` and (`v <= 0`, or the pad/gravity-portal frame): snap `y = blockTop + h/2` and set grounded (the 10 is 7 for ship/ufo and for slopes).
- Else: nothing. A side hit only kills through the inner hitbox.
- OGD: `mod = 10` for the cube, `6` for the ship.
- Death also occurs when the inner hitbox top reaches the ceiling (cube).
- Block "snap-up" X-nudges for stair patterns are in PF's Block.cpp::snapThreshold (you said snap-up thresholds are already known).
- Mini player: after a size portal the hitbox size changes at end of frame (PF postCollision).

Jump initial velocities (units/s, from PF jumpHeights = yStart*54) [R]:
- 0.7x 573.4817 (10.6200 *54)
- 1x 603.7217 (11.180032*54)
- 2x 616.6817
- 3x/4x 606.4217
- Small cube: `v * 0.8` [R, PF setVelocity].
- GDC jump const 11.180032 matches [V for 1x].

## 2. Hitboxes (game units, 30 = 1 block)

Player [V]:
- Outer: 30x30 centred on the player position; mini = 0.6 -> 18x18 [R, OGD `getOuterBounds(0.6,0.6)`; PF size*0.6].
- Inner:
  - 2.2 [R, OH]: outer rect scaled by 0.25 (`vehicleSize >= 1`, normal) -> 7.5x7.5; or by 0.4 (`vehicleSize < 1`, mini) -> 7.2x7.2 for the 18-unit cube.
  - OGD (2.1-era): 7.5x7.5 centred (offset 11.25) for normal [R]. OGD mini inner is not specified.
  - PF (older): 9x9 fixed [R]. This conflicts with the 7.5 above; PF may be tuned for pre-2.1 versions. Use 7.5 / 7.2 for 2.2 [V: OH + OGD for 7.5; mini 0.4 only OH].
- Outer is unrotated for object tests when the object's rotation is a multiple of 90; otherwise the game uses an OBB. PF: `touching()` uses the player's unrotated hitbox for axis-aligned rotations and the rotated hitbox otherwise [R]. GDP/PF: cube rotation affects collision only vs non-90-multiple objects.

Object hitboxes [from GDC object.json, extracted from the game, ids <= 1911 = 2.1-era; PF Object.cpp table agrees except where noted] (V where PF and GDC agree):

| Object (id) | Hitbox | Role |
|---|---|---|
| Block 1-4 etc. | 30x30 | solid |
| Half slab (40, 147 ...) | 30x14 in PF | solid [R] |
| Standard spike (8) | box 6 x 12 (centred) | kill [V] |
| Ground spike big (9) | 9 x 10.8 | kill [V] |
| Small spike (39) | 6 x 5.6 | kill [V] |
| Small ground spike (103) | 4 x 7.6 | kill [V] |
| Spike 61 | 9 x 7.2 | kill [V] |
| Spike 135 | 14.1 x 20 | kill [V] |
| Spike 392 | 2.6 x 4.8 | kill [V] |
| Saw 88 | circle r = 32.3 | kill [V] |
| Saw 89 | circle r = 21.6 | kill [V] |
| Saw 98 | circle r = 11.78 (PF says 12) | kill |
| Orb (36 yellow, 84 blue, 141 pink, 1333 red, 1022 green, 1330 black) | 36 x 36 | trigger [V] |
| Pad yellow 35 | 25 x 4 | trigger [V] |
| Pad blue 67 | 25 x 6 | trigger [V] |
| Pad pink 140 | 25 x 5 | trigger [V] |
| Pad red 1332 | 29 x 7 (GDC) | trigger [R] |
| Gravity portal 10/11 | 25 x 75 | trigger [V] |
| Vehicle portals 12/13/47/111 | 34 x 86 | trigger [V] |
| Size portal 99/101 | 31 x 90 | trigger [V] |
| Speed portal 200 | 35 x 44 | trigger [V] |
| Speed portal 201 | 33 x 56 | trigger [V] |
| Speed portal 202 | 51 x 56 | trigger [V] |
| Speed portal 203 | 65 x 56 | trigger [V] |
| Speed portal 1334 | 69 x 56 | trigger [V] |
| Mirror portal 45/46 | 44 x 92 | trigger [R, GDC] |

Notes:
- Object sizes are multiplied by object scale (PF: scale * scaleX / scaleY). Block rotation swaps width/height (PF Block ctor).
- 2.2 changes: GEO shows animated saw 918 radius 24 (48x48 box) and 919 a 25x6 box. Spike and block sizes are probably unchanged in 2.2 [?]; I found no 2.2 hitbox table. 2.2 added many new ids (>1911) that GDC does not cover.
- Slopes are triangles on a 30x30 or 60x30 box (PF `Slope`).
- Hitbox roles from GEO `GameObjectType`: Solid=0, Hazard=2, YellowJumpPad=8, PinkJumpPad=9, GravityPad=10, YellowJumpRing=11, PinkJumpRing=12, GravityRing=13, Breakable=21, Slope=25, GreenRing=29, RedJumpPad=34, RedJumpRing=35, DashRing=37, Special=40, AnimatedHazard=47 and so on. Hazard (and AnimatedHazard) kills; Solid blocks per the rule above; pads, rings and portals are triggers [R, GEO Enums.hpp]. OH colours them solid=blue, danger=red, other (portals etc.)=green [R].
- Kill test: hazards use player outer rect vs object rect, or outer rect vs circle for saws (circle test: any outer corner within r, or the centre inside the rect) [R, OGD + PF Sawblade::touching]. Hazards are tested last, after blocks, in the same tick [R, PF].
- Orb: tested with a one-frame coyote (current OR previous player rect) [R, PF Orb::touching]. The click must be buffered/pressed this frame.
- Pad: fires on touch without a click.

## 3. Reference measurements (my sim of PF constants, 240 Hz, landing on the starting level)

Jump table (cube, ground to ground), dx in blocks:

| Speed | apex (blocks) | ticks | seconds | dx (blocks) |
|---|---|---|---|---|
| 0.7x | 1.955 | 100 | 0.4167 | 3.49 |
| 1x | 2.128 (2.132 without rounding) | 103 | 0.4292 | 4.457 |
| 2x | 2.232 | 106 | 0.4417 | 5.70 |
| 3x | 2.147 | 103 | 0.4292 | 6.69 |
| 4x | 2.147 | 103 | 0.4292 | 8.24 |

- Mini 1x: apex 1.355 blocks, 82 ticks, dx 3.55 blocks.
- Landing criterion: first tick with y <= 0 and v <= 0.
- Speeds in blocks/s: 8.372, 10.386, 12.914, 15.6, 19.2.
- 1x: 52 ticks from jump to apex. A fall from rest reaches the -810 limit after 70 ticks.

Compare with the fandom Portals page, which lists cube jump height 2.1333 blocks (sustained 2.233) and mini 1.3583 (1.435) [R, https://geometry-dash.fandom.com/api.php?action=parse&page=Portals&prop=wikitext&format=json]:
- My 1x cube is 2.13 -> matches 2.1333 (= 64 units).
- Mini is 1.355 -> matches 1.3583.
- The "sustained" 2.233 matches my 2x number (2.232), so it may be a 2x value; ambiguous [?].
- So my integration (move-then-accelerate, 240 Hz, dt=0.25) reproduces published heights to < 0.01 blocks [V: PF constants + fandom].

Boosts at 1x (PF velocities, same-level landing, my sim):

| Boost | v (units/s) | apex (blocks) | ticks | dx (blocks) |
|---|---|---|---|---|
| Yellow pad | 864 | 4.385 | 148 | 6.40 |
| Pink pad | 561.6 | 1.839 | 96 | 4.15 |
| Red pad | 1080 | 6.870 | 188 | 8.14 |
| Yellow orb | 603.72 | 2.128 | 103 | 4.46 |
| Pink orb | 434.7 | 1.095 | 74 | 3.20 |
| Red orb | 821.448 | 3.961 | 140 | 6.06 |
| Green orb | 592.056 | 2.046 | 101 | 4.37 |

- Pad velocities are the same at all speeds for the cube (864 yellow, 561.6 pink, 1080 red, -345.6 blue). Mini: 691.2, 449.28, 864, -276.48 [R, PF Pad.cpp].
- Orb velocities depend on speed (the table in PF Orb.cpp is [slow, 1x, 2x, 3x]); the 4x column equals 3x. Yellow/cube: 573.48, 603.72, 616.68, 606.42. Pink/cube: 412.884, 434.7, 443.988, 436.644. Red/cube: 779.976, 821.448, 839.43, 825.174. Blue/cube: -229.392, -241.488, -246.672, -242.568. Mini variants are in the same file.
- Black orb sets v = -810.

Everything in this section is derived from PF constants plus my own integrator. No measured in-game data was available except the two fandom numbers.

## 4. Replay formats and clones

GDR / GDR2 (maxnut/GDReplayFormat) [R]:
- Stores inputs only: frame (uint64, 240 Hz default; `framerate` field), button, player2, down. Plus metadata (author, duration, gameVersion e.g. 22074, seed, ldm, platformer, level id/name, deaths list).
- Binary with varints. File extension .gdr2 for the binary form. Older .gdr (JSON) is used by xdBot.
- Extensions allow custom per-frame data.
- No positions in the base format.

xdBot's gdr.hpp (Zilko/xdBot) adds "frame fixes": per frame `{pos.x, pos.y, rotation}` for p1/p2 [R]. That is a tick-level position source, but only through macros made with xdBot. I found no sample files.
- ToastyReplay: own TTR3 format (exact input timing, TPS events, anchors, checkpoints) [R].
- TCBot: `.tcm` (v1/v2) via tcbot-gd/tcm-rs, which includes sample `.tcm` files in `examples/data/` (restartv1/v2, longv2); TPS field present. Inputs only.
- SLC format (crate slc_oxide): inputs, deaths, TPS.
- Mega Hack / Zilko replays: proprietary.

Result: NO publicly accessible tick-by-tick position traces found. Comparing tick-by-tick requires recording one yourself (GDR + xdBot frame fixes, or Geode mod logging `m_position` per physics step).

Other clones and sims:
- PF `gd-sim` — best source for exact constants. Debug output prints per-frame X, Y, Vel, Accel; its README says Y is offset by 105 vs real GD. Not licensed for redistribution — constants only.
- opstic/gdclone — Rust/Bevy; has a hitbox table in `assets/data/object.json` (extracted, 2.1-era), crude cube physics (not accurate).
- Open-GD/OpenGD — C++/Axmol, GPL-3.0, README says UNMAINTAINED; readable PlayerObject/PlayLayer with 2.1-era logic.
- Emanuele Feronato's Phaser/TS tutorial uses a 1/240 s step with axis-aligned hitboxes (cosmetic rotation) per the search snippet [?].

## 5. Unresolved / caveats

- No 2.2 PlayerObject::update or checkCollisions source: tick order and the exact step where jump/gravity are applied are inferred from PF (2.1-era) and OGD (older); a one-tick phase uncertainty remains.
- No 2.2-specific hitbox table. 2.1 object hitboxes (GDC/PF) probably still hold for classic objects; verify spikes/saws against a hitbox-view mod on 2.2 if possible.
- Player inner hitbox: 7.5 / 0.4*18=7.2 in 2.2 per OpenHack; PF uses 9.
- wiki.gg (the best hitbox/physics wiki) was unreachable.
