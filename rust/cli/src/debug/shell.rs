//! The interactive debugger: a command shell over the engine. It copies
//! the Python debugger it replaced, down to how Python's `cmd.Cmd` parses
//! a line and prints help, so its sessions read as they always did.

use std::collections::BTreeSet;
use std::io::{BufRead, Write};

use detrytus_asm::pyrepr::repr;
use detrytus_asm::text::{is_space, strip};
use detrytus_core::Hotplug;
use detrytus_core::harts::{self, ABSENT, RUNNING, STOPPED};
use detrytus_core::isa;
use detrytus_core::trits::low;

use super::engine::{DEFAULT_LIMIT, Debugger, Stop, show};

pub const INTRO: &str = "detrytus debugger. 'help' lists commands; 'help CMD' explains one.";
const PROMPT: &str = "(dt) ";

const ALIASES: [(&str, &str); 10] = [
    ("s", "step"),
    ("n", "next"),
    ("c", "continue"),
    ("b", "break"),
    ("d", "delete"),
    ("w", "watch"),
    ("p", "print"),
    ("x", "examine"),
    ("q", "quit"),
    ("i", "info"),
];

const CAUSES: [(&str, i64); 9] = [
    ("illegal", isa::ILLEGAL),
    ("misaligned", isa::MISALIGNED),
    ("access", isa::ACCESS_FAULT),
    ("syscall", isa::SYSCALL),
    ("breakpoint", isa::BREAKPOINT),
    ("fetch-fault", isa::INSTR_PAGE_FAULT),
    ("load-fault", isa::LOAD_PAGE_FAULT),
    ("store-fault", isa::STORE_PAGE_FAULT),
    ("timer", -1),
];

/// Each command's help, as `inspect.cleandoc` leaves the Python
/// docstring; `help CMD` prints it.
const HELP: [(&str, &str); 24] = [
    ("EOF", "quit: leave the debugger. Alias: q"),
    (
        "break",
        "break ADDR: stop before running the instruction at ADDR. Alias: b\n\nADDR is an expression: a label, a number, $ra, loop+6 and so on.",
    ),
    (
        "catch",
        "catch [all|off|CAUSE ...]: stop when a trap is taken.\n\nCAUSE is a number such as 4 or -1, or one of: illegal, misaligned,\naccess, syscall, breakpoint, fetch-fault, load-fault, store-fault,\ntimer.",
    ),
    (
        "continue",
        "continue: run to a breakpoint, watchpoint, caught trap or power-off.\n\nAlias: c",
    ),
    ("csr", "csr NAME|NUMBER: show a CSR, read as ring +1 would."),
    ("delete", "delete [ID]: remove a breakpoint or watchpoint, or all of them. Alias: d"),
    ("disas", "disas [ADDR] [N]: disassemble N words from ADDR (default: the pc, 8)."),
    (
        "eof",
        "eof: close the console's input.\n\nOnce what is queued is read, console in reads CONSOLE_CLOSED (-2)\ninstead of CONSOLE_EMPTY (-1): no more input will come. restart opens\nit again.",
    ),
    ("examine", "examine ADDR [N]: show N words from ADDR (default 4). Alias: x"),
    (
        "hart",
        "hart [N]: list the harts, or look at hart N.\n\nregs, csr, print, examine, disas and next then use hart N. A step\nruns every running hart one instruction, whichever is in focus.",
    ),
    ("help", "List available commands with \"help\" or detailed help with \"help cmd\"."),
    ("info", "info: list breakpoints, watchpoints and caught traps. Alias: i"),
    (
        "input",
        "input TEXT: queue TEXT on the console's input, after what is left.\n\n\\n is a newline, \\t a tab, \\s a space and \\\\ a backslash; nothing\nelse is added, so end a line with \\n. restart drops this text.",
    ),
    ("next", "next: step, running a whole call as one step. Alias: n"),
    (
        "plug",
        "plug HART [TIME]: plug a hart in at TIME (default: the next step).\n\nThe hart arrives stopped, and the hart controller announces it.",
    ),
    ("print", "print EXPR: evaluate, with $name for registers. Alias: p"),
    ("quit", "quit: leave the debugger. Alias: q"),
    ("regs", "regs: show every register, the pc and the ring."),
    ("restart", "restart: reload the program on a fresh machine; breakpoints stay."),
    ("step", "step [N]: run N instructions (default 1). Alias: s"),
    ("trace", "trace on|off: print every instruction as it runs."),
    (
        "unplug",
        "unplug HART [TIME]: ask for a hart to go at TIME (default: the next step).\n\nThe hart controller asks the program; the hart goes once it stops.",
    ),
    (
        "watch",
        "watch ADDR [r|w|rw]: stop after an access to ADDR (default w). Alias: w\n\nAn aligned ADDR watches the word centred on it, any other one tryte.\nAddresses are virtual, as the program sees them.",
    ),
    (
        "xp",
        "xp ADDR [N]: show N words of physical memory from ADDR (default 4).\n\nUnlike examine, ADDR is not translated: it reads RAM and the boot\npage as they are, whatever ring and page table are current.",
    ),
];

