//! The back end: a function's instructions on virtual registers, the
//! IR, into assembly.
//!
//! Code generation emits ordinary detrytus instructions whose operands
//! may be virtual registers (`%1`, `%2`, ...) as well as physical ones:
//! the argument registers around calls, `s0` and `sp` for memory. That
//! machine-level IR keeps the assembler's own vocabulary, so it reads
//! like the output, and it is what a later backend (an LLVM or Rust
//! one) would lower to as well. Here it is cleaned up and given
//! registers:
//!
//! 1. Within a block, values already computed are reused, and copies
//!    of values written once are read through.
//! 2. Instructions whose results nothing reads are removed.
//! 3. Liveness over the control flow graph gives the interference
//!    graph. Copies whose sides never interfere are merged (Briggs, so
//!    never at the cost of a spill), and the graph is coloured with the
//!    21 allocatable registers, a value preferring the register of its
//!    copy partner or operand. A value live across a call gets a saved
//!    register; one that finds no register is spilled to the frame and
//!    allocation runs again.
//! 4. Peepholes: branches over jumps, jumps to jumps, copies undone or
//!    to themselves, unreachable code and unused labels; then the frame
//!    is built around what was used.
//!
//! Small functions that call nothing are copied into their callers
//! before all this ([`inline`]).

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

/// Marks a line from an `asm` block: passed through untouched.
pub const RAW: &str = "\u{1}";

const ARGS: [&str; 6] = ["a0", "a1", "a2", "a3", "a4", "a5"];
const TEMPS: [&str; 7] = ["t0", "t1", "t2", "t3", "t4", "t5", "t6"];
const SAVED: [&str; 8] = ["s1", "s2", "s3", "s4", "s5", "s6", "s7", "s8"];

/// Three-register instructions: rd, rs1, rs2.
const ALU3: [&str; 23] = [
    "tlut", "add", "sub", "mul", "div", "rem", "sh", "tmin", "tmax", "min", "max", "cmp", "divr",
    "remr", "tmul", "tsum", "tcons", "tany", "band", "bor", "bxor", "bsh", "and",
];
/// Instructions of the form rd, rs1[, immediates].
const ALU2: [&str; 11] =
    ["mv", "addi", "shi", "cmpi", "tex", "neg", "tabs", "abs", "lzt", "tzt", "nzt"];
const BRANCH2: [&str; 6] = ["beq", "bne", "blt", "bge", "ble", "bgt"];
const BRANCH1: [&str; 6] = ["beqz", "bnez", "bltz", "bgez", "blez", "bgtz"];

thread_local! {
    static NAMES: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
}

/// The name of virtual register `n`, `%n`.
pub fn vreg(n: usize) -> &'static str {
    NAMES.with(|names| {
        let mut names = names.borrow_mut();
        while names.len() <= n {
            let k = names.len();
            names.push(Box::leak(format!("%{k}").into_boxed_str()));
        }
        names[n]
    })
}

/// A function's body, as code generation leaves it.
pub struct Func<'a> {
    pub name: &'a str,
    /// Words of result left in a0 (and a1) at the return: 0, 1 or 2.
    pub results: usize,
    /// Frame words code generation gave to locals in memory.
    pub slots: i128,
    /// Parameters past the sixth, which the callee reads from `s0`.
    pub stack_params: bool,
    pub body: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
enum Line {
    /// A comment or blank line.
    Text(String),
    Label(String),
    Raw(String),
    Op {
        op: String,
        args: Vec<String>,
    },
}

/// What an instruction reads and writes, among the registers the
/// allocator tracks (virtual ones, and `a0`..`a5`, `t0`..`t6`).
#[derive(Default)]
struct Effects {
    defs: Vec<String>,
    uses: Vec<String>,
    targets: Vec<String>,
    /// Whether control can go on to the next line.
    falls: bool,
    /// It must stay even when nothing reads its result.
    keep: bool,
    /// `mv rd, rs`: the two may share a register.
    copy: Option<(String, String)>,
}

fn tracked(r: &str) -> bool {
    r.starts_with('%') || ARGS.contains(&r) || TEMPS.contains(&r)
}

/// The register in `off(reg)`, if the operand is one.
fn base_of(arg: &str) -> Option<&str> {
    let open = arg.find('(')?;
    arg.ends_with(')').then(|| &arg[open + 1..arg.len() - 1])
}

fn clobbered() -> Vec<String> {
    ARGS.iter().chain(TEMPS.iter()).map(|r| r.to_string()).collect()
}

