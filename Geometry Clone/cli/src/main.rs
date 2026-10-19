//! `gd parity [ticks]`  -> prints the final state hash of the shared parity script.
//! `gd bench [ticks]`   -> raw single-thread simulation throughput.
use gd_core::sim::{run_parity, Sim};
use gd_core::version::{PhysicsVersion, Speed};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("parity");
    let ticks: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(100_000);
    match cmd {
        "parity" => {
            let h = run_parity(PhysicsVersion::V22, Speed::Normal, ticks).expect("table");
            println!("{h:016x}");
        }
        "bench" => {
            let mut s = Sim::new(PhysicsVersion::V22, Speed::Normal).unwrap();
            let t0 = Instant::now();
            for t in 0..ticks {
                s.step(gd_core::sim::parity_input(t));
            }
            let dt = t0.elapsed().as_secs_f64();
            println!("{} ticks in {:.3}s = {:.1} M ticks/s (hash {:016x})",
                ticks, dt, ticks as f64 / dt / 1e6, s.state_hash());
        }
        _ => {
            eprintln!("usage: gd [parity|bench] [ticks]");
            std::process::exit(2);
        }
    }
}