/// `help` with no topic: cmd.Cmd's columns of documented commands.
const HELP_LIST: &str = "\nDocumented commands (type help <topic>):\n========================================\nEOF    continue  disas    hart  input  print  restart  unplug\nbreak  csr       eof      help  next   quit   step     watch \ncatch  delete    examine  info  plug   regs   trace    xp    \n\n";

fn format_trits(value: i64) -> String {
    let mut x = value;
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

/// The text of `input TEXT`, its escapes read. Space at either end is
/// trimmed with the rest of the line, so `\s` writes one there.
fn unescape(arg: &str) -> Result<String, String> {
    let mut text = String::new();
    let mut chars = arg.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            text.push(c);
            continue;
        }
        text.push(match chars.next() {
            Some('n') => '\n',
            Some('t') => '\t',
            Some('s') => ' ',
            Some('\\') => '\\',
            Some(other) => return Err(format!("unknown escape \\{other}")),
            None => return Err("a lone \\ at the end".into()),
        });
    }
    Ok(text)
}

/// Python's sorted(set) as a list repr: "[-1, 4]".
fn list_repr(set: &BTreeSet<i64>) -> String {
    let items: Vec<String> = set.iter().map(i64::to_string).collect();
    format!("[{}]", items.join(", "))
}

/// Python's int(text) for a cause number, with its error message.
fn python_int(text: &str) -> Result<i64, String> {
    let t = strip(text);
    let digits = t.strip_prefix(['+', '-']).unwrap_or(t);
    let ok = !digits.is_empty()
        && !digits.starts_with('_')
        && !digits.ends_with('_')
        && !digits.contains("__")
        && digits.chars().all(|c| c.is_ascii_digit() || c == '_');
    let invalid = || format!("invalid literal for int() with base 10: {}", repr(text));
    if !ok {
        return Err(invalid());
    }
    t.replace('_', "").parse().map_err(|_| invalid())
}

fn is_digits(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| c.is_ascii_digit())
}

pub struct Shell {
    pub dbg: Debugger,
}

type Done = Result<bool, String>;

impl Shell {
    fn say(&self, text: &str) {
        println!("{text}");
    }

    /// Expand an alias in the first word, as precmd does.
    pub fn precmd(line: &str) -> String {
        let line = strip(line);
        let (word, rest) = line.split_once(' ').unwrap_or((line, ""));
        let word = ALIASES.iter().find(|(a, _)| *a == word).map_or(word, |(_, c)| *c);
        strip(&format!("{word} {rest}")).to_string()
    }

