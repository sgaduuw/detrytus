//! The detrytus command line in Rust (docs/rust-plan.md, phase 5).
//!
//! `detrytus asm` and `detrytus run`, behaving as the Python command
//! does: the same output, messages and exit statuses for the same files.
//! The debugger is still Python.

use std::collections::HashSet;
use std::io::{Read, Write};
use std::process::ExitCode;

use detrytus_asm::{AsmError, Image, assemble};
use detrytus_core::image::{self, Segment};
use detrytus_core::trits::low;
use detrytus_core::{Machine, Outcome};

const USAGE: &str = "usage: detrytus asm FILE [-o FILE] [--listing]
       detrytus run [--ram-pages N] [--max-steps N] [--stats] FILE...";
const MAGIC: &[u8] = b"DTI\x01";

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
            flag if flag.starts_with('-') && flag != "-" => {
                return Err(usage(&format!("unknown option {flag}")));
            }
            _ => run.files.push(arg),
        }
    }
    if run.files.is_empty() {
        return Err(usage("run needs at least one file"));
    }
    Ok(run)
}

/// A file as the commands take it: assembly source, an image, or - for
/// source on stdin.
enum Loaded {
    Source(Image),
    Image(Vec<u8>, Vec<Segment>),
}

impl Loaded {
    fn segments(&self) -> Vec<Segment> {
        match self {
            Loaded::Image(_, segments) => segments
                .iter()
                .map(|s| Segment { start: s.start, trytes: s.trytes.clone() })
                .collect(),
            Loaded::Source(image) => {
                let mut runs: Vec<Segment> = Vec::new();
                for (&addr, &tryte) in &image.trytes {
                    match runs.last_mut() {
                        Some(run) if run.start + run.trytes.len() as i64 == addr => {
                            run.trytes.push(tryte)
                        }
                        _ => runs.push(Segment { start: addr, trytes: vec![tryte] }),
                    }
                }
                runs
            }
        }
    }
}

fn asm_error(e: AsmError) -> Fail {
    Fail::new(e.to_string(), BAD_INPUT)
}

fn load_one(path: &str) -> Result<Loaded, Fail> {
    if path == "-" {
        let mut text = String::new();
        std::io::stdin()
            .read_to_string(&mut text)
            .map_err(|e| Fail::new(format!("<stdin>: {e}"), NO_INPUT))?;
        return assemble(&text, Some("<stdin>")).map(Loaded::Source).map_err(asm_error);
    }
    let data = std::fs::read(path)
        .map_err(|e| Fail::new(format!("{path}: {}", strerror(&e)), NO_INPUT))?;
    if data.starts_with(MAGIC) {
        let segments = image::parse(&data).map_err(|e| Fail::new(e, BAD_INPUT))?;
        return Ok(Loaded::Image(data, segments));
    }
    let text = String::from_utf8(data)
        .map_err(|_| Fail::new(format!("{path}: not UTF-8 text"), BAD_INPUT))?;
    assemble(&text, Some(path)).map(Loaded::Source).map_err(asm_error)
}

fn strerror(e: &std::io::Error) -> String {
    let text = e.to_string();
    text.rfind(" (os error ").map_or(text.clone(), |i| text[..i].to_string())
}

/// Every file's trytes; images that overlap are refused, as the Python
/// command does.
fn load(files: &[String]) -> Result<Vec<Segment>, Fail> {
    let mut segments = Vec::new();
    let mut used = HashSet::new();
    for file in files {
        let parsed = load_one(file)?.segments();
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

fn format_trits(word: i64) -> String {
    let mut x = word;
    let mut trits = Vec::with_capacity(27);
    for _ in 0..27 {
        let t = low(x, 1);
        trits.push(match t {
            -1 => '-',
            0 => '0',
            _ => '+',
        });
        x = (x - t) / 3;
    }
    trits.iter().rev().collect()
}

fn cmd_asm(args: impl Iterator<Item = String>) -> Result<u8, Fail> {
    let (mut file, mut output, mut listing) = (None, None, false);
    let mut args = args;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" | "--output" => {
                output = Some(args.next().ok_or_else(|| usage("-o needs a file"))?)
            }
            "--listing" => listing = true,
            flag if flag.starts_with('-') && flag != "-" => {
                return Err(usage(&format!("unknown option {flag}")));
            }
            _ if file.is_some() => return Err(usage("asm takes one file")),
            _ => file = Some(arg),
        }
    }
    let file = file.ok_or_else(|| usage("asm needs a file"))?;
    let loaded = load_one(&file)?;
    if let Some(out) = &output {
        let bytes = match &loaded {
            Loaded::Source(image) => image.to_bytes(),
            Loaded::Image(data, _) => data.clone(),
        };
        std::fs::write(out, bytes)
            .map_err(|e| Fail::new(format!("{out}: {}", strerror(&e)), NO_INPUT))?;
        if !listing {
            return Ok(0);
        }
    }
    let Loaded::Source(image) = loaded else { return Ok(0) };
    let mut stdout = std::io::stdout().lock();
    for e in &image.listing {
        // A section that runs somewhere other than where it loads shows both.
        let at =
            if e.load == e.addr { e.addr.to_string() } else { format!("{}@{}", e.addr, e.load) };
        let trits = e.word.map(format_trits).unwrap_or_default();
        let source = detrytus_asm::text::strip(&e.source);
        let _ = writeln!(stdout, "{at:>22}  {trits:27}  {:28}  {source}", e.text);
    }
    Ok(0)
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
        Some("asm") => cmd_asm(args),
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
