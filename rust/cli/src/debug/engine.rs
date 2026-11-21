//! The debugger's engine: run control, breakpoints, watchpoints,
//! inspection. Ported from the original Python tools.
//!
//! It drives a Machine one step at a time, with the harts recording what
//! each step did, and decides when to stop. Program output and trace
//! lines go straight to stdout, in the order they happen.
//!
//! A step is a round: every running hart runs one instruction. One hart
//! is in focus, the one the inspection commands look at; a breakpoint,
//! watchpoint or caught trap on any hart stops the run and moves the
//! focus there.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::Write;

use detrytus_asm::disasm::disassemble;
use detrytus_asm::expr::{Symbols, evaluate};
use detrytus_core::hart::{AccessKind, Event, Hart};
use detrytus_core::isa;
use detrytus_core::{Hotplug, Machine, Outcome};

use crate::disk::DiskSpec;
use crate::program::Program;

pub const DEFAULT_LIMIT: u64 = 10_000_000;

/// Why execution stopped, for the user to read.
pub struct Stop {
    /// "step", "breakpoint", "watchpoint", "trap", "halted", ...
    pub reason: &'static str,
    pub message: String,
}

impl Stop {
    fn new(reason: &'static str, message: impl Into<String>) -> Self {
        Stop { reason, message: message.into() }
    }

    fn step() -> Self {
        Stop::new("step", "")
    }
}

pub struct Watch {
    /// Lowest and highest tryte watched.
    pub first: i64,
    pub last: i64,
    /// Any of 'r' and 'w'.
    pub kinds: String,
    /// The last value seen, for old/new reports.
    pub value: Option<i64>,
}

impl Watch {
    fn hit(&self, addr: i64, width: u32, kind: &str) -> bool {
        let (lo, hi) = if width == 3 { (addr - 1, addr + 1) } else { (addr, addr) };
        lo <= self.last && hi >= self.first && kind.chars().any(|k| self.kinds.contains(k))
    }
}

/// Python's repr of an optional int, for watch values: None or a number.
pub fn show(value: Option<i64>) -> String {
    value.map_or_else(|| "None".to_string(), |v| v.to_string())
}

pub struct Debugger {
    program: Program,
    symbols: Symbols,
    ram_pages: i64,
    harts: i64,
    /// Hot-plug changes, kept so a restart makes them again.
    pub hotplug: Vec<(i64, Hotplug)>,
    /// The --input text, queued again on every restart.
    input: String,
    /// The --disk, opened again on every restart; what was written stays.
    disk: Option<DiskSpec>,
    /// The hart the inspection commands look at.
    pub focus: i64,
    pub machine: Machine,
    /// Id 0 is the temporary breakpoint of `next`.
    pub breakpoints: BTreeMap<u64, i64>,
    pub watches: BTreeMap<u64, Watch>,
    /// Trap causes to stop on; empty means all, None none.
    pub catch: Option<BTreeSet<i64>>,
    ids: u64,
    pub halted: Option<i64>,
    pub trace: bool,
    printed: usize,
}

impl Debugger {
    pub fn new(
        program: Program,
        ram_pages: i64,
        harts: i64,
        hotplug: Vec<(i64, Hotplug)>,
        input: String,
        disk: Option<DiskSpec>,
    ) -> Self {
        let symbols: Symbols = program.symbols.iter().cloned().collect::<HashMap<_, _>>();
        let mut dbg = Debugger {
            program,
            symbols,
            ram_pages,
            harts,
            hotplug,
            input,
            disk,
            focus: 0,
            machine: Machine::new(ram_pages, 1),
            breakpoints: BTreeMap::new(),
            watches: BTreeMap::new(),
            catch: None,
            ids: 0,
            halted: None,
            trace: false,
            printed: 0,
        };
        dbg.restart();
        dbg
    }

    // --- the machine -------------------------------------------------------