    /// One command line, as cmd.Cmd.onecmd runs it: true to leave.
    pub fn onecmd(&mut self, line: &str) -> bool {
        let line = strip(line);
        if line.is_empty() {
            return false; // never repeat the last command: stepping by accident hurts
        }
        let line = match line.strip_prefix('?') {
            Some(rest) => format!("help {rest}"),
            None => line.to_string(),
        };
        let end =
            line.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).unwrap_or(line.len());
        let (cmd, arg) = (&line[..end], strip(&line[end..]));
        match self.dispatch(cmd, arg, &line) {
            Ok(done) => done,
            Err(message) => {
                self.say(&format!("error: {message}"));
                false
            }
        }
    }

    fn dispatch(&mut self, cmd: &str, arg: &str, line: &str) -> Done {
        match cmd {
            "step" => self.report(|d| Ok(d.step(Self::count(arg)?))),
            "next" => self.report(|d| Ok(d.next())),
            "continue" => self.report(|d| Ok(d.cont(DEFAULT_LIMIT))),
            "restart" => {
                self.dbg.restart();
                self.show_current();
                Ok(false)
            }
            "break" => self.do_break(arg),
            "watch" => self.do_watch(arg),
            "catch" => self.do_catch(arg),
            "delete" => {
                let id = if strip(arg).is_empty() { None } else { Some(Self::count(arg)?) };
                self.dbg.delete(id).map(|()| false)
            }
            "info" => self.do_info(),
            "hart" => self.do_hart(arg),
            "plug" => self.do_plug(arg, Hotplug::Add),
            "unplug" => self.do_plug(arg, Hotplug::Remove),
            "input" => {
                let text = unescape(arg)?;
                self.dbg.machine.bus.console.feed(&text);
                Ok(false)
            }
            "eof" => {
                self.dbg.machine.bus.console.closed = true;
                Ok(false)
            }
            "regs" => self.do_regs(),
            "csr" => self.do_csr(arg),
            "print" => {
                let value = self.dbg.evaluate(arg)?;
                self.say(&format!("{value}  0t{}", format_trits(value)));
                Ok(false)
            }
            "examine" => self.do_examine(arg),
            "disas" => self.do_disas(arg),
            "xp" => self.do_xp(arg),
            "trace" => {
                self.dbg.trace = match strip(arg) {
                    "on" => true,
                    "off" => false,
                    _ => return Err("trace on or trace off".into()),
                };
                Ok(false)
            }
            "help" => {
                self.do_help(arg);
                Ok(false)
            }
            "quit" | "EOF" => Ok(true),
            _ => {
                let first = line.split(is_space).find(|w| !w.is_empty()).unwrap_or("");
                self.say(&format!("unknown command {}; 'help' lists them", repr(first)));
                Ok(false)
            }
        }
    }

    fn count(arg: &str) -> Result<u64, String> {
        let a = strip(arg);
        if a.is_empty() {
            return Ok(1);
        }
        if !is_digits(a) {
            return Err(format!("expected a count, got {}", repr(a)));
        }
        Ok(a.parse().unwrap_or(u64::MAX))
    }

    fn report(&mut self, run: impl FnOnce(&mut Debugger) -> Result<Stop, String>) -> Done {
        let stop = run(&mut self.dbg)?;
        if !stop.message.is_empty() {
            if stop.reason == "step" {
                self.say(&stop.message);
            } else {
                self.say(&format!("stopped: {}", stop.message));
            }
        }
        if self.dbg.halted.is_none() {
            self.show_current();
        }
        Ok(false)
    }

    pub fn show_current(&mut self) {
        let (pc, ring) = (self.dbg.hart().pc, self.dbg.hart().ring);
        let text = self.dbg.instruction(pc);
        let mut place = format!("ring {ring:+}");
        if self.dbg.several() {
            let state = match self.dbg.machine.bus.harts.state(self.dbg.focus) {
                RUNNING => "",
                _ => " stopped",
            };
            place = format!("hart {}{state}, {place}", self.dbg.focus);
        }
        self.say(&format!("=> {}  [{place}]  {text}", self.dbg.where_(pc)));
    }

    /// `hart`: every present hart; `hart N`: look at hart N.
    fn do_hart(&mut self, arg: &str) -> Done {
        if strip(arg).is_empty() {
            for h in -harts::MAX_HART..=harts::MAX_HART {
                let state = match self.dbg.machine.bus.harts.state(h) {
                    ABSENT => continue,
                    STOPPED => "stopped",
                    _ => "running",
                };
                let mark = if h == self.dbg.focus { '*' } else { ' ' };
                let mut line = format!("{mark} {h:>4}  {state}");
                if let Some(hart) = self.dbg.machine.hart(h) {
                    let wait = match (hart.waiting(), hart.watching()) {
                        _ if state != "running" => "",
                        (true, true) => ", in wrs",
                        (true, false) => ", in wfi",
                        _ => "",
                    };
                    let (pc, ring) = (hart.pc, hart.ring);
                    line += &format!("  pc {}  ring {ring:+}{wait}", self.dbg.where_(pc));
                }
                self.say(&line);
            }
            return Ok(false);
        }
        let h = self.dbg.evaluate(arg)?;
        if self.dbg.machine.hart(h).is_none() {
            let why = match self.dbg.machine.bus.harts.state(h) {
                ABSENT => "is not present",
                _ => "has not run yet",
            };
            return Err(format!("hart {h} {why}"));
        }
        self.dbg.focus = h;
        self.show_current();
        Ok(false)
    }

    /// `plug HART [TIME]` and `unplug HART [TIME]`.
    fn do_plug(&mut self, arg: &str, change: fn(i64) -> Hotplug) -> Done {
        let words: Vec<&str> = arg.split(is_space).filter(|w| !w.is_empty()).collect();
        let (hart, time) = match words.as_slice() {
            [h] => (self.dbg.evaluate(h)?, None),
            [h, t] => (self.dbg.evaluate(h)?, Some(self.dbg.evaluate(t)?)),
            _ => return Err("give a hart, and optionally a time".into()),
        };
        let change = change(hart);
        let time = self.dbg.hotplug(change, time)?;
        let what = match change {
            Hotplug::Add(_) => "plugged in",
            Hotplug::Remove(_) => "asked to go",
        };
        self.say(&format!("hart {hart} will be {what} at time {time}"));
        Ok(false)
    }

    pub fn do_break(&mut self, arg: &str) -> Done {
        if strip(arg).is_empty() {
            return Err("break needs an address".into());
        }
        let (id, addr) = self.dbg.add_breakpoint(arg)?;
        self.say(&format!("breakpoint {id} at {}", self.dbg.where_(addr)));
        Ok(false)
    }

    fn do_watch(&mut self, arg: &str) -> Done {
        let mut parts: Vec<&str> = arg.split(is_space).filter(|w| !w.is_empty()).collect();
        if parts.is_empty() {
            return Err("watch needs an address".into());
        }
        let kinds = if parts.len() > 1 && matches!(*parts.last().unwrap(), "r" | "w" | "rw") {
            parts.pop().unwrap()
        } else {
            "w"
        };
        let id = self.dbg.add_watch(&parts.join(" "), kinds)?;
        let w = &self.dbg.watches[&id];
        self.say(&format!(
            "watchpoint {id} on {}..{} ({kinds}), now {}",
            w.first,
            w.last,
            show(w.value)
        ));
        Ok(false)
    }

    fn do_catch(&mut self, arg: &str) -> Done {
        let words: Vec<&str> = arg.split(is_space).filter(|w| !w.is_empty()).collect();
        if words.is_empty() || words == ["all"] {
            self.dbg.catch = Some(BTreeSet::new());
            self.say("catching every trap");
        } else if words == ["off"] {
            self.dbg.catch = None;
            self.say("not catching traps");
        } else {
            let mut causes = BTreeSet::new();
            for w in words {
                match CAUSES.iter().find(|(n, _)| *n == w) {
                    Some((_, c)) => causes.insert(*c),
                    None => causes.insert(python_int(w)?),
                };
            }
            let mut catch = self.dbg.catch.take().unwrap_or_default();
            catch.extend(causes);
            self.say(&format!("catching trap causes {}", list_repr(&catch)));
            self.dbg.catch = Some(catch);
        }
        Ok(false)
    }

    fn do_info(&mut self) -> Done {
        for (id, addr) in &self.dbg.breakpoints {
            if *id != 0 {
                self.say(&format!("  {id}: break at {}", self.dbg.where_(*addr)));
            }
        }
        for (id, w) in &self.dbg.watches {
            self.say(&format!("  {id}: watch {}..{} ({})", w.first, w.last, w.kinds));
        }
        let catch = match &self.dbg.catch {
            None => "off".to_string(),
            Some(c) if c.is_empty() => "all".to_string(),
            Some(c) => list_repr(c),
        };
        self.say(&format!("  catch: {catch}"));
        for (time, change) in &self.dbg.hotplug {
            let (verb, h) = match change {
                Hotplug::Add(h) => ("plug", h),
                Hotplug::Remove(h) => ("unplug", h),
            };
            self.say(&format!("  {verb} hart {h} at time {time}"));
        }
        Ok(false)
    }

    fn do_regs(&mut self) -> Done {
        // Arguments and temporaries, then saved registers, then gp, tp, ra, sp.
        let order: Vec<i64> = (1..=13).chain((-13..=-1).rev()).collect();
        let hart = self.dbg.hart();
        let cells: Vec<String> = order
            .iter()
            .map(|&r| {
                let name = isa::abi_name(r).unwrap_or_else(|| format!("r{r}"));
                format!("{name:>4} {:>15}", hart.x(r))
            })
            .collect();
        for row in cells.chunks(3) {
            self.say(&row.join("   "));
        }
        let (pc, ring) = (hart.pc, hart.ring);
        self.say(&format!("  pc {}   ring {ring:+}", self.dbg.where_(pc)));
        Ok(false)
    }

    fn do_csr(&mut self, arg: &str) -> Done {
        let name = strip(arg).to_lowercase();
        if name.is_empty() {
            return Err("csr needs a name, such as mtcause".into());
        }
        let number = match isa::csr_by_name(&name) {
            Some(n) => n,
            None => self.dbg.evaluate(&name)?,
        };
        let value =
            self.dbg.hart_mut().csr_by_number(number).ok_or_else(|| format!("no CSR {number}"))?;
        let shown = isa::csr_name(number).unwrap_or_else(|| number.to_string());
        self.say(&format!("{shown} = {value}  0t{}", format_trits(value)));
        Ok(false)
    }

    fn addr_and_count(arg: &str, default: usize) -> (String, usize) {
        let parts: Vec<&str> = arg.split(is_space).filter(|w| !w.is_empty()).collect();
        if parts.len() > 1 && is_digits(parts[parts.len() - 1]) {
            let n = parts[parts.len() - 1].parse().unwrap_or(usize::MAX);
            return (parts[..parts.len() - 1].join(" "), n);
        }
        (parts.join(" "), default)
    }

    fn do_examine(&mut self, arg: &str) -> Done {
        let (expr, count) = Self::addr_and_count(arg, 4);
        let addr = self.dbg.evaluate(&expr)?;
        for i in 0..count as i64 {
            let a = addr + 3 * i;
            let shown = match self.dbg.peek(a, 3) {
                None => "<unmapped>".to_string(),
                Some(w) => format!("{w:>15}  0t{}", format_trits(w)),
            };
            self.say(&format!("{:>24}  {shown}", self.dbg.where_(a)));
        }
        Ok(false)
    }

    /// examine, but of physical memory: no translation.
    fn do_xp(&mut self, arg: &str) -> Done {
        let (expr, count) = Self::addr_and_count(arg, 4);
        let addr = self.dbg.evaluate(&expr)?;
        for i in 0..count as i64 {
            let a = addr + 3 * i;
            let shown = match self.dbg.machine.bus.peek(a, 3) {
                None => "<unmapped>".to_string(),
                Some(w) => format!("{w:>15}  0t{}", format_trits(w)),
            };
            self.say(&format!("{a:>24}  {shown}"));
        }
        Ok(false)
    }

    fn do_disas(&mut self, arg: &str) -> Done {
        let (expr, count) = Self::addr_and_count(arg, 8);
        let pc = self.dbg.hart().pc;
        let addr = if expr.is_empty() { pc } else { self.dbg.evaluate(&expr)? };
        for (a, text) in self.dbg.disassemble(addr, count) {
            let mark = if a == pc { "=>" } else { "  " };
            self.say(&format!("{mark} {:>24}  {text}", self.dbg.where_(a)));
        }
        Ok(false)
    }

    fn do_help(&self, arg: &str) {
        if arg.is_empty() {
            print!("{HELP_LIST}");
            return;
        }
        match HELP.iter().find(|(n, _)| *n == arg) {
            Some((_, doc)) => self.say(doc),
            None => self.say(&format!("*** No help on {arg}")),
        }
    }

    /// Read commands from stdin until quit or end of input, as cmdloop.
    pub fn cmdloop(&mut self) {
        let stdin = std::io::stdin();
        let mut lines = stdin.lock().lines();
        loop {
            print!("{PROMPT}");
            let _ = std::io::stdout().flush();
            let line = match lines.next() {
                Some(Ok(line)) => line,
                _ => "EOF".to_string(),
            };
            if self.onecmd(&Self::precmd(&line)) {
                return;
            }
        }
    }
}
