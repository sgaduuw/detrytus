//! The detrytus command line in Rust (docs/rust-plan.md, phase 5).
//!
//! `detrytus run` for `.dti` images so far, behaving as the Python
//! command does: same output, same messages, same exit statuses. Sources
//! need the assembler, which is still Python: `detrytus asm -o` (Python)
//! turns them into images.

use std::collections::HashSet;
use std::io::Write;
use std::process::ExitCode;

use detrytus_core::image::{self, Segment};
use detrytus_core::{Machine, Outcome};

const USAGE: &str = "usage: detrytus run [--ram-pages N] [--max-steps N] [--stats] IMAGE...";

// Exit statuses, as the Python command uses them (sysexits.h where one fits).
const USAGE_ERROR: u8 = 2;
const BAD_INPUT: u8 = 65;
const NO_INPUT: u8 = 66;
const NOT_IMPLEMENTED: u8 = 69;
const UNHANDLED_TRAP: u8 = 70;
const STEP_LIMIT: u8 = 124;

/// Console output is written out between chunks of this many steps.
const CHUNK: u64 = 100_000;

struct RunArgs {
    files: Vec<String>,
    ram_pages: i64,
    max_steps: u64,
    stats: bool,
}

/// A failure that ends the command: its message and exit status.
struct Fail(String, u8);

impl Fail {
    /// A problem with the command or its files.
    fn new(message: String, status: u8) -> Self {
        Fail(format!("detrytus: {message}"), status)
    }

    /// How a run ended badly: on a fresh line, after the program's output.
    fn after_output(message: String, status: u8) -> Self {
        Fail(format!("\ndetrytus: {message}"), status)
    }
}

fn usage(problem: &str) -> Fail {
    Fail(format!("{USAGE}\ndetrytus: {problem}"), USAGE_ERROR)
}

fn number<T: std::str::FromStr>(flag: &str, value: Option<String>) -> Result<T, Fail> {
    value.and_then(|v| v.parse().ok()).ok_or_else(|| usage(&format!("{flag} needs a number")))
}

fn parse_run(args: impl Iterator<Item = String>) -> Result<RunArgs, Fail> {
    let mut run =
        RunArgs { files: Vec::new(), ram_pages: 729, max_steps: 10_000_000, stats: false };
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--ram-pages" => run.ram_pages = number(&arg, args.next())?,
            "--max-steps" => run.max_steps = number(&arg, args.next())?,
            "--stats" => run.stats = true,
            flag if flag.starts_with("--") => return Err(usage(&format!("unknown option {flag}"))),
            _ => run.files.push(arg),
        }
    }
    if run.files.is_empty() {
        return Err(usage("run needs at least one image"));
    }
    Ok(run)
}

/// Read every image; refuse a file that is not one, and images that
/// overlap, as the Python command does.
fn load(files: &[String]) -> Result<Vec<Segment>, Fail> {
    let mut segments = Vec::new();
    let mut used = HashSet::new();
    for file in files {
        let data = std::fs::read(file).map_err(|e| Fail::new(format!("{file}: {e}"), NO_INPUT))?;
        let parsed = image::parse(&data).map_err(|e| {
            let hint = if data.starts_with(b"DTI") {
                String::new()
            } else {
                " (the Rust command runs images only: make one with `detrytus asm -o`)".into()
            };
            Fail::new(format!("{file}: {e}{hint}"), BAD_INPUT)
        })?;
        for s in &parsed {
            if let Some(addr) =
                (s.start..s.start + s.trytes.len() as i64).find(|a| !used.insert(*a))
            {
                return Err(Fail::new(format!("images overlap at address {addr}"), BAD_INPUT));
            }
        }
        segments.extend(parsed);
    }
    Ok(segments)
}

fn sign(n: i64) -> String {
    format!("{n:+}")
}

fn run(args: RunArgs) -> Result<u8, Fail> {
    let mut machine = Machine::new(args.ram_pages, seed());
    machine.load(&load(&args.files)?);
    let mut stdout = std::io::stdout();
    let (mut done, mut printed) = (0u64, 0usize);
    loop {
        let chunk = CHUNK.min(args.max_steps - done);
        let outcome = machine.run(chunk);
        let text = machine.bus.console.text();
        // Output is best effort, as in Python: a closed pipe ends nothing.
        let _ = stdout.write_all(&text.as_bytes()[printed..]).and_then(|()| stdout.flush());
        printed = text.len();
        match outcome {
            Outcome::Exit { code, steps } => {
                if args.stats {
                    eprintln!("\ndetrytus: {} steps, exit {code}", done + steps);
                }
                // The low byte, as Python's sys.exit gives the system.
                return Ok((code & 0xFF) as u8);
            }
            Outcome::StepLimit => {
                done += chunk;
                if done >= args.max_steps {
                    let message = format!("no power-off after {done} steps");
                    return Err(Fail::after_output(message, STEP_LIMIT));
                }
            }
            Outcome::Unhandled { cause, tval, pc, ring } => {
                let what =
                    format!("cause {} at pc {pc} in ring {}, tval {tval}", sign(cause), sign(ring));
                return Err(Fail::after_output(format!("unhandled trap: {what}"), UNHANDLED_TRAP));
            }
            Outcome::Unsupported(what) => {
                let message = format!("not implemented yet: {what}");
                return Err(Fail::after_output(message, NOT_IMPLEMENTED));
            }
        }
    }
}

/// The entropy seed: random, as the Python command's hardware entropy.
fn seed() -> u32 {
    use std::hash::{BuildHasher, Hasher};
    let mut h = std::collections::hash_map::RandomState::new().build_hasher();
    h.write_u64(std::process::id() as u64);
    h.finish() as u32
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let result = match args.next().as_deref() {
        Some("run") => parse_run(args).and_then(run),
        Some("--version") => {
            println!("detrytus {}", env!("CARGO_PKG_VERSION"));
            Ok(0)
        }
        Some(other) => Err(usage(&format!("unknown command {other}"))),
        None => Err(usage("a command is needed")),
    };
    match result {
        Ok(status) => ExitCode::from(status),
        Err(Fail(message, status)) => {
            eprintln!("{message}");
            ExitCode::from(status)
        }
    }
}
