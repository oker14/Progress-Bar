//! SCAFFOLD simulation (step 2). Just enough cube physics from spec sections 3-4 to prove the
//! pipeline: fixed tick, state hashing, native == WASM. Step 3 replaces/extends this and
//! validates it against reference arcs.
//!
//! Units: 30 units = 1 block. Velocities are per 1/60 s frame (the reference's convention);
//! one tick is 1/240 s = 0.25 frame (`TICK_FRAMES`).

use crate::hash::Fnv64;
use crate::version::{speed_params, PhysicsVersion, Speed, SpeedParams};

pub const TICK_HZ: u32 = 240;
pub const TICK_FRAMES: f64 = 0.25;
pub const FALL_CAP: f64 = -15.0;

#[derive(Clone, Copy, Debug)]
pub struct Sim {
    pub version: PhysicsVersion,
    pub speed: Speed,
    params: SpeedParams,
    pub tick: u64,
    pub x: f64,
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

    /// Advance exactly one 240 Hz tick. `held` = jump input down this tick.
    pub fn step(&mut self, held: bool) {
        let p = self.params;
        if held && self.on_ground {
            self.vy = p.y_start;
            self.on_ground = false;
        }
        // velocity first, then position (see docs/01-physics-spec.md section 4)
        if !self.on_ground {
            self.vy -= p.gravity * TICK_FRAMES;
            if self.vy < FALL_CAP {
                self.vy = FALL_CAP;
            }
        }
        self.y += self.vy * TICK_FRAMES;
        if self.y <= 0.0 {
            self.y = 0.0;
            self.vy = 0.0;
            self.on_ground = true;
        }
        self.x += p.player_speed * p.speed_mult * TICK_FRAMES;
        self.tick += 1;
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

    #[test]
    fn normal_speed_is_10_386_blocks_per_second() {
        let mut s = Sim::new(PhysicsVersion::V22, Speed::Normal).unwrap();
        for _ in 0..TICK_HZ {
            s.step(false);
        }
        let blocks = s.x / 30.0;
        assert!((blocks - 10.386).abs() < 0.001, "{blocks}");
    }

    #[test]
    fn cube_jump_apex_close_to_wiki_2_1333() {
        let mut s = Sim::new(PhysicsVersion::V22, Speed::Normal).unwrap();
        let mut apex = 0.0f64;
        s.step(true);
        while !s.on_ground {
            apex = apex.max(s.y);
            s.step(false);
        }
        let blocks = apex / 30.0;
        // Step 3 tightens this to the exact reference arc; scaffold tolerance only.
        assert!((blocks - 2.1333).abs() < 0.02, "{blocks}");
    }

    #[test]
    fn deterministic() {
        let a = run_parity(PhysicsVersion::V22, Speed::Normal, 5000).unwrap();
        let b = run_parity(PhysicsVersion::V22, Speed::Normal, 5000).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn v21_not_available_yet() {
        assert!(Sim::new(PhysicsVersion::V21, Speed::Normal).is_none());
    }
}