fn effects(line: &Line) -> Effects {
    let mut e = Effects { falls: true, ..Default::default() };
    let reg = |s: &str| tracked(s).then(|| s.to_string());
    match line {
        Line::Text(_) | Line::Label(_) => {}
        Line::Raw(_) => {
            e.uses = ARGS.iter().map(|r| r.to_string()).collect();
            e.defs = clobbered();
            e.keep = true;
        }
        Line::Op { op, args } => {
            let op = op.as_str();
            let arg = |i: usize| args.get(i).map(String::as_str).unwrap_or("");
            if ALU3.contains(&op) {
                e.defs.extend(reg(arg(0)));
                e.uses.extend(reg(arg(1)));
                e.uses.extend(reg(arg(2)));
            } else if ALU2.contains(&op) {
                e.defs.extend(reg(arg(0)));
                e.uses.extend(reg(arg(1)));
                if op == "mv" && tracked(arg(0)) && tracked(arg(1)) {
                    e.copy = Some((arg(0).to_string(), arg(1).to_string()));
                }
            } else if op == "tins" {
                // Changes only a field of rd, so reads it too.
                e.defs.extend(reg(arg(0)));
                e.uses.extend(reg(arg(0)));
                e.uses.extend(reg(arg(1)));
            } else if op == "li" || op == "la" {
                e.defs.extend(reg(arg(0)));
            } else if op == "lw" || op == "lt" {
                e.defs.extend(reg(arg(0)));
                e.uses.extend(base_of(arg(1)).and_then(reg));
                e.keep = true;
            } else if op == "sw" || op == "st" {
                e.uses.extend(reg(arg(0)));
                e.uses.extend(base_of(arg(1)).and_then(reg));
                e.keep = true;
            } else if BRANCH2.contains(&op) {
                e.uses.extend(reg(arg(0)));
                e.uses.extend(reg(arg(1)));
                e.targets.push(arg(2).to_string());
                e.keep = true;
            } else if BRANCH1.contains(&op) {
                e.uses.extend(reg(arg(0)));
                e.targets.push(arg(1).to_string());
                e.keep = true;
            } else if op == "br3" {
                e.uses.extend(reg(arg(0)));
                e.targets.push(arg(1).to_string());
                e.targets.push(arg(2).to_string());
                e.keep = true;
            } else if op == "j" {
                e.targets.push(arg(0).to_string());
                e.falls = false;
                e.keep = true;
            } else if op == "callr" {
                // `callr reg, n`: a call through a register (a function
                // pointer), with n arguments; it becomes `jalr ra, 0(reg)`.
                let n: usize = arg(1).parse().unwrap_or(ARGS.len());
                e.uses = ARGS[..n.min(ARGS.len())].iter().map(|r| r.to_string()).collect();
                e.uses.extend(reg(arg(0)));
                e.defs = clobbered();
                e.keep = true;
            } else if op == "call" || op == "sys" {
                let n: usize = args.last().and_then(|a| a.parse().ok()).unwrap_or(ARGS.len());
                e.uses = ARGS[..n.min(ARGS.len())].iter().map(|r| r.to_string()).collect();
                // A system call answers in argument registers and leaves
                // the rest as they were (trice.md 10).
                e.defs = if op == "sys" {
                    ARGS.iter().map(|r| r.to_string()).collect()
                } else {
                    clobbered()
                };
                e.keep = true;
            } else {
                // Anything else is taken to read and change everything a
                // call may.
                e.uses = ARGS.iter().map(|r| r.to_string()).collect();
                e.defs = clobbered();
                e.keep = true;
            }
            // Writes to registers the allocator does not track (sp)
            // must stay.
            if args.first().is_some_and(|a| !tracked(a) && a != "zero")
                && (ALU3.contains(&op) || ALU2.contains(&op))
            {
                e.keep = true;
            }
        }
    }
    e
}

fn parse(body: Vec<String>) -> Vec<Line> {
    body.into_iter()
        .map(|l| {
            if let Some(raw) = l.strip_prefix(RAW) {
                return Line::Raw(raw.to_string());
            }
            let t = l.trim();
            if t.is_empty() || t.starts_with('#') {
                Line::Text(l)
            } else if !l.starts_with(' ') && t.ends_with(':') {
                Line::Label(t[..t.len() - 1].to_string())
            } else {
                let (op, rest) = t.split_once(char::is_whitespace).unwrap_or((t, ""));
                let args = if rest.trim().is_empty() {
                    Vec::new()
                } else {
                    rest.split(',').map(|a| a.trim().to_string()).collect()
                };
                Line::Op { op: op.to_string(), args }
            }
        })
        .collect()
}

fn render(line: &Line) -> String {
    match line {
        Line::Text(t) => t.clone(),
        Line::Label(l) => format!("{l}:"),
        Line::Raw(r) => r.clone(),
        Line::Op { op, args } if op == "callr" => format!("        jalr  ra, 0({})", args[0]),
        Line::Op { op, args } => {
            // A call's argument count was for the allocator only.
            let args = if op == "call" || op == "sys" {
                &args[..args.len().saturating_sub(1)]
            } else {
                &args[..]
            };
            if args.is_empty() {
                format!("        {op}")
            } else {
                format!("        {op:<5} {}", args.join(", "))
            }
        }
    }
}