    /// A fresh machine with the program loaded and the --input text
    /// queued. Stopping points and hot-plug changes stay; text given
    /// with `input` does not.
    pub fn restart(&mut self) {
        self.machine = Machine::with_harts(self.ram_pages, 1, self.harts);
        for &(time, change) in &self.hotplug {
            self.machine.schedule(time, change).expect("checked when it was given");
        }
        self.machine.record_events(true);
        self.focus = 0;
        for (&addr, &tryte) in &self.program.trytes {
            self.machine.bus.poke(addr, tryte);
        }
        self.machine.bus.console.feed(&self.input);
        if let Some(spec) = &self.disk {
            match spec.open() {
                Ok(disk) => self.machine.bus.block.disk = Some(Box::new(disk)),
                Err(crate::Fail(message, _)) => eprintln!("{message}"),
            }
        }
        self.halted = None;
        self.printed = 0;
        let spans: Vec<(u64, i64, i64)> =
            self.watches.iter().map(|(id, w)| (*id, w.first, w.last)).collect();
        for (id, first, last) in spans {
            let value = self.watched_value(first, last);
            self.watches.get_mut(&id).unwrap().value = value;
        }
    }

    /// Whether there can be more than one hart: messages then say which.
    pub fn several(&self) -> bool {
        self.harts > 1 || !self.hotplug.is_empty()
    }

    /// " on hart N" when there can be several harts.
    fn on_hart(&self, h: i64) -> String {
        if self.several() { format!(" on hart {h}") } else { String::new() }
    }

    /// The hart in focus.
    pub fn hart(&self) -> &Hart {
        self.machine.hart(self.focus).unwrap_or_else(|| self.hart())
    }

    pub fn hart_mut(&mut self) -> &mut Hart {
        let focus = if self.machine.hart(self.focus).is_some() { self.focus } else { 0 };
        self.machine.hart_mut(focus).unwrap()
    }

    /// Plug a hart in, or ask for one to go, at `time` (default: the next
    /// step). Returns the time.
    pub fn hotplug(&mut self, change: Hotplug, time: Option<i64>) -> Result<i64, String> {
        let time = time.unwrap_or(self.machine.time + 1);
        self.machine.schedule(time, change)?;
        self.hotplug.push((time, change));
        Ok(time)
    }

    // --- run control -------------------------------------------------------

    /// Run count instructions, or fewer if something stops it.
    pub fn step(&mut self, count: u64) -> Stop {
        for _ in 0..count {
            if let Some(stop) = self.step_once() {
                return stop;
            }
        }
        Stop::step()
    }

    /// Run until a breakpoint, watchpoint, caught trap or power-off. The
    /// first instruction always runs, so continuing from a breakpoint
    /// does not stop on it again.
    pub fn cont(&mut self, limit: u64) -> Stop {
        for n in 0..limit {
            if n > 0
                && let Some(stop) = self.breakpoint_hit()
            {
                return stop;
            }
            if let Some(stop) = self.step_once() {
                return stop;
            }
        }
        Stop::new("limit", format!("still running after {limit} steps"))
    }

    /// Step, treating a call as one step: run until it returns.
    pub fn next(&mut self) -> Stop {
        let pc = self.hart().pc;
        let is_call = self
            .peek(pc, 3)
            .and_then(|w| isa::decode(w).ok())
            .is_some_and(|d| matches!(d.op.name, "jal" | "jalr") && d.rd != 0);
        if !is_call {
            return self.step(1);
        }
        let return_to = pc + 3;
        self.breakpoints.insert(0, return_to);
        let stop = self.cont(DEFAULT_LIMIT);
        self.breakpoints.remove(&0);
        if stop.reason == "breakpoint" && self.hart().pc == return_to {
            return Stop::step();
        }
        stop
    }

    fn flush_console(&mut self) {
        let output = &self.machine.bus.console.output;
        if output.len() > self.printed {
            let text: String = output[self.printed..]
                .iter()
                .map(|&c| char::from_u32(c).unwrap_or('\u{FFFD}'))
                .collect();
            let mut stdout = std::io::stdout();
            let _ = stdout.write_all(text.as_bytes()).and_then(|()| stdout.flush());
            self.printed = output.len();
        }
    }

