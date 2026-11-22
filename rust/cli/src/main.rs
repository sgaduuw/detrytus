//! The `detrytus` command: `asm`, `run` and `debug`.
//!
//! Its output, messages and exit statuses are those of the Python
//! command it replaced (docs/rust-plan.md), held there by the recorded
//! transcripts in tests/golden/.

use std::io::{IsTerminal, Write};
use std::process::ExitCode;
use std::time::Duration;

use detrytus_asm::disasm::disassemble;
use detrytus_core::hart::Event;
use detrytus_core::trits::low;
use detrytus_core::{Hotplug, Machine, Outcome};

mod debug;
mod disk;
mod live;
mod mix;
mod program;

use disk::DiskSpec;

use program::{Loaded, load, load_one, strerror};

const USAGE: &str = "usage: detrytus asm FILE [-o FILE] [--listing] [--mix]
       detrytus run [--ram-pages N] [--max-steps N] [--stats] [--trace] [--mix] [--input FILE]
                    [--live] [--harts N] [--plug TIME:HART]... [--unplug TIME:HART]...
                    [--disk FILE [--disk-blocks N]] FILE...
       detrytus debug [--ram-pages N] [-b ADDR]... [-x FILE] [--batch] [--input FILE]
                    [--harts N] [--plug TIME:HART]... [--unplug TIME:HART]...
                    [--disk FILE [--disk-blocks N]] FILE...";

// Exit statuses, as the Python command uses them (sysexits.h where one fits).
const USAGE_ERROR: u8 = 2;
pub(crate) const BAD_INPUT: u8 = 65;
pub(crate) const NO_INPUT: u8 = 66;
const NOT_IMPLEMENTED: u8 = 69;
const UNHANDLED_TRAP: u8 = 70;
const STEP_LIMIT: u8 = 124;
const STOPPED: u8 = 130; // Ctrl-] on a live console, as for Ctrl-C

/// Console output is written out between chunks of this many steps.
const CHUNK: u64 = 100_000;
/// With a live console, smaller chunks, so that typing is answered at once.
const LIVE_CHUNK: u64 = 10_000;
/// How long a stalled machine (Machine::take_stalled) waits for a key
/// before it runs on.
const LIVE_WAIT: Duration = Duration::from_millis(20);

struct RunArgs {
    files: Vec<String>,
    ram_pages: i64,
    max_steps: Option<u64>,
    stats: bool,
    trace: bool,
    mix: bool,
    harts: i64,
    hotplug: Vec<(i64, Hotplug)>,
    input: Option<String>,
    live: bool,
    disk: Option<DiskSpec>,
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
        max_steps: None,
        stats: false,
        trace: false,
        mix: false,
        harts: 1,
        hotplug: Vec::new(),
        input: None,
        live: false,
        disk: None,
    };
    let mut disk = DiskArgs::default();
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--ram-pages" => run.ram_pages = number(&arg, args.next())?,
            "--max-steps" => run.max_steps = Some(number(&arg, args.next())?),
            "--live" => run.live = true,
            "--stats" => run.stats = true,
            "--trace" => run.trace = true,
            "--mix" => run.mix = true,
            "--input" => {
                run.input = Some(args.next().ok_or_else(|| usage("--input needs a file"))?)
            }
            "--harts" | "--plug" | "--unplug" => {
                hart_option(&arg, args.next(), &mut run.harts, &mut run.hotplug)?
            }
            "--disk" | "--disk-blocks" => disk.option(&arg, args.next())?,
            flag if flag.starts_with('-') && flag != "-" => {
                return Err(usage(&format!("unknown option {flag}")));
            }
            _ => run.files.push(arg),
        }
    }
    run.disk = disk.spec()?;
    if run.files.is_empty() {
        return Err(usage("run needs at least one file"));
    }
    if run.input.as_deref() == Some("-") && run.files.iter().any(|f| f == "-") {
        return Err(usage("stdin can be a FILE or the --input, not both"));
    }
    if run.live && run.input.is_some() {
        return Err(usage("--input and --live both give the console its input"));
    }
    // Typed at a terminal, with the output on one, the console is live.
    let terminal = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    if run.input.is_none() && terminal {
        run.live = true;
    }
    if run.live && run.files.iter().any(|f| f == "-") {
        return Err(usage("stdin can be a FILE or the live console, not both"));
    }
    Ok(run)
}

