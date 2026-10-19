//! Deterministic physics core. `no_std`, no dependencies, no I/O, no randomness.
//!
//! Rules that keep native and WASM builds bit-identical:
//! * only `+ - * /` and comparisons on floats (all IEEE-754 exact); no sin/cos/pow/exp here
//! * no fused multiply-add, no fast-math (rustc does neither by default)
//! * fixed tick, input is a pure function of (tick) supplied by the caller
#![no_std]

pub mod hash;
pub mod sim;
pub mod version;

pub use sim::{Sim, TICK_HZ};
pub use version::PhysicsVersion;