    fn step_once(&mut self) -> Option<Stop> {
        if let Some(code) = self.halted {
            return Some(Stop::new("halted", format!("the machine has powered off (exit {code})")));
        }
        let outcome = self.machine.step();
        let events = self.machine.take_events();
        if self.trace {
            for &(h, event) in &events {
                if let Event::Execute { pc, word } = event {
                    let who = if self.several() { format!("[{h:>4}] ") } else { String::new() };
                    println!("   {who}{:>24}  {}", self.where_(pc), disassemble(word, Some(pc)));
                }
            }
        }
        self.flush_console();
        if self.machine.hart(self.focus).is_none() {
            self.focus = 0; // the hart in focus was removed
        }
        match outcome {
            Some(Outcome::Exit { code, .. }) => {
                self.halted = Some(code);
                Some(Stop::new("halted", format!("powered off with exit code {code}")))
            }
            Some(Outcome::Unhandled { hart, cause, tval, pc, ring }) => {
                self.halted = Some(-1);
                self.focus = hart;
                let on = self.on_hart(hart);
                Some(Stop::new(
                    "unhandled",
                    format!(
                        "unhandled trap: cause {cause:+} at pc {pc} in ring {ring:+}, tval {tval}{on}"
                    ),
                ))
            }
            Some(Outcome::Unsupported(what)) => {
                Some(Stop::new("unimplemented", format!("not implemented yet: {what}")))
            }
            Some(Outcome::StepLimit) => unreachable!("a single step has no limit"),
            None => self.caught_trap(&events).or_else(|| self.watch_hit(&events)),
        }
    }

    fn caught_trap(&mut self, events: &[(i64, Event)]) -> Option<Stop> {
        let catch = self.catch.as_ref()?;
        let (h, cause, tval, tpc) = events.iter().find_map(|&(h, e)| match e {
            Event::Trap { cause, tval, tpc } if catch.is_empty() || catch.contains(&cause) => {
                Some((h, cause, tval, tpc))
            }
            _ => None,
        })?;
        self.focus = h;
        let on = self.on_hart(h);
        Some(Stop::new(
            "trap",
            format!("trap cause {cause:+} at {}, tval {tval}{on}", self.where_(tpc)),
        ))
    }

    fn watch_hit(&mut self, events: &[(i64, Event)]) -> Option<Stop> {
        for &(h, event) in events {
            let Event::Access { addr, width, kind } = event else { continue };
            let kind = match kind {
                AccessKind::Read => "r",
                AccessKind::Write => "w",
                AccessKind::Update => "rw",
            };
            let hit = self
                .watches
                .iter()
                .find(|(_, w)| w.hit(addr, width, kind))
                .map(|(id, w)| (*id, w.first, w.last));
            if let Some((id, first, last)) = hit {
                self.focus = h;
                let new = self.watched_value(first, last);
                let watch = self.watches.get_mut(&id).unwrap();
                let old = std::mem::replace(&mut watch.value, new);
                let verb = match kind {
                    "r" => "read",
                    "w" => "write",
                    _ => "update",
                };
                let change = if kind == "r" {
                    String::new()
                } else {
                    format!(": {} -> {}", show(old), show(new))
                };
                let on = self.on_hart(h);
                return Some(Stop::new(
                    "watchpoint",
                    format!("watchpoint {id}: {verb} at {}{change}{on}", self.where_(addr)),
                ));
            }
        }
        None
    }

    /// A breakpoint at the pc of a hart about to run, the hart in focus
    /// first; the focus moves to it. The temporary breakpoint of `next`
    /// (id 0) is only for the hart in focus.
    fn breakpoint_hit(&mut self) -> Option<Stop> {
        let mut harts = self.machine.prepare_round();
        harts.sort_by_key(|&h| h != self.focus);
        for h in harts {
            let pc = self.machine.hart(h).unwrap().pc;
            let ids: Vec<u64> = self
                .breakpoints
                .iter()
                .filter(|(id, a)| **a == pc && (**id != 0 || h == self.focus))
                .map(|(id, _)| *id)
                .collect();
            if ids.is_empty() {
                continue;
            }
            self.focus = h;
            let label = ids
                .iter()
                .find(|id| **id != 0)
                .map_or_else(|| "returned".to_string(), |id| format!("breakpoint {id}"));
            let on = self.on_hart(h);
            return Some(Stop::new("breakpoint", format!("{label} at {}{on}", self.where_(pc))));
        }
        None
    }

    // --- breakpoints, watchpoints ------------------------------------------

    fn new_id(&mut self) -> u64 {
        self.ids += 1;
        self.ids
    }

    pub fn add_breakpoint(&mut self, expr: &str) -> Result<(u64, i64), String> {
        let addr = self.evaluate(expr)?;
        let id = self.new_id();
        self.breakpoints.insert(id, addr);
        Ok((id, addr))
    }