/// --harts N, --plug TIME:HART or --unplug TIME:HART, checked here so
/// that a bad one is a usage error before anything runs.
fn hart_option(
    flag: &str,
    value: Option<String>,
    harts: &mut i64,
    hotplug: &mut Vec<(i64, Hotplug)>,
) -> Result<(), Fail> {
    if flag == "--harts" {
        *harts = number(flag, value)?;
        if !(1..=729).contains(harts) {
            return Err(usage("--harts takes 1 to 729"));
        }
        return Ok(());
    }
    let (time, hart) = value
        .as_deref()
        .and_then(|v| v.split_once(':'))
        .and_then(|(t, h)| Some((t.parse().ok()?, h.parse().ok()?)))
        .ok_or_else(|| usage(&format!("{flag} needs TIME:HART")))?;
    let change = if flag == "--plug" { Hotplug::Add(hart) } else { Hotplug::Remove(hart) };
    Machine::new(1, 1).schedule(time, change).map_err(|e| usage(&e))?;
    hotplug.push((time, change));
    Ok(())
}

/// --disk FILE and --disk-blocks N, gathered while parsing.
#[derive(Default)]
struct DiskArgs {
    path: Option<String>,
    blocks: Option<i64>,
}

impl DiskArgs {
    fn option(&mut self, flag: &str, value: Option<String>) -> Result<(), Fail> {
        if flag == "--disk" {
            self.path = Some(value.ok_or_else(|| usage("--disk needs a file"))?);
            return Ok(());
        }
        let blocks: i64 = number(flag, value)?;
        if blocks < 1 {
            return Err(usage("--disk-blocks takes 1 or more"));
        }
        self.blocks = Some(blocks);
        Ok(())
    }

    /// The disk, checked when it opens; --disk-blocks alone is a mistake.
    fn spec(self) -> Result<Option<DiskSpec>, Fail> {
        match (self.path, self.blocks) {
            (None, Some(_)) => Err(usage("--disk-blocks needs --disk")),
            (path, blocks) => Ok(path.map(|path| DiskSpec { path, blocks })),
        }
    }
}

