//! The `detrytus` command: `asm`, `run` and `debug`.
//!
//! Its output, messages and exit statuses are those of the Python
//! command it replaced (docs/rust-plan.md), held there by the recorded
//! transcripts in tests/golden/.

use std::io::Write;
use std::process::ExitCode;

use detrytus_asm::disasm::disassemble;
use detrytus_core::hart::Event;
use detrytus_core::trits::low;
use detrytus_core::{Hotplug, Machine, Outcome};

mod debug;
mod program;

use program::{Loaded, load, load_one, strerror};

const USAGE: &str = "usage: detrytus asm FILE [-o FILE] [--listing]
       detrytus run [--ram-pages N] [--max-steps N] [--stats] [--trace]
                    [--harts N] [--plug TIME:HART]... [--unplug TIME:HART]... FILE...
       detrytus debug [--ram-pages N] [-b ADDR]... [-x FILE] [--batch] FILE...";

// Exit statuses, as the Python command uses them (sysexits.h where one fits).
const USAGE_ERROR: u8 = 2;
pub(crate) const BAD_INPUT: u8 = 65;
pub(crate) const NO_INPUT: u8 = 66;
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
    trace: bool,
    harts: i64,
    hotplug: Vec<(i64, Hotplug)>,
}

/// A failure that ends the command: its message and exit status.
pub(crate) struct Fail(String, u8);

impl Fail {
    /// A problem with the command or its files.
    pub(crate) fn new(message: String, status: u8) -> Self {
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
    let mut run = RunArgs {
        files: Vec::new(),
        ram_pages: 729,
        max_steps: 10_000_000,
        stats: false,
        trace: false,
        harts: 1,
        hotplug: Vec::new(),
    };
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--ram-pages" => run.ram_pages = number(&arg, args.next())?,
            "--max-steps" => run.max_steps = number(&arg, args.next())?,
            "--stats" => run.stats = true,
            "--trace" => run.trace = true,
            "--harts" => {
                run.harts = number(&arg, args.next())?;
                if !(1..=729).contains(&run.harts) {
                    return Err(usage("--harts takes 1 to 729"));
                }
            }
            "--plug" | "--unplug" => {
                let spec = args.next().unwrap_or_default();
                let (time, hart) = spec
                    .split_once(':')
                    .and_then(|(t, h)| Some((t.parse().ok()?, h.parse().ok()?)))
                    .ok_or_else(|| usage(&format!("{arg} needs TIME:HART")))?;
                let change =
                    if arg == "--plug" { Hotplug::Add(hart) } else { Hotplug::Remove(hart) };
                run.hotplug.push((time, change));
            }
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
            Loaded::Image(data) => data.clone(),
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
    let mut machine = Machine::with_harts(args.ram_pages, seed(), args.harts);
    for &(time, change) in &args.hotplug {
        machine.schedule(time, change).map_err(|e| usage(&e))?;
    }
    for (addr, tryte) in load(&args.files)?.trytes {
        machine.bus.poke(addr, tryte);
    }
    // With more than one hart, trace lines say which ran each instruction.
    let several = args.harts > 1 || !args.hotplug.is_empty();
    if args.trace {
        machine.record_events(true);
    }
    let mut stdout = std::io::stdout();
    let (mut done, mut printed) = (0u64, 0usize);
    loop {
        // Tracing runs one step at a time, printing what each executed.
        let chunk = if args.trace { 1 } else { CHUNK.min(args.max_steps - done) };
        let rings: Vec<(i64, i64)> = machine.harts().map(|(h, hart)| (h, hart.ring)).collect();
        let outcome = machine.run(chunk);
        for (h, event) in machine.take_events() {
            if let Event::Execute { pc, word } = event {
                // A hart started this step has no ring recorded: it began in +1.
                let ring = rings.iter().find(|(r, _)| *r == h).map_or(1, |(_, ring)| *ring);
                let who = if several { format!("{h:>4} ") } else { String::new() };
                eprintln!("[{who}{pc:>12} r{ring:+}] {}", disassemble(word, Some(pc)));
            }
        }
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
            Outcome::Unhandled { hart, cause, tval, pc, ring } => {
                let mut what =
                    format!("cause {} at pc {pc} in ring {}, tval {tval}", sign(cause), sign(ring));
                if hart != 0 {
                    what += &format!(" on hart {hart}");
                }
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

fn cmd_debug(args: impl Iterator<Item = String>) -> Result<u8, Fail> {
    let (mut files, mut ram_pages, mut breaks, mut commands, mut batch) =
        (Vec::new(), 729, Vec::new(), None, false);
    let mut args = args;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--ram-pages" => ram_pages = number(&arg, args.next())?,
            "-b" | "--break" => {
                breaks.push(args.next().ok_or_else(|| usage("-b needs an address"))?)
            }
            "-x" | "--commands" => {
                commands = Some(args.next().ok_or_else(|| usage("-x needs a file"))?)
            }
            "--batch" => batch = true,
            flag if flag.starts_with('-') && flag != "-" => {
                return Err(usage(&format!("unknown option {flag}")));
            }
            _ => files.push(arg),
        }
    }
    if files.is_empty() {
        return Err(usage("debug needs at least one file"));
    }
    let program = load(&files)?;
    let mut shell = debug::shell::Shell { dbg: debug::engine::Debugger::new(program, ram_pages) };
    for expr in &breaks {
        shell.onecmd(&format!("break {expr}"));
    }
    if let Some(path) = commands {
        let text = std::fs::read_to_string(&path)
            .map_err(|e| Fail::new(format!("{path}: {}", strerror(&e)), NO_INPUT))?;
        for line in text.split_inclusive('\n') {
            if shell.onecmd(&debug::shell::Shell::precmd(line.trim_end_matches('\n'))) {
                return Ok(0);
            }
        }
    }
    if batch {
        return Ok(0);
    }
    println!("{}", debug::shell::INTRO);
    shell.show_current();
    shell.cmdloop();
    Ok(0)
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let result = match args.next().as_deref() {
        Some("asm") => cmd_asm(args),
        Some("run") => parse_run(args).and_then(run),
        Some("debug") => cmd_debug(args),
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