    /// Watch the word centred on an aligned address, else one tryte.
    pub fn add_watch(&mut self, expr: &str, kinds: &str) -> Result<u64, String> {
        if kinds.is_empty() || kinds.chars().any(|c| c != 'r' && c != 'w') {
            return Err("watch kinds are r, w or rw".into());
        }
        let addr = self.evaluate(expr)?;
        let span = i64::from(addr.rem_euclid(3) == 0);
        let (first, last) = (addr - span, addr + span);
        let value = self.watched_value(first, last);
        let watch = Watch { first, last, kinds: kinds.into(), value };
        let id = self.new_id();
        self.watches.insert(id, watch);
        Ok(id)
    }

    pub fn delete(&mut self, id: Option<u64>) -> Result<(), String> {
        match id {
            None => {
                self.breakpoints.retain(|k, _| *k == 0);
                self.watches.clear();
            }
            Some(id) => {
                if self.breakpoints.remove(&id).is_none() && self.watches.remove(&id).is_none() {
                    return Err(format!("no breakpoint or watchpoint {id}"));
                }
            }
        }
        Ok(())
    }

    // --- inspection ----------------------------------------------------------

    /// An assembler expression, where $name is a register and $pc the pc.
    pub fn evaluate(&self, text: &str) -> Result<i64, String> {
        let replaced = self.registers_in(text)?;
        let pc = self.hart().pc;
        let value = evaluate(&replaced, &self.symbols, Some(pc as i128))
            .map_err(|e| e.message().to_string())?;
        i64::try_from(value).map_err(|_| format!("{value} is too large"))
    }

    /// Replace $name as the pattern `\$(r-?\d+|[a-z][a-z0-9]*)` finds it.
    fn registers_in(&self, text: &str) -> Result<String, String> {
        let chars: Vec<char> = text.chars().collect();
        let mut out = String::new();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] != '$' {
                out.push(chars[i]);
                i += 1;
                continue;
            }
            let numbered = {
                let mut j = i + 1;
                if chars.get(j) == Some(&'r') {
                    j += 1;
                    if chars.get(j) == Some(&'-') {
                        j += 1;
                    }
                    let digits = chars[j..].iter().take_while(|c| c.is_ascii_digit()).count();
                    (digits > 0).then_some(j + digits)
                } else {
                    None
                }
            };
            let named = || {
                let first = chars.get(i + 1).is_some_and(|c| c.is_ascii_lowercase());
                first.then(|| {
                    i + 2
                        + chars[i + 2..]
                            .iter()
                            .take_while(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
                            .count()
                })
            };
            let Some(end) = numbered.or_else(named) else {
                out.push('$');
                i += 1;
                continue;
            };
            let name: String = chars[i + 1..end].iter().collect();
            if name == "pc" {
                out.push_str(&self.hart().pc.to_string());
            } else {
                let value = self.hart().reg(&name).ok_or_else(|| format!("no register ${name}"))?;
                out.push_str(&format!("({value})"));
            }
            i = end;
        }
        Ok(out)
    }

    /// addr, with the nearest label at or below it: '42 <loop+3>'.
    pub fn where_(&self, addr: i64) -> String {
        let mut best: Option<&(String, i64)> = None;
        for entry in &self.program.labels {
            if entry.1 <= addr && best.is_none_or(|b| entry.1 > b.1) {
                best = Some(entry);
            }
        }
        match best {
            None => addr.to_string(),
            Some((name, value)) if *value == addr => format!("{addr} <{name}>"),
            Some((name, value)) => format!("{addr} <{name}+{}>", addr - value),
        }
    }

    /// Memory as the current ring sees it, with no side effects.
    pub fn peek(&mut self, addr: i64, width: u32) -> Option<i64> {
        let physical = self.machine.probe(self.focus, addr)?;
        self.machine.bus.peek(physical, width)
    }

    /// The watched word (centred one past first) or tryte.
    fn watched_value(&mut self, first: i64, last: i64) -> Option<i64> {
        if last > first { self.peek(first + 1, 3) } else { self.peek(first, 1) }
    }

    pub fn disassemble(&mut self, addr: i64, count: usize) -> Vec<(i64, String)> {
        (0..count as i64)
            .map(|i| {
                let a = addr + 3 * i;
                let text = self
                    .peek(a, 3)
                    .map_or_else(|| "<unmapped>".to_string(), |w| disassemble(w, Some(a)));
                (a, text)
            })
            .collect()
    }
}
