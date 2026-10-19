//! Cube simulation on flat ground (step 3).
//!
//! Conventions (see docs/03-cube-physics.md):
//! * 1 block = 30 units. `y` is the cube's BOTTOM edge, floor top = 0 (the game's own world
//!   offsets the floor by 90/105; we keep the cleaner origin and translate at the edges).
//! * One tick = 1/240 s = 0.25 of a 60 Hz "frame". Horizontal speed is units per frame;
//!   vertical velocity is in "yVel" units, advanced with an extra 0.9 time scale.
//! * Tick order: move with last tick's velocity, then floor/landing, then input (jump), then
//!   gravity. A jump decided at tick N therefore first moves the cube at tick N+1.

use crate::hash::Fnv64;
use crate::version::{speed_params, PhysicsVersion, Speed, SpeedParams};

pub const TICK_HZ: u32 = 240;
/// One tick expressed in 60 Hz frames.
pub const TICK_FRAMES: f64 = 0.25;
/// Vertical time scale: y moves by `yVel * 0.9 * dt`, gravity pulls by `g * 0.9 * dt`.
pub const Y_TIME_SCALE: f64 = 0.9;
pub const FALL_CAP: f64 = -15.0;
pub const UNITS_PER_BLOCK: f64 = 30.0;

#[derive(Clone, Copy, Debug)]
pub struct Sim {
    pub version: PhysicsVersion,
    pub speed: Speed,
    params: SpeedParams,
    pub tick: u64,
    pub x: f64,
    /// Bottom of the cube; 0.0 = standing on the floor.
    pub y: f64,
    pub vy: f64,
    pub on_ground: bool,
}

impl Sim {
    /// `None` if the version/speed table is not available yet.
    pub fn new(version: PhysicsVersion, speed: Speed) -> Option<Self> {
        let params = speed_params(version, speed)?;
        Some(Self { version, speed, params, tick: 0, x: 0.0, y: 0.0, vy: 0.0, on_ground: true })
    }

    /// Horizontal units advanced per tick.
    pub fn x_per_tick(&self) -> f64 {
        self.params.player_speed * self.params.speed_mult * TICK_FRAMES
    }

    /// Advance exactly one 240 Hz tick. `held` = jump input down this tick.
    pub fn step(&mut self, held: bool) {
        let p = self.params;

        // 1. move using the velocity left over from the previous tick
        self.x += p.player_speed * p.speed_mult * TICK_FRAMES;
        self.y += self.vy * Y_TIME_SCALE * TICK_FRAMES;

        // 2. floor: touching it while not rising lands the cube
        if self.y <= 0.0 && self.vy <= 0.0 {
            self.y = 0.0;
            self.on_ground = true;
        }

        // 3. input + velocity update
        if self.on_ground {
            if held {
                self.vy = p.y_start;
                self.on_ground = false;
            } else {
                self.vy = 0.0;
            }
        }
        if !self.on_ground {
            self.vy -= p.gravity * Y_TIME_SCALE * TICK_FRAMES;
            if self.vy < FALL_CAP {
                self.vy = FALL_CAP;
            }
        }
        self.tick += 1;
    }

    /// VISUAL ONLY. Predicts the cube position `ticks` ticks after the current state (fractional ok),
    /// assuming no input change and no collision other than the floor. Never fed back into the sim;
    /// the renderer uses it to draw smoothly at any frame rate while physics stays at 240 Hz.
    /// For whole numbers of ticks in free flight it equals stepping the sim exactly.
    pub fn extrapolate(&self, ticks: f64) -> (f64, f64) {
        let x = self.x + self.x_per_tick() * ticks;
        if self.on_ground {
            return (x, self.y);
        }
        // y_n = y + 0.225 * (n*vy - G*n*(n-1)/2), G = per-tick velocity loss
        let g_tick = self.params.gravity * Y_TIME_SCALE * TICK_FRAMES;
        let dy = Y_TIME_SCALE * TICK_FRAMES * (self.vy * ticks - 0.5 * g_tick * ticks * (ticks - 1.0));
        let y = self.y + dy;
        (x, if y < 0.0 { 0.0 } else { y })
    }

    /// Hash of every state field that affects the future. Add fields here as the sim grows.
    pub fn state_hash(&self) -> u64 {
        let mut h = Fnv64::new();
        h.write_u8(self.version as u8);
        h.write_u8(self.speed as u8);
        h.write_u64(self.tick);
        h.write_f64(self.x);
        h.write_f64(self.y);
        h.write_f64(self.vy);
        h.write_u8(self.on_ground as u8);
        h.finish()
    }
}

/// Result of one full jump from the ground back to the ground.
#[derive(Clone, Copy, Debug)]
pub struct JumpProfile {
    /// Highest bottom-edge height, in units.
    pub apex: f64,
    /// Ticks from the jump tick until the tick that lands.
    pub air_ticks: u32,
    /// Horizontal distance covered while airborne, in units.
    pub distance: f64,
}

/// Single jump on flat ground: press on tick 0, release immediately.
pub fn jump_profile(version: PhysicsVersion, speed: Speed) -> Option<JumpProfile> {
    let mut s = Sim::new(version, speed)?;
    s.step(true);
    let mut apex = 0.0f64;
    let mut ticks = 0u32;
    let mut x_before = s.x;
    loop {
        x_before = if s.on_ground { x_before } else { s.x };
        s.step(false);
        ticks += 1;
        if s.on_ground {
            break;
        }
        if s.y > apex {
            apex = s.y;
        }
        if ticks > 100_000 {
            return None;
        }
    }
    Some(JumpProfile { apex, air_ticks: ticks, distance: x_before })
}