/// Rename register `from` to `to` wherever it appears in an operand.
fn rename(line: &mut Line, from: &str, to: &str) {
    if let Line::Op { args, .. } = line {
        for a in args.iter_mut() {
            if a == from {
                *a = to.to_string();
            } else if let Some(b) = base_of(a)
                && b == from
            {
                let open = a.find('(').unwrap();
                *a = format!("{}({to})", &a[..open]);
            }
        }
    }
}

/// Liveness over the control flow graph: the registers live after
/// each line.
fn live_after(lines: &[Line], returns: usize) -> Vec<BTreeSet<String>> {
    let fx: Vec<Effects> = lines.iter().map(effects).collect();
    // Blocks: a label starts one, a branch or jump ends one.
    let mut starts = vec![0];
    for (i, line) in lines.iter().enumerate() {
        if matches!(line, Line::Label(_)) && i > 0 {
            starts.push(i);
        }
        if (!fx[i].targets.is_empty() || !fx[i].falls) && i + 1 < lines.len() {
            starts.push(i + 1);
        }
    }
    starts.sort();
    starts.dedup();
    let blocks: Vec<(usize, usize)> = starts
        .iter()
        .enumerate()
        .map(|(k, &s)| (s, starts.get(k + 1).copied().unwrap_or(lines.len())))
        .collect();
    let mut at_label = BTreeMap::new();
    for (b, &(s, e)) in blocks.iter().enumerate() {
        for line in &lines[s..e] {
            match line {
                Line::Label(l) => {
                    at_label.insert(l.clone(), b);
                }
                Line::Text(_) => {}
                _ => break,
            }
        }
    }
    let exit = blocks.len();
    let succs: Vec<Vec<usize>> = blocks
        .iter()
        .enumerate()
        .map(|(b, &(s, e))| {
            let mut out = Vec::new();
            let last = (s..e).rev().find(|&i| matches!(lines[i], Line::Op { .. } | Line::Raw(_)));
            let mut falls = true;
            if let Some(i) = last {
                for t in &fx[i].targets {
                    out.push(at_label.get(t).copied().unwrap_or(exit));
                }
                falls = fx[i].falls;
            }
            if falls {
                out.push(if b + 1 < blocks.len() { b + 1 } else { exit });
            }
            out
        })
        .collect();
    let exit_live: BTreeSet<String> = ARGS[..returns].iter().map(|r| r.to_string()).collect();
    let mut live_in = vec![BTreeSet::<String>::new(); blocks.len()];
    let mut live_out = vec![BTreeSet::<String>::new(); blocks.len()];
    loop {
        let mut changed = false;
        for b in (0..blocks.len()).rev() {
            let mut out = BTreeSet::new();
            for &s in &succs[b] {
                out.extend(if s == exit { exit_live.iter() } else { live_in[s].iter() }.cloned());
            }
            let mut live = out.clone();
            let (s, e) = blocks[b];
            for i in (s..e).rev() {
                for d in &fx[i].defs {
                    live.remove(d);
                }
                live.extend(fx[i].uses.iter().cloned());
            }
            if live != live_in[b] || out != live_out[b] {
                live_in[b] = live;
                live_out[b] = out;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let mut after = vec![BTreeSet::new(); lines.len()];
    for (b, &(s, e)) in blocks.iter().enumerate() {
        let mut live = live_out[b].clone();
        for i in (s..e).rev() {
            after[i] = live.clone();
            for d in &fx[i].defs {
                live.remove(d);
            }
            live.extend(fx[i].uses.iter().cloned());
        }
    }
    after
}

/// Remove instructions whose results nothing reads, until none is left.
fn dead_code(lines: &mut Vec<Line>, returns: usize) {
    loop {
        let after = live_after(lines, returns);
        let mut removed = false;
        let mut kept = Vec::with_capacity(lines.len());
        for (i, line) in lines.drain(..).enumerate() {
            let e = effects(&line);
            let dead = matches!(line, Line::Op { .. })
                && !e.keep
                && !e.defs.is_empty()
                && e.defs.iter().all(|d| d.starts_with('%') && !after[i].contains(d));
            if dead {
                removed = true;
            } else {
                kept.push(line);
            }
        }
        *lines = kept;
        if !removed {
            break;
        }
    }
}

/// How many instructions write each virtual register.
fn def_counts(lines: &[Line]) -> BTreeMap<String, usize> {
    let mut n = BTreeMap::new();
    for line in lines {
        for d in effects(line).defs {
            if d.starts_with('%') {
                *n.entry(d).or_insert(0) += 1;
            }
        }
    }
    n
}

/// Within a block, reuse a value already computed: a constant, an
/// address, a result of arithmetic on the same registers, or a global
/// loaded (or stored) since the last store. Then, for virtual registers
/// written once, read through copies.
///
/// Only loads of a global by name are reused, never loads through a
/// pointer, which may be a device's register.
fn reuse(lines: &mut [Line]) {
    let defs = def_counts(lines);
    let once = |r: &str| defs.get(r) == Some(&1);
    // What each register holds, as the instruction that computed it.
    let mut held: Vec<(String, String)> = Vec::new();
    for slot in lines.iter_mut() {
        let line = slot.clone();
        let e = effects(&line);
        let mut key = None;
        if let Line::Op { op, args } = &line
            && args.first().is_some_and(|d| d.starts_with('%'))
        {
            let pure = op == "li"
                || op == "la"
                || ((ALU3.contains(&op.as_str()) || ALU2.contains(&op.as_str())) && op != "mv");
            let global = (op == "lw" || op == "lt") && base_of(&args[1]).is_none();
            if (pure || global) && !args[1..].contains(&args[0]) {
                key = Some(format!("{op} {}", args[1..].join(",")));
            }
        }
        if let Some(k) = &key
            && let Some((_, prev)) = held.iter().find(|(h, _)| h == k)
        {
            let Line::Op { args, .. } = &line else { unreachable!() };
            *slot = Line::Op { op: "mv".into(), args: vec![args[0].clone(), prev.clone()] };
        }
        // Forget what this line changes. Control arriving from elsewhere
        // (a label) or a call ends the block; keeping a value over a call would cost a saved register,
        // more than computing it again.
        if matches!(line, Line::Label(_)) || e.defs.iter().any(|d| TEMPS.contains(&d.as_str())) {
            held.clear();
            continue;
        }
        let mentions = |k: &str, r: &str| k.split([' ', ',', '(', ')']).any(|w| w == r);
        for d in &e.defs {
            held.retain(|(k, v)| v != d && !mentions(k, d));
        }
        if let Line::Op { op, args } = &line
            && (op == "sw" || op == "st")
        {
            held.retain(|(k, _)| !k.starts_with("lw ") && !k.starts_with("lt "));
            // A word stored to a global is what loading it gives.
            if op == "sw" && base_of(&args[1]).is_none() && args[0].starts_with('%') {
                held.push((format!("lw {}", args[1]), args[0].clone()));
            }
        }
        if let Some(k) = key
            && matches!(&*slot, Line::Op { op, .. } if op != "mv")
        {
            let Line::Op { args, .. } = &line else { unreachable!() };
            held.push((k, args[0].clone()));
        }
    }
    // Copies: once `%b = %a` with both written once, %b is %a.
    let mut alias: BTreeMap<String, String> = BTreeMap::new();
    for line in lines.iter() {
        if let Line::Op { op, args } = line
            && op == "mv"
            && args[0].starts_with('%')
            && args[1].starts_with('%')
            && once(&args[0])
            && once(&args[1])
        {
            alias.insert(args[0].clone(), args[1].clone());
        }
    }
    let resolve = |mut r: String| {
        while let Some(a) = alias.get(&r) {
            r = a.clone();
        }
        r
    };
    for line in lines.iter_mut() {
        let e = effects(line);
        for u in e.uses {
            if alias.contains_key(&u) {
                let to = resolve(u.clone());
                // Only the operands read: the copy itself becomes a
                // copy to itself, removed as dead.
                if let Line::Op { args, .. } = line {
                    let first_is_def = !e.defs.is_empty();
                    for (k, a) in args.iter_mut().enumerate() {
                        if k == 0 && first_is_def {
                            continue;
                        }
                        if *a == u {
                            *a = to.clone();
                        } else if base_of(a) == Some(u.as_str()) {
                            let open = a.find('(').unwrap();
                            *a = format!("{}({to})", &a[..open]);
                        }
                    }
                }
            }
        }
    }
}

/// The registers values may be given, in order of preference: those a
/// call may change first, the saved ones (which cost a save) last.
fn colours() -> Vec<&'static str> {
    TEMPS.iter().chain(ARGS.iter().rev()).chain(SAVED.iter()).copied().collect()
}

type Graph = (BTreeMap<String, BTreeSet<String>>, Vec<(String, String)>);

/// The interference graph (each virtual register and the registers live
/// where it is written, both ways) and the copies between registers.
fn graph(lines: &[Line], returns: usize) -> Graph {
    let after = live_after(lines, returns);
    let mut adj: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut moves: Vec<(String, String)> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let e = effects(line);
        for r in e.defs.iter().chain(e.uses.iter()) {
            if r.starts_with('%') {
                adj.entry(r.clone()).or_default();
            }
        }
        if let Some(m) = &e.copy {
            moves.push(m.clone());
        }
        for d in &e.defs {
            for l in &after[i] {
                if l == d || e.copy.as_ref().is_some_and(|(_, src)| src == l) {
                    continue;
                }
                if d.starts_with('%') {
                    adj.entry(d.clone()).or_default().insert(l.clone());
                }
                if l.starts_with('%') {
                    adj.entry(l.clone()).or_default().insert(d.clone());
                }
            }
        }
    }
    (adj, moves)
}

/// Merge the two sides of a copy when they never hold different values
/// at once, so the copy disappears. Two virtual registers merge only
/// when the merged one still has fewer busy neighbours than there are
/// registers (Briggs), so merging never forces a spill; a virtual
/// register merges into a fixed one whenever they do not interfere.
fn coalesce(lines: &mut [Line], returns: usize) {
    let k = colours().len();
    loop {
        let (adj, moves) = graph(lines, returns);
        let pick = moves.iter().find_map(|(a, b)| {
            if a == b {
                return None;
            }
            let (keep, gone) = match (a.starts_with('%'), b.starts_with('%')) {
                (true, true) => (a, b),
                (true, false) => (b, a),
                (false, true) => (a, b),
                (false, false) => return None,
            };
            if !colours().contains(&keep.as_str()) && !keep.starts_with('%') {
                return None;
            }
            if adj[gone].contains(keep) || adj.get(keep).is_some_and(|n| n.contains(gone)) {
                return None;
            }
            if keep.starts_with('%') {
                let both: BTreeSet<&String> = adj[keep].iter().chain(adj[gone].iter()).collect();
                let busy = both
                    .iter()
                    .filter(|n| {
                        !n.starts_with('%') || adj.get(n.as_str()).is_none_or(|m| m.len() >= k)
                    })
                    .count();
                if busy >= k {
                    return None;
                }
            }
            Some((keep.clone(), gone.clone()))
        });
        let Some((keep, gone)) = pick else { return };
        for line in lines.iter_mut() {
            rename(line, &gone, &keep);
        }
    }
}

/// Colour the interference graph: a register for every virtual one, or
/// the virtual registers that must be spilled.
fn colour(lines: &[Line], returns: usize) -> Result<BTreeMap<String, String>, Vec<String>> {
    let (adj, moves) = graph(lines, returns);
    let palette = colours();
    let k = palette.len();
    // Simplify: take out a node with fewer neighbours than colours, or
    // else the busiest, optimistically.
    let mut degree: BTreeMap<String, usize> =
        adj.iter().map(|(v, n)| (v.clone(), n.len())).collect();
    let mut order = Vec::new();
    while !degree.is_empty() {
        let pick = degree
            .iter()
            .find(|(_, d)| **d < k)
            .or_else(|| degree.iter().max_by_key(|(_, d)| **d))
            .map(|(v, _)| v.clone())
            .unwrap();
        degree.remove(&pick);
        for n in &adj[&pick] {
            if let Some(d) = degree.get_mut(n) {
                *d -= 1;
            }
        }
        order.push(pick);
    }
    let mut colour: BTreeMap<String, String> = BTreeMap::new();
    let mut spills = Vec::new();
    let mut used_saved: BTreeSet<&str> = BTreeSet::new();
    for v in order.iter().rev() {
        let taken: BTreeSet<&str> = adj[v]
            .iter()
            .map(|n| {
                if n.starts_with('%') { colour.get(n).map(String::as_str).unwrap_or("") } else { n }
            })
            .collect();
        let free: Vec<&str> = palette.iter().copied().filter(|c| !taken.contains(c)).collect();
        if free.is_empty() {
            spills.push(v.clone());
            continue;
        }
        // Prefer the register of a value this one is copied from or to.
        let partner = moves.iter().find_map(|(a, b)| {
            let other = if a == v {
                b
            } else if b == v {
                a
            } else {
                return None;
            };
            let c = if other.starts_with('%') { colour.get(other)?.as_str() } else { other };
            free.iter().find(|f| **f == c).copied()
        });
        // Else the register of an operand of the instruction that writes
        // this one, or of the result of one reading it: `rd = rd op rs`
        // has a compressed form (isa.md 6.1), `rd = rs1 op rs2` none.
        let partner = partner.or_else(|| {
            lines.iter().find_map(|line| {
                let Line::Op { op, args } = line else { return None };
                let two =
                    ALU3.contains(&op.as_str()) || ["addi", "shi", "cmpi"].contains(&op.as_str());
                if !two || args.len() < 2 {
                    return None;
                }
                let swaps = ["add", "mul", "min", "max", "tmin", "tmax"].contains(&op.as_str());
                let mut others: Vec<&String> = Vec::new();
                if &args[0] == v {
                    others.push(&args[1]);
                    if swaps {
                        others.extend(args.get(2));
                    }
                } else if &args[1] == v || (swaps && args.get(2) == Some(v)) {
                    others.push(&args[0]);
                }
                others.into_iter().find_map(|o| {
                    let c = if o.starts_with('%') { colour.get(o)?.as_str() } else { o.as_str() };
                    free.iter().find(|f| **f == c).copied()
                })
            })
        });
        let choice = partner.unwrap_or_else(|| {
            if SAVED.contains(&free[0]) {
                free.iter().find(|c| used_saved.contains(*c)).copied().unwrap_or(free[0])
            } else {
                free[0]
            }
        });
        if SAVED.contains(&choice) {
            used_saved.insert(choice);
        }
        colour.insert(v.clone(), choice.to_string());
    }
    if spills.is_empty() { Ok(colour) } else { Err(spills) }
}

/// Keep `v` in the frame word at `off` from `s0`: load it before each
/// use, store it after each write, through a new short-lived register.
fn spill(lines: &mut Vec<Line>, v: &str, off: i128, fresh: &mut usize) {
    let mut out = Vec::with_capacity(lines.len());
    for mut line in lines.drain(..) {
        let e = effects(&line);
        let used = e.uses.iter().any(|u| u == v);
        let defined = e.defs.iter().any(|d| d == v);
        if !used && !defined {
            out.push(line);
            continue;
        }
        *fresh += 1;
        let t = format!("%spill{fresh}");
        rename(&mut line, v, &t);
        if used {
            out.push(Line::Op { op: "lw".into(), args: vec![t.clone(), format!("{off}(s0)")] });
        }
        out.push(line);
        if defined {
            out.push(Line::Op { op: "sw".into(), args: vec![t, format!("{off}(s0)")] });
        }
    }
    *lines = out;
}

/// Drop copies to themselves, jumps to the next line, branches that
/// go nowhere, and code no jump reaches.
fn tidy(lines: &mut Vec<Line>) {
    // A jump or branch to a jump goes straight to where that one goes.
    let mut onward: BTreeMap<String, String> = BTreeMap::new();
    for (i, line) in lines.iter().enumerate() {
        if let Line::Label(name) = line
            && let Some(Line::Op { op, args }) =
                lines[i + 1..].iter().find(|l| !matches!(l, Line::Text(_) | Line::Label(_)))
            && op == "j"
        {
            onward.insert(name.clone(), args[0].clone());
        }
    }
    let last = |mut l: String| {
        for _ in 0..onward.len() {
            match onward.get(&l) {
                Some(to) if *to != l => l = to.clone(),
                _ => break,
            }
        }
        l
    };
    for line in lines.iter_mut() {
        let e = effects(line);
        if let Line::Op { args, .. } = line
            && !e.targets.is_empty()
        {
            for a in args.iter_mut() {
                if e.targets.contains(a) {
                    *a = last(a.clone());
                }
            }
        }
    }
    let invert = |op: &str| {
        let pairs = [
            ("beq", "bne"),
            ("blt", "bge"),
            ("ble", "bgt"),
            ("beqz", "bnez"),
            ("bltz", "bgez"),
            ("blez", "bgtz"),
        ];
        pairs.iter().find_map(|&(a, b)| {
            if op == a {
                Some(b)
            } else if op == b {
                Some(a)
            } else {
                None
            }
        })
    };
    loop {
        let before = lines.len();
        // A label nothing jumps to would keep its neighbours out of a bundle.
        let named: BTreeSet<String> = lines
            .iter()
            .flat_map(|l| match l {
                Line::Raw(r) => r
                    .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '$'))
                    .map(str::to_string)
                    .collect(),
                _ => effects(l).targets,
            })
            .collect();
        lines.retain(|l| !matches!(l, Line::Label(name) if !named.contains(name)));
        let mut changed = false;
        let next_label = |lines: &[Line], i: usize| {
            lines[i + 1..].iter().find(|l| !matches!(l, Line::Text(_))).and_then(|l| match l {
                Line::Label(name) => Some(name.clone()),
                _ => None,
            })
        };
        let mut out: Vec<Line> = Vec::with_capacity(lines.len());
        let mut unreachable = false;
        let mut skip_to = 0;
        changed |= lines.len() != before;
        for i in 0..lines.len() {
            if i == skip_to && i > 0 {
                continue;
            }
            let line = &lines[i];
            match line {
                Line::Label(_) => unreachable = false,
                Line::Op { .. } if unreachable => {
                    changed = true;
                    continue;
                }
                Line::Op { op, args } => {
                    if op == "mv" && args[0] == args[1] {
                        changed = true;
                        continue;
                    }
                    // `mv a, b` then `mv b, a`: b still holds it.
                    if op == "mv"
                        && let Some(Line::Op { op: p, args: before }) =
                            out.iter().rev().find(|l| !matches!(l, Line::Text(_)))
                        && p == "mv"
                        && before[0] == args[1]
                        && before[1] == args[0]
                    {
                        changed = true;
                        continue;
                    }
                    // A branch over a jump: branch on the opposite test.
                    let next = (i + 1..lines.len()).find(|&k| !matches!(lines[k], Line::Text(_)));
                    if let Some(inverse) = invert(op)
                        && let Some(n) = next
                        && let Line::Op { op: j, args: to } = &lines[n]
                        && j == "j"
                        && args.last() == next_label(lines, n).as_ref()
                    {
                        let mut args = args.clone();
                        *args.last_mut().unwrap() = to[0].clone();
                        out.push(Line::Op { op: inverse.into(), args });
                        skip_to = n;
                        changed = true;
                        continue;
                    }
                    let e = effects(line);
                    if !e.targets.is_empty()
                        && e.targets.iter().all(|t| Some(t.clone()) == next_label(lines, i))
                    {
                        changed = true;
                        continue;
                    }
                    if !e.falls {
                        unreachable = true;
                    }
                }
                _ => {}
            }
            out.push(line.clone());
        }
        *lines = out;
        if !changed {
            break;
        }
    }
    // `rd = rs op rd` as `rd = rd op rs` where op commutes, for the
    // compressed form.
    for line in lines.iter_mut() {
        if let Line::Op { op, args } = line
            && ["add", "mul", "min", "max", "tmin", "tmax"].contains(&op.as_str())
            && args[0] == args[2]
            && args[0] != args[1]
        {
            args.swap(1, 2);
        }
    }
}

