//! Run `.dti` images on the Rust core, for trying it out and timing it
//! against the Python one. Not the Rust command line: that is phase 5 of
//! docs/rust-plan.md.
//!
//!     cargo run --release --example run -- [--ram-pages N] IMAGE...

use std::process::ExitCode;
use std::time::Instant;

use detrytus_core::{Machine, Outcome, image};

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let (mut ram_pages, mut files) = (729, Vec::new());
    while let Some(arg) = args.next() {
        if arg == "--ram-pages" {
            ram_pages = args.next().and_then(|n| n.parse().ok()).expect("--ram-pages N");
        } else {
            files.push(arg);
        }
    }
    if files.is_empty() {
        eprintln!("usage: run [--ram-pages N] IMAGE...");
        return ExitCode::from(2);
    }
    let mut machine = Machine::new(ram_pages, 1);
    for file in &files {
        let data = std::fs::read(file).unwrap_or_else(|e| panic!("{file}: {e}"));
        machine.load(&image::parse(&data).unwrap_or_else(|e| panic!("{file}: {e}")));
    }
    let start = Instant::now();
    let outcome = machine.run(u64::MAX);
    let elapsed = start.elapsed();
    print!("{}", machine.bus.console.text());
    eprintln!(
        "{outcome:?} after {:.3} s, {} instructions",
        elapsed.as_secs_f64(),
        machine.hart.instret
    );
    match outcome {
        Outcome::Exit { code: 0, .. } => ExitCode::SUCCESS,
        _ => ExitCode::FAILURE,
    }
}