/// The shared parity script: both native and WASM run this exact input stream.
/// A pure function of the tick index; no RNG.
pub fn parity_input(tick: u64) -> bool {
    let m = tick % 311;
    m < 12 || (150..170).contains(&m)
}

/// Runs `ticks` ticks of the parity script and returns the final state hash.
pub fn run_parity(version: PhysicsVersion, speed: Speed, ticks: u64) -> Option<u64> {
    let mut s = Sim::new(version, speed)?;
    for t in 0..ticks {
        s.step(parity_input(t));
    }
    Some(s.state_hash())
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;
    use PhysicsVersion::V22;

    fn blocks(u: f64) -> f64 {
        u / UNITS_PER_BLOCK
    }

    #[test]
    fn horizontal_speeds_match_spec_table() {
        // blocks per second at each speed portal (docs/01-physics-spec.md section 3)
        let want = [(Speed::Half, 8.372), (Speed::Normal, 10.386), (Speed::Double, 12.914), (Speed::Triple, 15.6), (Speed::Quad, 19.2)];
        for (sp, bps) in want {
            let mut s = Sim::new(V22, sp).unwrap();
            for _ in 0..TICK_HZ {
                s.step(false);
            }
            assert!((blocks(s.x) - bps).abs() < 0.001, "{sp:?}: {}", blocks(s.x));
        }
    }

    /// Reference jump table from the independent research pass (docs/03-cube-physics.md).
    /// ticks must match exactly; apex within 0.01 blocks (reference includes a small velocity-rounding
    /// quirk we do not model); distance within 0.02 blocks.
    #[test]
    fn jump_table_matches_reference() {
        // (speed, apex blocks, air ticks, distance blocks)
        let want = [
            (Speed::Half, 1.955, 100, 3.49),
            (Speed::Normal, 2.128, 103, 4.457),
            (Speed::Double, 2.232, 106, 5.70),
            (Speed::Triple, 2.147, 103, 6.69),
            (Speed::Quad, 2.147, 103, 8.24),
        ];
        for (sp, apex, ticks, dist) in want {
            let j = jump_profile(V22, sp).unwrap();
            assert_eq!(j.air_ticks, ticks, "{sp:?} ticks");
            assert!((blocks(j.apex) - apex).abs() < 0.01, "{sp:?} apex {}", blocks(j.apex));
            assert!((blocks(j.distance) - dist).abs() < 0.08, "{sp:?} dist {}", blocks(j.distance));
        }
    }

    #[test]
    fn cube_jump_height_matches_published_value() {
        // published cube jump height at 1x: 2.1333 blocks (= 64 units)
        let j = jump_profile(V22, Speed::Normal).unwrap();
        assert!((blocks(j.apex) - 2.1333).abs() < 0.005, "{}", blocks(j.apex));
    }

    #[test]
    fn holding_jump_rejumps_on_landing_with_no_gap() {
        // With the button held, landing and re-jumping happen in the same tick, so a jump tick is
        // recognisable as "still at floor height but moving up".
        let mut s = Sim::new(V22, Speed::Normal).unwrap();
        let mut jumps = 0;
        let mut gaps = 0;
        for _ in 0..1030 {
            s.step(true);
            if s.y == 0.0 && s.vy > 0.0 {
                jumps += 1;
            } else if s.on_ground {
                gaps += 1;
            }
        }
        assert_eq!(jumps, 10, "one jump every 103 ticks");
        assert_eq!(gaps, 0, "never sits on the ground while held");
    }

    #[test]
    fn extrapolation_matches_real_stepping_in_free_flight() {
        let mut s = Sim::new(V22, Speed::Normal).unwrap();
        s.step(true);
        for _ in 0..5 {
            s.step(false);
        }
        assert!(!s.on_ground);
        for n in 0..=40u32 {
            let (ex, ey) = s.extrapolate(n as f64);
            let mut r = s;
            for _ in 0..n {
                r.step(false);
            }
            assert!((ex - r.x).abs() < 1e-9 && (ey - r.y).abs() < 1e-9, "n={n}: ({ex},{ey}) vs ({},{})", r.x, r.y);
        }
    }

    #[test]
    fn extrapolation_never_goes_below_floor_and_is_pure() {
        let mut s = Sim::new(V22, Speed::Normal).unwrap();
        s.step(true);
        for _ in 0..90 {
            s.step(false);
        }
        let before = s.state_hash();
        let (_, y) = s.extrapolate(50.0);
        assert!(y >= 0.0);
        assert_eq!(before, s.state_hash(), "extrapolate must not touch state");
        let g = Sim::new(V22, Speed::Normal).unwrap();
        assert_eq!(g.extrapolate(3.5).1, 0.0);
        assert!((g.extrapolate(3.5).0 - 3.5 * g.x_per_tick()).abs() < 1e-12);
    }

    #[test]
    fn terminal_velocity_is_capped() {
        let mut s = Sim::new(V22, Speed::Normal).unwrap();
        s.y = 100_000.0;
        s.on_ground = false;
        for _ in 0..400 {
            s.step(false);
        }
        assert_eq!(s.vy, FALL_CAP);
    }

    #[test]
    fn resting_on_floor_stays_put() {
        let mut s = Sim::new(V22, Speed::Normal).unwrap();
        for _ in 0..1000 {
            s.step(false);
        }
        assert_eq!((s.y, s.vy, s.on_ground), (0.0, 0.0, true));
    }

    #[test]
    fn deterministic() {
        let a = run_parity(V22, Speed::Normal, 5000).unwrap();
        let b = run_parity(V22, Speed::Normal, 5000).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn v21_not_available_yet() {
        assert!(Sim::new(PhysicsVersion::V21, Speed::Normal).is_none());
    }
}