/// Allocate registers for a function and build its frame: the lines
/// after its label.
pub fn finish(f: Func) -> Result<Vec<String>, String> {
    let mut lines = parse(f.body);
    reuse(&mut lines);
    let mut spills: i128 = 0;
    let mut fresh = 0;
    let colours = loop {
        dead_code(&mut lines, f.results);
        coalesce(&mut lines, f.results);
        match colour(&lines, f.results) {
            Ok(c) => break c,
            Err(vs) => {
                if spills > 1000 {
                    return Err("too many values for the registers and the frame".into());
                }
                for v in vs {
                    spills += 1;
                    let off = -3 * (2 + f.slots + spills);
                    spill(&mut lines, &v, off, &mut fresh);
                }
            }
        }
    };
    for line in lines.iter_mut() {
        let regs: Vec<String> = {
            let e = effects(line);
            e.defs.into_iter().chain(e.uses).filter(|r| r.starts_with('%')).collect()
        };
        for r in regs {
            rename(line, &r, &colours[&r]);
        }
    }
    tidy(&mut lines);

    let saved: Vec<&str> =
        SAVED.iter().copied().filter(|s| colours.values().any(|c| c == s)).collect();
    let calls = lines.iter().any(|l| match l {
        Line::Raw(_) => true,
        // A system call traps and comes back, leaving ra alone.
        Line::Op { op, .. } => {
            let e = effects(l);
            op != "sys" && e.defs.len() > 1 && e.keep && e.targets.is_empty()
        }
        _ => false,
    });
    let memory = f.slots + spills;
    let fp = memory > 0 || f.stack_params;
    let ret_label = format!("{}$ret", f.name);
    let jumped = lines.iter().any(|l| match l {
        Line::Raw(r) => r.contains(&ret_label),
        _ => effects(l).targets.contains(&ret_label),
    });
    let ret = format!("{ret_label}:");
    let mut out = Vec::new();
    let emit = |out: &mut Vec<String>, s: String| out.push(format!("        {s}"));
    let body: Vec<String> = lines.iter().map(render).collect();
    if !fp && !calls && saved.is_empty() {
        // Nothing to keep: no frame (abi.md 2.4).
        out.extend(body);
        if jumped {
            out.push(ret.clone());
        }
        emit(&mut out, "ret".into());
    } else if !fp {
        // Nothing in memory: ra (if calls change it) and the saved
        // registers, reached from sp, with no frame pointer.
        let mut keep: Vec<&str> = Vec::new();
        if calls {
            keep.push("ra");
        }
        keep.extend(saved.iter());
        let frame = 3 * keep.len() as i128;
        emit(&mut out, format!("addi  sp, sp, {}", -frame));
        for (k, r) in keep.iter().enumerate() {
            emit(&mut out, format!("sw    {r}, {}(sp)", frame - 3 - 3 * k as i128));
        }
        out.extend(body);
        if jumped {
            out.push(ret.clone());
        }
        for (k, r) in keep.iter().enumerate().rev() {
            emit(&mut out, format!("lw    {r}, {}(sp)", frame - 3 - 3 * k as i128));
        }
        emit(&mut out, format!("addi  sp, sp, {frame}"));
        emit(&mut out, "ret".into());
    } else {
        // A frame pointer: ra at -3(s0), the caller's s0 at -6, the
        // locals and spills below, then the saved registers.
        let save_at = |k: usize| -3 * (2 + memory + k as i128 + 1);
        let frame = 3 * (2 + memory + saved.len() as i128);
        emit(&mut out, format!("addi  sp, sp, {}", -frame));
        emit(&mut out, format!("sw    ra, {}(sp)", frame - 3));
        emit(&mut out, format!("sw    s0, {}(sp)", frame - 6));
        emit(&mut out, format!("addi  s0, sp, {frame}"));
        for (k, r) in saved.iter().enumerate() {
            emit(&mut out, format!("sw    {r}, {}(s0)", save_at(k)));
        }
        out.extend(body);
        if jumped {
            out.push(ret.clone());
        }
        for (k, r) in saved.iter().enumerate() {
            emit(&mut out, format!("lw    {r}, {}(s0)", save_at(k)));
        }
        emit(&mut out, "mv    sp, s0".into());
        emit(&mut out, "lw    ra, -3(sp)".into());
        emit(&mut out, "lw    s0, -6(sp)".into());
        emit(&mut out, "ret".into());
    }
    Ok(out)
}