/// The text --input names, to queue on the console before the first
/// step; `-` is stdin.
fn read_input(path: &str) -> Result<String, Fail> {
    let data = if path == "-" {
        let mut data = Vec::new();
        std::io::Read::read_to_end(&mut std::io::stdin(), &mut data)
            .map_err(|e| Fail::new(format!("<stdin>: {e}"), NO_INPUT))?;
        data
    } else {
        std::fs::read(path).map_err(|e| Fail::new(format!("{path}: {}", strerror(&e)), NO_INPUT))?
    };
    String::from_utf8(data).map_err(|_| Fail::new(format!("{path}: not UTF-8 text"), BAD_INPUT))
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
    let (mut file, mut output, mut listing, mut mix) = (None, None, false, false);
    let mut args = args;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" | "--output" => {
                output = Some(args.next().ok_or_else(|| usage("-o needs a file"))?)
            }
            "--listing" => listing = true,
            "--mix" => mix = true,
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
        if !listing && !mix {
            return Ok(0);
        }
    }
    let Loaded::Source(image) = loaded else {
        return if mix {
            Err(Fail::new(format!("{file}: --mix needs source"), BAD_INPUT))
        } else {
            Ok(0)
        };
    };
    let mut stdout = std::io::stdout().lock();
    if mix {
        let _ = stdout
            .write_all(mix::of_listing(&image.listing, image.labels.values().copied()).as_bytes());
        if !listing {
            return Ok(0);
        }
    }
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
        machine.schedule(time, change).expect("checked when parsed");
    }
    for (addr, tryte) in load(&args.files)?.trytes {
        machine.bus.poke(addr, tryte);
    }
    if let Some(path) = &args.input {
        machine.bus.console.feed(&read_input(path)?);
    }
    // Without a live console nothing can type into a run: once its input
    // is read, no more comes. A live one runs until it powers off.
    let mut live = None;
    if args.live {
        live = Some(live::Live::start(std::io::stdin().is_terminal()));
    } else {
        machine.bus.console.closed = true;
    }
    let max_steps = args.max_steps.unwrap_or(if args.live { u64::MAX } else { 10_000_000 });
    if let Some(spec) = &args.disk {
        machine.bus.block.disk = Some(Box::new(spec.open()?));
    }
    // With more than one hart, trace lines say which ran each instruction.
    let several = args.harts > 1 || !args.hotplug.is_empty();
    if args.trace || args.mix {
        machine.record_events(true);
    }
    let mut mix = mix::Mix::default();
    let mut stdout = std::io::stdout();
    let (mut done, mut printed) = (0u64, 0usize);
    let result = loop {
        if let Some(live) = &mut live {
            // A machine waiting on the outside waits a little for a key,
            // rather than spin.
            let stalled = machine.take_stalled();
            let console = &mut machine.bus.console;
            let waiting = stalled && console.input.is_empty() && !console.closed;
            let typed = live.take(if waiting { LIVE_WAIT } else { Duration::ZERO });
            console.feed(&typed.text);
            console.closed |= typed.closed;
            if typed.stop {
                break Err(Fail::after_output("stopped".to_string(), STOPPED));
            }
        }
        // Tracing runs one step at a time, printing what each executed.
        let size = if live.is_some() { LIVE_CHUNK } else { CHUNK };
        let chunk = if args.trace { 1 } else { size.min(max_steps - done) };
        let rings: Vec<(i64, i64)> = machine.harts().map(|(h, hart)| (h, hart.ring)).collect();
        // A live run stops as soon as the machine waits on the outside.
        let (outcome, ran) = if live.is_some() {
            machine.run_until_stalled(chunk)
        } else {
            (machine.run(chunk), chunk)
        };
        for (h, event) in machine.take_events() {
            if let Event::Execute { pc, word } = event {
                mix.add(word);
                if !args.trace {
                    continue;
                }
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
                break Ok((code & 0xFF) as u8);
            }
            Outcome::StepLimit => {
                done += ran;
                if done >= max_steps {
                    let message = format!("no power-off after {done} steps");
                    break Err(Fail::after_output(message, STEP_LIMIT));
                }
            }
            Outcome::Unhandled { hart, cause, tval, pc, ring } => {
                let mut what =
                    format!("cause {} at pc {pc} in ring {}, tval {tval}", sign(cause), sign(ring));
                if hart != 0 {
                    what += &format!(" on hart {hart}");
                }
                break Err(Fail::after_output(format!("unhandled trap: {what}"), UNHANDLED_TRAP));
            }
            Outcome::Unsupported(what) => {
                let message = format!("not implemented yet: {what}");
                break Err(Fail::after_output(message, NOT_IMPLEMENTED));
            }
        }
    };
    // However the run ended, what ran is worth seeing.
    if args.mix {
        eprint!("\n{}", mix.report("executed"));
    }
    result
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
    let (mut harts, mut hotplug, mut input) = (1, Vec::new(), None);
    let mut disk = DiskArgs::default();
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
            "--input" => input = Some(args.next().ok_or_else(|| usage("--input needs a file"))?),
            "--harts" | "--plug" | "--unplug" => {
                hart_option(&arg, args.next(), &mut harts, &mut hotplug)?
            }
            "--disk" | "--disk-blocks" => disk.option(&arg, args.next())?,
            flag if flag.starts_with('-') && flag != "-" => {
                return Err(usage(&format!("unknown option {flag}")));
            }
            _ => files.push(arg),
        }
    }
    let disk = disk.spec()?;
    if files.is_empty() {
        return Err(usage("debug needs at least one file"));
    }
    if input.as_deref() == Some("-") {
        return Err(usage("debug reads its commands from stdin; give --input a file"));
    }
    let program = load(&files)?;
    let input = input.as_deref().map(read_input).transpose()?.unwrap_or_default();
    if let Some(spec) = &disk {
        spec.open()?; // a bad disk is a startup error, not a debugger message
    }
    let mut shell = debug::shell::Shell {
        dbg: debug::engine::Debugger::new(program, ram_pages, harts, hotplug, input, disk),
    };
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
