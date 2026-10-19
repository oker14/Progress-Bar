//! Thin C-ABI wrapper for the browser. No wasm-bindgen: fewer moving parts, smaller binary,
//! works from a single base64-inlined file. A fixed pool of sims supports batch use later.
#![no_std]

use gd_core::sim::{parity_input, Sim};
use gd_core::version::{PhysicsVersion, Speed};

const POOL: usize = 64;
static mut SIMS: [Option<Sim>; POOL] = [None; POOL];

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

fn slot(i: u32) -> Option<&'static mut Option<Sim>> {
    // SAFETY: wasm is single-threaded; every access goes through these exports.
    unsafe { (*core::ptr::addr_of_mut!(SIMS)).get_mut(i as usize) }
}

/// Returns 1 on success, 0 on bad index / unavailable version table.
#[no_mangle]
pub extern "C" fn gd_reset(i: u32, version: u32, speed: u32) -> u32 {
    let (Some(v), Some(sp), Some(s)) =
        (PhysicsVersion::from_u8(version as u8), Speed::from_u8(speed as u8), slot(i))
    else {
        return 0;
    };
    match Sim::new(v, sp) {
        Some(sim) => {
            *s = Some(sim);
            1
        }
        None => 0,
    }
}

#[no_mangle]
pub extern "C" fn gd_step(i: u32, held: u32) {
    if let Some(Some(s)) = slot(i) {
        s.step(held != 0);
    }
}

/// Run `n` ticks of the shared parity input script from the sim's current tick.
#[no_mangle]
pub extern "C" fn gd_run_parity(i: u32, n: u32) {
    if let Some(Some(s)) = slot(i) {
        for _ in 0..n {
            let t = s.tick;
            s.step(parity_input(t));
        }
    }
}

#[no_mangle]
pub extern "C" fn gd_hash_lo(i: u32) -> u32 {
    match slot(i) { Some(Some(s)) => s.state_hash() as u32, _ => 0 }
}
#[no_mangle]
pub extern "C" fn gd_hash_hi(i: u32) -> u32 {
    match slot(i) { Some(Some(s)) => (s.state_hash() >> 32) as u32, _ => 0 }
}
#[no_mangle]
pub extern "C" fn gd_x(i: u32) -> f64 { match slot(i) { Some(Some(s)) => s.x, _ => 0.0 } }
#[no_mangle]
pub extern "C" fn gd_y(i: u32) -> f64 { match slot(i) { Some(Some(s)) => s.y, _ => 0.0 } }
#[no_mangle]
pub extern "C" fn gd_tick(i: u32) -> f64 { match slot(i) { Some(Some(s)) => s.tick as f64, _ => 0.0 } }
#[no_mangle]
pub extern "C" fn gd_vy(i: u32) -> f64 { match slot(i) { Some(Some(s)) => s.vy, _ => 0.0 } }
#[no_mangle]
pub extern "C" fn gd_on_ground(i: u32) -> u32 { match slot(i) { Some(Some(s)) => s.on_ground as u32, _ => 0 } }