/// A function small enough to copy into its callers, as code generation
/// left it: it reads its arguments from a0.. and leaves its result in
/// a0, as a call would.
pub struct Inline {
    pub body: Vec<String>,
}

/// Whether `body`, the code of function `name`, can be copied into its
/// callers: it calls nothing, holds nothing in memory and has no `asm`.
/// Returns the count of its instructions other than copies, which
/// coalescing usually removes.
pub fn inline_cost(body: &[String], slots: i128, stack_params: bool) -> Option<usize> {
    if slots > 0 || stack_params {
        return None;
    }
    let mut cost = 0;
    for line in parse(body.to_vec()) {
        match &line {
            Line::Raw(_) => return None,
            Line::Op { op, .. } if op == "call" => return None,
            Line::Op { op, .. } if op != "mv" => cost += 1,
            _ => {}
        }
    }
    Some(cost)
}

/// `line` with each register or label name passed through `f`.
fn retoken(line: &str, f: &mut dyn FnMut(&str) -> Option<String>) -> String {
    let token = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '$' || c == '%';
    let mut s = String::new();
    let mut word = String::new();
    let mut flush = |word: &mut String, s: &mut String| {
        match f(word) {
            Some(w) => s.push_str(&w),
            None => s.push_str(word),
        }
        word.clear();
    };
    for c in line.chars() {
        if token(c) {
            word.push(c);
        } else {
            flush(&mut word, &mut s);
            s.push(c);
        }
    }
    flush(&mut word, &mut s);
    s
}

fn effects_of(line: &str) -> Effects {
    effects(&parse(vec![line.to_string()])[0])
}

/// Copy the bodies of the functions in `callees` over the calls to them
/// in `body`, each with its own virtual registers (from `fresh`) and
/// labels (numbered by `count`).
///
/// The caller sets the argument registers just before the call, and
/// the copy's first lines move them to its parameters. Fixed registers
/// do not coalesce, so the arguments go through new virtual registers
/// instead, where the caller sets them in straight-line code.
pub fn inline(
    body: Vec<String>,
    callees: &BTreeMap<String, Inline>,
    count: &mut usize,
    fresh: &mut dyn FnMut() -> &'static str,
) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(body.len());
    for line in body {
        let words: Vec<&str> = line.split_whitespace().collect();
        let callee = match words.as_slice() {
            ["call", name, n] if !line.starts_with(RAW) => {
                let name = name.trim_end_matches(',');
                callees.get(name).map(|c| (name.to_string(), c, n.parse::<usize>().unwrap_or(0)))
            }
            _ => None,
        };
        let Some((name, callee, n)) = callee else {
            out.push(line);
            continue;
        };
        *count += 1;
        let k = *count;

        // The straight-line run before the call.
        let run = out
            .iter()
            .rposition(|l| {
                !l.trim_start().starts_with('#') && {
                    let e = effects_of(l);
                    l.ends_with(':')
                        || l.starts_with(RAW)
                        || !e.targets.is_empty()
                        || e.defs.len() > 1
                }
            })
            .map_or(0, |i| i + 1);
        let mut args: BTreeMap<String, &'static str> = BTreeMap::new();
        for a in ARGS.iter().take(n) {
            let first = out[run..].iter().find_map(|l| {
                if l.trim_start().starts_with('#') {
                    return None;
                }
                let e = effects_of(l);
                if e.uses.iter().any(|u| u == a) {
                    Some(false)
                } else if e.defs.iter().any(|d| d == a) {
                    Some(true)
                } else {
                    None
                }
            });
            if first == Some(true) {
                let v = fresh();
                args.insert(a.to_string(), v);
                for l in out[run..].iter_mut() {
                    if !l.trim_start().starts_with('#') {
                        *l = retoken(l, &mut |w| (w == *a).then(|| v.to_string()));
                    }
                }
            }
        }

        let prefix = format!("{name}$");
        let mut regs: BTreeMap<String, &'static str> = BTreeMap::new();
        let mut params = true;
        for l in &callee.body {
            if l.trim_start().starts_with('#') {
                out.push(l.clone());
                continue;
            }
            // The leading copies from argument registers take the
            // caller's values.
            let words: Vec<&str> = l.split([' ', ',']).filter(|w| !w.is_empty()).collect();
            params &= matches!(words.as_slice(), ["mv", _, a] if ARGS.contains(a));
            out.push(retoken(l, &mut |w| {
                if w.starts_with('%') {
                    Some(regs.entry(w.to_string()).or_insert_with(&mut *fresh).to_string())
                } else if w.starts_with(&prefix) {
                    Some(format!("{w}_{k}"))
                } else if params {
                    args.get(w).map(|v| v.to_string())
                } else {
                    None
                }
            }));
        }
        out.push(format!("{name}$ret_{k}:"));
    }
    out
}
