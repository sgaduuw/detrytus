//! Code generation: the checked program into assembly (abi.md).
//!
//! It walks the tree, checking types on the way, and emits assembly over
//! virtual registers: each value, and each local whose address is never
//! taken, gets one of its own. Locals in memory live in the frame,
//! reached from `s0`. `alloc` then gives each function its registers.

use crate::alloc;
use std::collections::{BTreeMap, HashMap, HashSet};

use crate::ast::*;
use crate::lex::{TRYTE_MAX, WORD_MAX};
use crate::{Error, Output, Unit};

const ARGS: [&str; 6] = ["a0", "a1", "a2", "a3", "a4", "a5"];

/// Builtins of two arguments, each the instruction of the same name.
const BUILTIN2: [&str; 12] =
    ["min", "max", "divr", "remr", "tmul", "tsum", "tcons", "tany", "band", "bor", "bxor", "bsh"];
/// Builtins of one argument.
const BUILTIN1: [&str; 5] = ["abs", "tabs", "lzt", "tzt", "nzt"];
/// Builtins whose last arguments are constants the instruction holds.
const FIELD: [&str; 3] = ["tlut", "tex", "tins"];
/// The largest `tlut` table: nine trits, `fn[8:0]` (isa.md 7.5).
const TABLE_MAX: i128 = (19_683 - 1) / 2;

/// The 16-trit immediate of format I, H16.
const IMM_MAX: i128 = (43_046_721 - 1) / 2;

#[derive(Clone)]
struct Sig {
    ret: Type,
    params: Vec<Type>,
    defined: bool,
}

#[derive(Clone)]
struct Local {
    ty: Type,
    /// From `s0`: a scalar's or word array's centre, a tryte array's first tryte.
    off: i128,
    /// The saved register that holds it instead, for a scalar whose
    /// address is never taken.
    reg: Option<&'static str>,
}

/// One tryte or word of a global's initial value.
enum Cell {
    Tryte(i128),
    Word(String),
}

enum Place {
    Local(Local),
    Global(Type),
    Const(i128),
    /// A constant from imported assembly: emitted by name.
    Imported(String),
}

struct Gen {
    file: String,
    consts: HashMap<String, i128>,
    imported: HashMap<String, i128>,
    globals: HashMap<String, Type>,
    funcs: HashMap<String, Sig>,
    code: Vec<String>,
    data: Vec<String>,
    strings: usize,
    warnings: Vec<String>,
    // The function being compiled:
    fname: String,
    labels: usize,
    scopes: Vec<HashMap<String, Local>>,
    slots: i128,
    loops: Vec<(String, String)>,
    ret: Type,
    /// The virtual registers of the values being computed, innermost
    /// last, and how many virtual registers there are.
    stack: Vec<&'static str>,
    vregs: usize,
    /// Functions small enough to copy into their callers, so far.
    inlinable: BTreeMap<String, alloc::Inline>,
    /// Copies made, which number their labels.
    inlined: usize,
    /// How many calls each function has in the code kept.
    call_sites: HashMap<String, usize>,
    /// Names whose address the function takes: they stay in memory.
    addressed: HashSet<String>,
    /// Imported assembly set `.gp`: a global scalar is loaded and stored
    /// by name, in one instruction.
    gp: bool,
}

type R<T> = Result<T, Error>;

pub(crate) fn generate(units: &[Unit], main_file: &str, alone: bool) -> R<Output> {
    let mut g = Gen {
        file: main_file.to_string(),
        consts: HashMap::new(),
        imported: HashMap::new(),
        globals: HashMap::new(),
        funcs: HashMap::new(),
        code: Vec::new(),
        data: Vec::new(),
        strings: 0,
        warnings: Vec::new(),
        fname: String::new(),
        labels: 0,
        scopes: Vec::new(),
        slots: 0,
        loops: Vec::new(),
        ret: Type::Void,
        stack: Vec::new(),
        vregs: 0,
        inlinable: BTreeMap::new(),
        inlined: 0,
        call_sites: HashMap::new(),
        addressed: HashSet::new(),
        gp: units.iter().any(|u| matches!(u, Unit::Include { gp: true, own: true, .. })),
    };
    clear_structs();
    g.declare(units)?;
    // A file compiled alone is a library, whose functions and globals
    // other objects use, unless it defines main: then it is a program,
    // whose start code (in the same object) is all that calls into it.
    let library = alone
        && !units.iter().any(|u| {
            matches!(u, Unit::Item { item: Item::Func(f), own: true, .. }
                if f.name == "main" && f.body.is_some())
        });
    // Compiling a library, any function may be called from another file.
    let used = if library { None } else { used_functions(units) };
    g.call_sites = call_sites(units, used.as_ref());
    if library {
        // Other objects may call any of them too: none has one caller.
        for u in units {
            if let Unit::Item { item: Item::Func(f), own: true, .. } = u {
                *g.call_sites.entry(f.name.clone()).or_insert(0) += 2;
            }
        }
    }
    g.code.push(format!("# Compiled by detrytus tc from {main_file}: docs/trice.md, docs/abi.md."));
    if library {
        // What this file defines, for the other objects.
        let names: Vec<&str> = units
            .iter()
            .filter_map(|u| match u {
                Unit::Item { item: Item::Func(f), own: true, .. } if f.body.is_some() => {
                    Some(f.name.as_str())
                }
                Unit::Item { item: Item::Global { name, .. }, own: true, .. } => Some(name),
                _ => None,
            })
            .collect();
        if !names.is_empty() {
            g.code.push(format!("        .global {}", names.join(", ")));
        }
    } else if alone {
        // A program's start code may be an object of its own.
        g.code.push("        .global main".into());
    }
    let mut compressing = false;
    let mut ranges = Vec::new();
    for unit in units {
        match unit {
            Unit::Include { path, own: true, .. } => {
                if alone {
                    // Assembly a file imports is a part of its own too.
                    g.code.push("        .part".into());
                }
                g.code.push(format!("        .include \"{path}\""))
            }
            Unit::Item { file, item: Item::Func(f), own: true } if f.body.is_some() => {
                if !compressing {
                    g.code.push("        .compress".into());
                    compressing = true;
                }
                g.file = file.clone();
                // A function nothing calls is checked, and left out.
                let (code, data) = (g.code.len(), g.data.len());
                if library {
                    // A part of its own, which the linker leaves out
                    // when no other object uses it.
                    g.code.push("        .part".into());
                }
                g.function(f)?;
                if used.as_ref().is_some_and(|u| !u.contains(&f.name)) {
                    g.code.truncate(code);
                    g.data.truncate(data);
                } else {
                    ranges.push((f.name.clone(), code, g.code.len()));
                }
            }
            Unit::Item { .. } | Unit::Include { .. } => {}
        }
    }
    if !compressing {
        g.code.push("        .compress".into());
    }
    if used.is_some() {
        g.code = drop_inlined(units, std::mem::take(&mut g.code), ranges);
        g.data = used_data(units, &g.code, std::mem::take(&mut g.data));
    }
    let mut assembly = g.code.join("\n");
    assembly.push('\n');
    for line in &g.data {
        if library {
            assembly.push_str("        .part\n");
        }
        assembly.push_str(line);
        assembly.push('\n');
    }
    assembly.push_str("        .nocompress\n");
    Ok(Output { assembly, warnings: g.warnings })
}

fn is_register(name: &str) -> bool {
    let numbered = |prefix: &str, max: u32| {
        name.strip_prefix(prefix).and_then(|n| n.parse::<u32>().ok()).is_some_and(|n| n <= max)
    };
    matches!(name, "zero" | "ra" | "sp" | "gp" | "tp" | "fp")
        || numbered("a", 5)
        || numbered("t", 6)
        || numbered("s", 8)
        || numbered("r", 13)
}

fn is_builtin(name: &str) -> bool {
    name == "sys" || FIELD.contains(&name) || BUILTIN1.contains(&name) || BUILTIN2.contains(&name)
}

fn literal_minus_one(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Int(-1) => true,
        ExprKind::Unary("-", x) => matches!(x.kind, ExprKind::Int(1)),
        _ => false,
    }
}

fn is_zero_literal(e: &Expr) -> bool {
    matches!(e.kind, ExprKind::Int(0))
}

fn asm_string(cps: &[i128]) -> String {
    let mut s = String::from("\"");
    for &c in cps {
        match c {
            0 => s.push_str("\\0"),
            10 => s.push_str("\\n"),
            9 => s.push_str("\\t"),
            34 => s.push_str("\\\""),
            92 => s.push_str("\\\\"),
            _ => s.push(char::from_u32(c as u32).unwrap_or('?')),
        }
    }
    s.push('"');
    s
}

/// `x / 3^n` rounded to nearest, as a right shift does (isa.md 7.3).
fn shift_right(x: i128, n: u32) -> i128 {
    let d = 3i128.pow(n.min(40));
    let (q, r) = (x / d, x % d);
    if 2 * r.abs() > d { q + r.signum() } else { q }
}

impl Gen {
    fn err<T>(&self, line: usize, msg: impl Into<String>) -> R<T> {
        Err(Error::new(&self.file, line, msg.into()))
    }

    // --- declarations ----------------------------------------------------

    fn check_name(&self, name: &str, line: usize) -> R<()> {
        if is_register(name) {
            return self.err(line, format!("{name} is a register name"));
        }
        if is_builtin(name) {
            return self.err(line, format!("{name} is a builtin"));
        }
        if self.imported.contains_key(name) {
            return self.err(line, format!("{name} is already a constant from imported assembly"));
        }
        if self.consts.contains_key(name) || self.globals.contains_key(name) {
            return self.err(line, format!("{name} is already defined"));
        }
        Ok(())
    }

    fn declare(&mut self, units: &[Unit]) -> R<()> {
        for unit in units {
            match unit {
                Unit::Include { consts, file, line, .. } => {
                    self.file = file.clone();
                    for (name, value) in consts {
                        if self.consts.contains_key(name)
                            || self.globals.contains_key(name)
                            || self.funcs.contains_key(name)
                        {
                            return self.err(*line, format!("the import defines {name} again"));
                        }
                        self.imported.insert(name.clone(), *value);
                    }
                }
                Unit::Item { file, item, own } => {
                    self.file = file.clone();
                    self.declare_item(item, *own)?;
                }
            }
        }
        Ok(())
    }

    /// Declare an item; a global not `own` is another object's, so its
    /// data is not emitted here.
    fn declare_item(&mut self, item: &Item, own: bool) -> R<()> {
        match item {
            Item::Import(..) => Ok(()),
            Item::Struct { name, fields, line } => {
                if layout(name).is_some() {
                    return self.err(*line, format!("struct {name} is already defined"));
                }
                let mut laid = Vec::new();
                for (ty, field, array) in fields {
                    if laid.iter().any(|(n, _)| n == field) {
                        return self
                            .err(*line, format!("struct {name} has two fields called {field}"));
                    }
                    if *ty == Type::Struct(name.clone()) {
                        return self.err(
                            *line,
                            format!("struct {name} cannot hold itself: use a pointer"),
                        );
                    }
                    laid.push((field.clone(), self.decl_type(ty, array, *line)?));
                }
                if laid.is_empty() {
                    return self.err(*line, format!("struct {name} needs a field"));
                }
                define_struct(name, lay_out(laid));
                Ok(())
            }
            Item::Const { ty, name, value, line } => {
                self.check_name(name, *line)?;
                if self.funcs.contains_key(name) {
                    return self.err(*line, format!("{name} is already a function"));
                }
                if !ty.numeric() {
                    return self.err(*line, "a constant is an int, tryte or trit");
                }
                let v = self.const_eval(value)?;
                let limit = if *ty == Type::Int { WORD_MAX } else { TRYTE_MAX.min(ty_max(ty)) };
                if v.abs() > limit {
                    return self.err(*line, format!("{v} does not fit {}", ty.a()));
                }
                self.consts.insert(name.clone(), v);
                Ok(())
            }
            Item::Global { ty, name, array, init, line } => {
                self.check_name(name, *line)?;
                if self.funcs.contains_key(name) {
                    return self.err(*line, format!("{name} is already a function"));
                }
                let ty = self.decl_type(ty, array, *line)?;
                let data = self.global_data(name, &ty, init.as_ref(), *line)?;
                if own {
                    self.data.push(data);
                }
                self.globals.insert(name.clone(), ty);
                Ok(())
            }
            Item::Func(f) => {
                if is_register(&f.name)
                    || is_builtin(&f.name)
                    || self.imported.contains_key(&f.name)
                {
                    self.check_name(&f.name, f.line)?;
                }
                if self.consts.contains_key(&f.name) || self.globals.contains_key(&f.name) {
                    return self.err(f.line, format!("{} is already defined", f.name));
                }
                if matches!(f.ret, Type::Struct(_)) {
                    return self.err(
                        f.line,
                        "a function cannot return a struct yet: fill one its caller points to",
                    );
                }
                let mut params = Vec::new();
                for (t, n) in &f.params {
                    if *t == Type::Void {
                        return self.err(f.line, format!("parameter {n} cannot be void"));
                    }
                    if is_register(n) {
                        return self.err(f.line, format!("{n} is a register name"));
                    }
                    if matches!(t, Type::Struct(_)) {
                        return self.err(
                            f.line,
                            format!("pass a pointer to the struct: {n} cannot be one yet"),
                        );
                    }
                    params.push(t.clone());
                }
                let sig = Sig { ret: f.ret.clone(), params, defined: f.body.is_some() };
                if let Some(old) = self.funcs.get(&f.name) {
                    if old.ret != sig.ret || old.params != sig.params {
                        return self.err(f.line, format!("{} was declared differently", f.name));
                    }
                    if old.defined && sig.defined {
                        return self.err(f.line, format!("{} is defined twice", f.name));
                    }
                    if old.defined {
                        return Ok(());
                    }
                }
                self.funcs.insert(f.name.clone(), sig);
                Ok(())
            }
        }
    }

    fn decl_type(&self, ty: &Type, array: &Option<Expr>, line: usize) -> R<Type> {
        if *ty == Type::Void {
            return self.err(line, "a variable cannot be void");
        }
        if !ty.complete() {
            return self.err(line, format!("{} is not defined", ty.name()));
        }
        match array {
            None => Ok(ty.clone()),
            Some(n) => {
                let n = self.const_eval(n)?;
                if n < 1 {
                    return self.err(line, "an array needs at least one element");
                }
                Ok(Type::Array(Box::new(ty.clone()), n))
            }
        }
    }

    fn global_data(
        &mut self,
        name: &str,
        ty: &Type,
        init: Option<&Expr>,
        line: usize,
    ) -> R<String> {
        let Some(init) = init else {
            return Ok(match ty {
                t if t.aggregate() && t.word_aligned() => {
                    format!("{name}: .words {}", t.size() / 3)
                }
                t if t.aggregate() => format!("{name}: .zero {}", t.size()),
                t if t.size() == 1 => format!("{name}: .tryte 0"),
                _ => format!("{name}: .word 0"),
            });
        };
        let mut cells = HashMap::new();
        self.image(ty, init, 0, &mut cells, line)?;
        let mut values = Vec::new();
        let directive = if ty.word_aligned() {
            for k in 0..(ty.size() / 3).max(1) {
                let c = 3 * k;
                values.push(match cells.get(&c) {
                    Some(Cell::Word(w)) => w.clone(),
                    _ => {
                        let t = |o: i128| match cells.get(&o) {
                            Some(Cell::Tryte(v)) => *v,
                            _ => 0,
                        };
                        (t(c - 1) + t(c) * 19_683 + t(c + 1) * 387_420_489).to_string()
                    }
                });
            }
            ".word "
        } else {
            for o in 0..ty.size() {
                values.push(match cells.get(&o) {
                    Some(Cell::Tryte(v)) => v.to_string(),
                    _ => "0".into(),
                });
            }
            ".tryte"
        };
        let mut out = String::new();
        for (i, chunk) in values.chunks(12).enumerate() {
            let label = if i == 0 { format!("{name}:") } else { String::new() };
            if i > 0 {
                out.push('\n');
            }
            out.push_str(&format!("{label:<12}{directive} {}", chunk.join(", ")));
        }
        Ok(out)
    }

    /// Lay out the constant initialiser `init` of an object of type `ty`
    /// at `off` from the object's address, one cell per tryte or word.
    fn image(
        &mut self,
        ty: &Type,
        init: &Expr,
        off: i128,
        cells: &mut HashMap<i128, Cell>,
        line: usize,
    ) -> R<()> {
        match (ty, &init.kind) {
            (Type::Array(elem, n), ExprKind::List(items)) => {
                if items.len() as i128 > *n {
                    return self
                        .err(line, format!("{} initialisers for {} elements", items.len(), n));
                }
                for (i, item) in items.iter().enumerate() {
                    self.image(elem, item, off + i as i128 * elem.size(), cells, line)?;
                }
            }
            (Type::Array(elem, n), ExprKind::Str(cps)) if **elem == Type::Tryte => {
                if cps.len() as i128 >= *n {
                    return self.err(
                        line,
                        format!("the string needs {} trytes with its 0", cps.len() + 1),
                    );
                }
                for (i, cp) in cps.iter().enumerate() {
                    cells.insert(off + i as i128, Cell::Tryte(*cp));
                }
            }
            (Type::Struct(name), ExprKind::List(items)) => {
                let fields = layout(name).map(|l| l.fields).unwrap_or_default();
                if items.len() > fields.len() {
                    return self.err(
                        line,
                        format!("{} initialisers for {} fields", items.len(), fields.len()),
                    );
                }
                for (item, (_, fty, foff)) in items.iter().zip(&fields) {
                    self.image(fty, item, off + foff, cells, line)?;
                }
            }
            (t, _) if t.aggregate() => {
                return self.err(line, format!("{} is initialised with a list in braces", t.a()));
            }
            (_, ExprKind::List(_)) => {
                return self
                    .err(line, format!("{} is initialised with a value, not a list", ty.a()));
            }
            _ => {
                let v = self.scalar_init(ty, init, line)?;
                if ty.size() == 1 {
                    let v: i128 = v.parse().unwrap_or(0);
                    if v.abs() > ty_max(ty) {
                        return self.err(line, format!("{v} does not fit {}", ty.a()));
                    }
                    cells.insert(off, Cell::Tryte(v));
                } else {
                    cells.insert(off, Cell::Word(v));
                }
            }
        }
        Ok(())
    }

    /// A scalar global's constant initialiser, as assembly.
    fn scalar_init(&mut self, ty: &Type, e: &Expr, line: usize) -> R<String> {
        Ok(match &e.kind {
            ExprKind::Str(s) if *ty == Type::Ptr(Box::new(Type::Tryte)) => self.string(s),
            ExprKind::Str(_) => {
                return self.err(line, "a string initialises a tryte* or a tryte array");
            }
            ExprKind::Bool(v) if *ty == Type::Bool => v.to_string(),
            ExprKind::Unary("&", x) if ty.is_ptr() => match &x.kind {
                ExprKind::Name(g) if self.globals.contains_key(g) => g.clone(),
                ExprKind::Index(a, k) => match (&a.kind, self.try_const(k)) {
                    (ExprKind::Name(g), Some(k)) => match self.globals.get(g) {
                        Some(Type::Array(elem, _)) => format!("{g} + {}", k * elem.size()),
                        _ => return self.err(line, "a global's initialiser must be a constant"),
                    },
                    _ => return self.err(line, "a global's initialiser must be a constant"),
                },
                _ => return self.err(line, "a global's initialiser must be a constant"),
            },
            ExprKind::Name(g)
                if ty.is_ptr() && matches!(self.globals.get(g), Some(Type::Array(..))) =>
            {
                g.clone()
            }
            _ if *ty == Type::Bool => {
                return self.err(line, "a bool is true, false or unknown");
            }
            _ if ty.is_ptr() && !is_zero_literal(e) => {
                return self.err(line, "a pointer starts as 0, a string or a global's address");
            }
            _ => self.const_eval(e)?.to_string(),
        })
    }

    fn string(&mut self, s: &[i128]) -> String {
        self.strings += 1;
        let label = format!("str${}", self.strings);
        self.data.push(format!("{label}: .string {}", asm_string(s)));
        label
    }

    /// The value of a constant expression.
    fn const_eval(&self, e: &Expr) -> R<i128> {
        self.try_const(e).ok_or(()).or_else(|_| self.err(e.line, "a constant is needed here"))
    }

    fn try_const(&self, e: &Expr) -> Option<i128> {
        let v = match &e.kind {
            ExprKind::Int(v) => *v,
            ExprKind::Name(n) => {
                if self.scopes.iter().any(|s| s.contains_key(n)) {
                    return None;
                }
                *self.consts.get(n).or_else(|| self.imported.get(n))?
            }
            ExprKind::Unary("-", x) => -self.try_const(x)?,
            ExprKind::Binary(op, a, b) => {
                let (a, b) = (self.try_const(a)?, self.try_const(b)?);
                match *op {
                    "+" => a + b,
                    "-" => a - b,
                    "*" => a.checked_mul(b)?,
                    "/" if b != 0 => a / b,
                    "%" if b != 0 => a % b,
                    "<<" if (0..=27).contains(&b) => a.checked_mul(3i128.pow(b as u32))?,
                    ">>" if (0..=27).contains(&b) => shift_right(a, b as u32),
                    _ => return None,
                }
            }
            ExprKind::Cast(Type::Int, x) => self.try_const(x)?,
            ExprKind::SizeofType(t) => t.size(),
            _ => return None,
        };
        (v.abs() <= WORD_MAX).then_some(v)
    }

    // --- emitting ----------------------------------------------------------

    fn emit(&mut self, s: impl AsRef<str>) {
        self.code.push(format!("        {}", s.as_ref()));
    }

    fn label(&mut self, l: &str) {
        self.code.push(format!("{l}:"));
    }

    fn new_label(&mut self) -> String {
        self.labels += 1;
        format!("{}${}", self.fname, self.labels)
    }

    /// A new virtual register on the stack of values being computed.
    fn push(&mut self, _line: usize) -> R<&'static str> {
        let r = self.vreg();
        self.stack.push(r);
        Ok(r)
    }

    /// A new virtual register, for a local or a value of its own.
    fn vreg(&mut self) -> &'static str {
        self.vregs += 1;
        alloc::vreg(self.vregs)
    }

    fn top(&self) -> &'static str {
        self.stack[self.stack.len() - 1]
    }

    fn second(&self) -> &'static str {
        self.stack[self.stack.len() - 2]
    }

    fn pop(&mut self) {
        self.stack.pop();
    }

    /// Whether `r` holds a value being computed, which may be changed
    /// in place, rather than a variable.
    fn is_temp(&self, r: &str) -> bool {
        self.stack.contains(&r)
    }

    fn load(&mut self, ty: &Type, dst: &str, off: i128, base: &str) {
        let op = if ty.size() == 1 { "lt" } else { "lw" };
        self.emit(format!("{op}    {dst}, {off}({base})"));
    }

    fn store(&mut self, ty: &Type, src: &str, off: i128, base: &str) {
        // st keeps 9 trits; a trit keeps one (a value in a register that
        // is not a temporary was cut to its type already).
        if *ty == Type::Trit && self.is_temp(src) {
            self.emit(format!("tex   {src}, {src}, 0, 1"));
        }
        let op = if ty.size() == 1 { "st" } else { "sw" };
        self.emit(format!("{op}    {src}, {off}({base})"));
    }

    // --- functions -------------------------------------------------------

    fn function(&mut self, f: &Func) -> R<()> {
        self.fname = f.name.clone();
        self.labels = 0;
        self.slots = 0;
        self.stack.clear();
        self.loops.clear();
        self.ret = f.ret.clone();
        self.scopes = vec![HashMap::new()];
        self.addressed.clear();
        for st in f.body.as_ref().unwrap() {
            stmt_addressed(st, &mut self.addressed);
        }
        let outer = std::mem::take(&mut self.code);
        for (i, (ty, name)) in f.params.iter().enumerate() {
            if self.scopes[0].contains_key(name) {
                return self.err(f.line, format!("two parameters are called {name}"));
            }
            let local = if i < ARGS.len() {
                let local = self.bind(name, ty);
                self.store_local(&local, ARGS[i], ty);
                local
            } else {
                // The caller's words at 0(sp), 3(sp), ... on entry (abi.md
                // 2.2), where s0 points; a tryte is the word's low tryte.
                let centre = 3 * (i - ARGS.len()) as i128;
                let off = if ty.size() == 1 { centre - 1 } else { centre };
                let mem = Local { ty: ty.clone(), off, reg: None };
                let local = self.bind(name, ty);
                if let Some(r) = local.reg {
                    self.load(ty, r, off, "s0");
                    local
                } else {
                    mem
                }
            };
            self.scopes[0].insert(name.clone(), local);
        }
        for s in f.body.as_ref().unwrap() {
            self.stmt(s)?;
        }
        let ret_label = format!("{}$ret", f.name);
        if self.code.last().is_some_and(|l| l.trim() == format!("j     {ret_label}")) {
            self.code.pop();
        }
        let body = std::mem::replace(&mut self.code, outer);
        let mut n = self.vregs;
        let body = alloc::inline(body, &self.inlinable, &mut self.inlined, &mut || {
            n += 1;
            alloc::vreg(n)
        });
        self.vregs = n;
        let stack_params = f.params.len() > ARGS.len();
        if f.name != "main"
            && let Some(cost) = alloc::inline_cost(&body, self.slots, stack_params)
            && (cost <= INLINE_MAX || self.call_sites.get(&f.name) == Some(&1))
        {
            self.inlinable.insert(f.name.clone(), alloc::Inline { body: body.clone() });
        }
        self.code.push(String::new());
        self.code.push(format!("# {} {}(...), {}:{}", f.ret.name(), f.name, self.file, f.line));
        self.label(&f.name);
        let done = alloc::finish(alloc::Func {
            name: &f.name,
            results: usize::from(f.ret != Type::Void),
            slots: self.slots,
            stack_params,
            body,
        })
        .map_err(|m| Error::new(&self.file, f.line, m))?;
        self.code.extend(done);
        Ok(())
    }

    /// Frame room for a local: its offset from `s0` (abi.md 2.4: `ra`
    /// at -3, the caller's `s0` at -6, locals below).
    fn allocate(&mut self, ty: &Type) -> i128 {
        let words = (ty.size() + 2) / 3;
        self.slots += words;
        let lowest_centre = -3 * (2 + self.slots);
        if ty.word_aligned() || !ty.aggregate() { lowest_centre } else { lowest_centre - 1 }
    }

    /// A new local's home: a virtual register for a scalar whose
    /// address is never taken, else a frame slot.
    fn bind(&mut self, name: &str, ty: &Type) -> Local {
        if !ty.aggregate() && !self.addressed.contains(name) {
            let r = self.vreg();
            return Local { ty: ty.clone(), off: 0, reg: Some(r) };
        }
        let off = self.allocate(ty);
        Local { ty: ty.clone(), off, reg: None }
    }

    /// Store register `r`, holding a value of type `from`, in a local.
    fn store_local(&mut self, local: &Local, r: &str, from: &Type) {
        match local.reg {
            Some(reg) => {
                self.emit(format!("mv    {reg}, {r}"));
                self.narrow(&local.ty, from, reg);
            }
            None => self.store(&local.ty, r, local.off, "s0"),
        }
    }

    /// Cut the value in register `r`, of type `from`, to fit `to`, as a
    /// store to memory would.
    fn narrow(&mut self, to: &Type, from: &Type, r: &str) {
        let len = match to {
            Type::Tryte if from.size() == 3 => 9,
            Type::Trit if !matches!(from, Type::Trit | Type::Bool) => 1,
            _ => return,
        };
        self.emit(format!("tex   {r}, {r}, 0, {len}"));
    }

    fn open_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn close_scope(&mut self) {
        self.scopes.pop();
    }

    fn lookup(&self, name: &str) -> Option<Place> {
        for scope in self.scopes.iter().rev() {
            if let Some(l) = scope.get(name) {
                return Some(Place::Local(l.clone()));
            }
        }
        if let Some(t) = self.globals.get(name) {
            return Some(Place::Global(t.clone()));
        }
        if let Some(v) = self.consts.get(name) {
            return Some(Place::Const(*v));
        }
        if self.imported.contains_key(name) {
            return Some(Place::Imported(name.to_string()));
        }
        None
    }

    // --- statements --------------------------------------------------------

    fn stmt(&mut self, s: &Stmt) -> R<()> {
        debug_assert!(self.stack.is_empty());
        if let Some(line) = stmt_line(s) {
            self.code.push(format!("# {}:{line}", self.file));
        }
        match s {
            Stmt::Block(body) => {
                self.open_scope();
                for s in body {
                    self.stmt(s)?;
                }
                self.close_scope();
            }
            Stmt::Local { ty, name, array, init, line } => {
                if is_register(name) {
                    return self.err(*line, format!("{name} is a register name"));
                }
                if self.scopes.last().unwrap().contains_key(name) {
                    return self.err(*line, format!("{name} is already declared here"));
                }
                let ty = self.decl_type(ty, array, *line)?;
                if let Some(init) = init
                    && (ty.aggregate() || matches!(init.kind, ExprKind::List(_)))
                {
                    let off = self.allocate(&ty);
                    self.init_local(&ty, off, init, *line)?;
                    self.scopes
                        .last_mut()
                        .unwrap()
                        .insert(name.clone(), Local { ty, off, reg: None });
                } else if let Some(init) = init
                    && self.is_simple(init)
                    && !self.addressed.contains(name)
                {
                    let local = self.bind(name, &ty);
                    let reg = local.reg.unwrap();
                    let t = self.simple_into(init, reg)?;
                    self.check_assign(&ty, &t, init)?;
                    self.narrow(&ty, &t, reg);
                    self.scopes.last_mut().unwrap().insert(name.clone(), local);
                } else if let Some(init) = init {
                    let t = self.expr(init)?;
                    self.check_assign(&ty, &t, init)?;
                    let local = self.bind(name, &ty);
                    let r = self.top();
                    self.store_local(&local, r, &t);
                    self.pop();
                    self.scopes.last_mut().unwrap().insert(name.clone(), local);
                } else {
                    let local = self.bind(name, &ty);
                    self.scopes.last_mut().unwrap().insert(name.clone(), local);
                }
            }
            Stmt::Expr(e) => self.effect(e)?,
            Stmt::If(cond, then, other) => {
                let skip = self.new_label();
                self.branch_false(cond, &skip)?;
                self.scoped(then)?;
                if let Some(other) = other {
                    let end = self.new_label();
                    self.emit(format!("j     {end}"));
                    self.label(&skip);
                    self.scoped(other)?;
                    self.label(&end);
                } else {
                    self.label(&skip);
                }
            }
            Stmt::While(cond, body) => {
                let (top, end) = (self.new_label(), self.new_label());
                self.label(&top);
                self.branch_false(cond, &end)?;
                self.loops.push((end.clone(), top.clone()));
                self.scoped(body)?;
                self.loops.pop();
                self.emit(format!("j     {top}"));
                self.label(&end);
            }
            Stmt::For(init, cond, step, body) => {
                self.open_scope();
                if let Some(init) = init {
                    self.stmt(init)?;
                }
                let (top, next, end) = (self.new_label(), self.new_label(), self.new_label());
                self.label(&top);
                if let Some(cond) = cond {
                    self.branch_false(cond, &end)?;
                }
                self.loops.push((end.clone(), next.clone()));
                self.scoped(body)?;
                self.loops.pop();
                self.label(&next);
                if let Some(step) = step {
                    self.effect(step)?;
                }
                self.emit(format!("j     {top}"));
                self.label(&end);
                self.close_scope();
            }
            Stmt::Break(line) | Stmt::Continue(line) => {
                let Some((brk, cont)) = self.loops.last().cloned() else {
                    return self.err(*line, "break and continue belong in a loop");
                };
                let to = if matches!(s, Stmt::Break(_)) { brk } else { cont };
                self.emit(format!("j     {to}"));
            }
            Stmt::Return(value, line) => {
                match (value, self.ret.clone()) {
                    (Some(e), Type::Void) => {
                        return self.err(e.line, format!("{} returns nothing", self.fname));
                    }
                    (Some(e), ret) if self.is_simple(e) => {
                        let t = self.simple_into(e, "a0")?;
                        self.check_assign(&ret, &t, e)?;
                        self.narrow(&ret, &t, "a0");
                    }
                    (Some(e), ret) => {
                        let t = self.expr(e)?;
                        self.check_assign(&ret, &t, e)?;
                        let r = self.top();
                        self.emit(format!("mv    a0, {r}"));
                        self.narrow(&ret, &t, "a0");
                        self.pop();
                    }
                    (None, Type::Void) => {}
                    (None, ret) => {
                        return self.err(*line, format!("return needs {}", ret.a()));
                    }
                }
                let to = format!("{}$ret", self.fname);
                self.emit(format!("j     {to}"));
            }
            Stmt::Which { value, arms, bool_arms } => self.which(value, arms, *bool_arms)?,
            Stmt::Asm(text, _) => {
                // Marked, so the allocator passes it through untouched.
                for line in text.lines().filter(|l| !l.trim().is_empty()) {
                    self.code.push(format!("{}{line}", alloc::RAW));
                }
            }
        }
        Ok(())
    }

    /// A statement in a scope of its own, as the body of an if or a loop.
    fn scoped(&mut self, s: &Stmt) -> R<()> {
        self.open_scope();
        let r = self.stmt(s);
        self.close_scope();
        r
    }

    fn which(&mut self, value: &Expr, arms: &[(Arm, Vec<Stmt>, usize)], bool_arms: bool) -> R<()> {
        let t = self.expr(value)?;
        if bool_arms && t != Type::Bool {
            return self.err(
                value.line,
                format!("true, unknown and false arms need a bool, not {}", t.a()),
            );
        }
        if !bool_arms && !t.numeric() {
            return self
                .err(value.line, format!("< = > arms need an int, tryte or trit, not {}", t.a()));
        }
        let end = self.new_label();
        let mut target = HashMap::new();
        for (arm, _, _) in arms {
            if *arm != Arm::Zero {
                target.insert(*arm, self.new_label());
            }
        }
        let to = |a: Arm| target.get(&a).cloned().unwrap_or_else(|| end.clone());
        let r = self.top();
        self.emit(format!("br3   {r}, {}, {}", to(Arm::Neg), to(Arm::Pos)));
        self.pop();
        for case in [Arm::Zero, Arm::Neg, Arm::Pos] {
            let Some((_, body, _)) = arms.iter().find(|(a, _, _)| *a == case) else {
                continue;
            };
            if case != Arm::Zero {
                let l = to(case);
                self.label(&l);
            }
            self.open_scope();
            for s in body {
                self.stmt(s)?;
            }
            self.close_scope();
            self.emit(format!("j     {end}"));
        }
        if self.code.last().is_some_and(|l| l.trim() == format!("j     {end}")) {
            self.code.pop();
        }
        self.label(&end);
        Ok(())
    }

    // --- conditions --------------------------------------------------------

    fn comparison(op: &str) -> Option<&'static str> {
        Some(match op {
            "<" => "blt",
            "<=" => "ble",
            ">" => "bgt",
            ">=" => "bge",
            "==" => "beq",
            "!=" => "bne",
            _ => return None,
        })
    }

    fn inverse(op: &str) -> &'static str {
        match op {
            "<" => ">=",
            "<=" => ">",
            ">" => "<=",
            ">=" => "<",
            "==" => "!=",
            _ => "==",
        }
    }

    /// Go to `to` unless `e` is true: when it is false or unknown.
    fn branch_false(&mut self, e: &Expr, to: &str) -> R<()> {
        match &e.kind {
            ExprKind::Binary(op, a, b) if Self::comparison(op).is_some() => {
                let (x, y, base) = self.compare_operands(op, a, b, e.line)?;
                let br = Self::comparison(Self::inverse(op)).unwrap();
                self.emit(format!("{br}   {x}, {y}, {to}"));
                self.stack.truncate(base);
            }
            ExprKind::Binary("&&", a, b) => {
                self.branch_false(a, to)?;
                self.branch_false(b, to)?;
            }
            ExprKind::Binary("||", a, b) => {
                let yes = self.new_label();
                self.branch_true(a, &yes)?;
                self.branch_false(b, to)?;
                self.label(&yes);
            }
            ExprKind::Bool(1) => {}
            ExprKind::Bool(_) => self.emit(format!("j     {to}")),
            ExprKind::Unary("!", x) => {
                // !x is true only when x is false.
                let (base, r) = self.bool_operand(x)?;
                self.emit(format!("bgez  {r}, {to}"));
                self.stack.truncate(base);
            }
            _ => {
                let (base, r) = self.bool_operand(e)?;
                self.emit(format!("blez  {r}, {to}"));
                self.stack.truncate(base);
            }
        }
        Ok(())
    }

    /// Go to `to` when `e` is true.
    fn branch_true(&mut self, e: &Expr, to: &str) -> R<()> {
        match &e.kind {
            ExprKind::Binary(op, a, b) if Self::comparison(op).is_some() => {
                let (x, y, base) = self.compare_operands(op, a, b, e.line)?;
                let br = Self::comparison(op).unwrap();
                self.emit(format!("{br}   {x}, {y}, {to}"));
                self.stack.truncate(base);
            }
            ExprKind::Binary("&&", a, b) => {
                let no = self.new_label();
                self.branch_false(a, &no)?;
                self.branch_true(b, to)?;
                self.label(&no);
            }
            ExprKind::Binary("||", a, b) => {
                self.branch_true(a, to)?;
                self.branch_true(b, to)?;
            }
            ExprKind::Bool(1) => self.emit(format!("j     {to}")),
            ExprKind::Bool(_) => {}
            ExprKind::Unary("!", x) => {
                let (base, r) = self.bool_operand(x)?;
                self.emit(format!("bltz  {r}, {to}"));
                self.stack.truncate(base);
            }
            _ => {
                let (base, r) = self.bool_operand(e)?;
                self.emit(format!("bgtz  {r}, {to}"));
                self.stack.truncate(base);
            }
        }
        Ok(())
    }

    /// A condition's value in a register (`operand`), checked to be a
    /// bool, with the depth to return to.
    fn bool_operand(&mut self, e: &Expr) -> R<(usize, &'static str)> {
        let base = self.stack.len();
        let (t, r) = self.operand(e, None)?;
        if t != Type::Bool {
            let hint = if t.numeric() { ": compare it, as in x != 0" } else { "" };
            return self.err(e.line, format!("a condition must be a bool, not {}{hint}", t.a()));
        }
        Ok((base, r))
    }

    fn bool_value(&mut self, e: &Expr) -> R<()> {
        let t = self.expr(e)?;
        if t != Type::Bool {
            let hint = if t.numeric() { ": compare it, as in x != 0" } else { "" };
            return self.err(e.line, format!("a condition must be a bool, not {}{hint}", t.a()));
        }
        Ok(())
    }

    /// Both sides of a comparison, checked: their registers, and the
    /// depth to return to once they are used.
    fn compare_operands(
        &mut self,
        op: &str,
        a: &Expr,
        b: &Expr,
        line: usize,
    ) -> R<(&'static str, &'static str, usize)> {
        if literal_minus_one(a) || literal_minus_one(b) {
            self.warnings.push(format!(
                "{}:{line}: warning: -1 is an ordinary value: test the sign or compare with a named constant",
                self.file
            ));
        }
        let base = self.stack.len();
        let (ta, x) = self.operand(a, Some(b))?;
        let (tb, y) = self.operand(b, None)?;
        let ok = (ta.numeric() && tb.numeric())
            || (ta == Type::Bool && tb == Type::Bool && (op == "==" || op == "!="))
            || (ta.is_ptr() && (ta == tb || is_zero_literal(b)))
            || (tb.is_ptr() && is_zero_literal(a));
        if !ok {
            return self.err(line, format!("cannot compare {} with {}", ta.a(), tb.a()));
        }
        Ok((x, y, base))
    }

    /// The value of `e` in a register without copying it when it has
    /// one: a register local's own, `zero` for the constant 0, else a
    /// new temporary. A register local is copied after all when `later`,
    /// evaluated before the value is used, may assign to it.
    fn operand(&mut self, e: &Expr, later: Option<&Expr>) -> R<(Type, &'static str)> {
        if let ExprKind::Name(n) = &e.kind
            && let Some(Place::Local(Local { ty, reg: Some(reg), .. })) = self.lookup(n)
            && !later.is_some_and(|l| writes(l, n))
        {
            return Ok((ty, reg));
        }
        if self.try_const(e) == Some(0) {
            return Ok((Type::Int, "zero"));
        }
        let t = self.expr(e)?;
        Ok((t, self.top()))
    }

    /// The register a result goes in, once the operands above `base`
    /// are used: the first of them, or a new temporary.
    fn result_reg(&mut self, base: usize, line: usize) -> R<&'static str> {
        self.stack.truncate(base);
        self.push(line)
    }

    /// Whether `e` can be put straight into a register with no
    /// temporaries: a constant, a string or a variable.
    fn is_simple(&self, e: &Expr) -> bool {
        match &e.kind {
            ExprKind::Int(_) | ExprKind::Bool(_) | ExprKind::Str(_) => true,
            ExprKind::Name(n) => self.lookup(n).is_some(),
            _ => self.try_const(e).is_some(),
        }
    }

    /// Put the simple expression `e` (is_simple) in register `dst`.
    fn simple_into(&mut self, e: &Expr, dst: &str) -> R<Type> {
        if let ExprKind::Name(n) = &e.kind {
            return Ok(match self.lookup(n) {
                Some(Place::Local(Local { ty, reg: Some(reg), .. })) => {
                    self.emit(format!("mv    {dst}, {reg}"));
                    ty
                }
                Some(Place::Local(Local { ty, off, .. })) => {
                    if ty.aggregate() {
                        self.emit(format!("addi  {dst}, s0, {off}"));
                    } else {
                        self.load(&ty, dst, off, "s0");
                    }
                    ty.decay()
                }
                Some(Place::Global(ty)) if self.gp && !ty.aggregate() => {
                    let op = if ty.size() == 1 { "lt" } else { "lw" };
                    self.emit(format!("{op}    {dst}, {n}"));
                    ty
                }
                Some(Place::Global(ty)) => {
                    self.emit(format!("la    {dst}, {n}"));
                    if !ty.aggregate() {
                        self.load(&ty, dst, 0, dst);
                    }
                    ty.decay()
                }
                Some(Place::Const(v)) => {
                    self.emit(format!("li    {dst}, {v}"));
                    Type::Int
                }
                Some(Place::Imported(name)) => {
                    self.emit(format!("li    {dst}, {name}"));
                    Type::Int
                }
                None => return self.err(e.line, format!("unknown name {n}")),
            });
        }
        Ok(match &e.kind {
            ExprKind::Bool(v) => {
                self.emit(format!("li    {dst}, {v}"));
                Type::Bool
            }
            ExprKind::Str(cps) => {
                let label = self.string(cps);
                self.emit(format!("la    {dst}, {label}"));
                Type::Ptr(Box::new(Type::Tryte))
            }
            _ => {
                let v = self.const_eval(e)?;
                self.emit(format!("li    {dst}, {v}"));
                Type::Int
            }
        })
    }

    /// Evaluate `e` for its effect only, as a statement does: an
    /// assignment or step of a register local needs no copy of its value.
    fn effect(&mut self, e: &Expr) -> R<()> {
        let reg_local = |g: &Self, x: &Expr| match &x.kind {
            ExprKind::Name(n) => match g.lookup(n) {
                Some(Place::Local(l)) if l.reg.is_some() => Some(l),
                _ => None,
            },
            _ => None,
        };
        match &e.kind {
            ExprKind::Step { op, target, .. } if reg_local(self, target).is_some() => {
                let l = reg_local(self, target).unwrap();
                let reg = l.reg.unwrap();
                let amount = match &l.ty {
                    t if t.numeric() => 1,
                    Type::Ptr(p) if **p != Type::Void => p.size(),
                    t => return self.err(e.line, format!("cannot {op} {}", t.a())),
                };
                let amount = if *op == "++" { amount } else { -amount };
                self.emit(format!("addi  {reg}, {reg}, {amount}"));
                self.narrow(&l.ty, &Type::Int, reg);
                Ok(())
            }
            ExprKind::Assign("=", l, r) if reg_local(self, l).is_some() && self.is_simple(r) => {
                let local = reg_local(self, l).unwrap();
                let reg = local.reg.unwrap();
                let t = self.simple_into(r, reg)?;
                self.check_assign(&local.ty, &t, r)?;
                self.narrow(&local.ty, &t, reg);
                Ok(())
            }
            _ => {
                self.expr(e)?;
                // The value of an assignment or step is not wanted.
                let copy = format!("        mv    {},", self.top());
                if matches!(e.kind, ExprKind::Assign(..) | ExprKind::Step { .. })
                    && self.code.last().is_some_and(|l| l.starts_with(&copy))
                {
                    self.code.pop();
                }
                self.pop();
                Ok(())
            }
        }
    }

    // --- expressions -------------------------------------------------------

    fn check_assign(&self, dst: &Type, src: &Type, e: &Expr) -> R<()> {
        let ok = match (dst, src) {
            (d, s) if d.numeric() && s.numeric() => true,
            (Type::Bool, Type::Bool) => true,
            (Type::Struct(a), Type::Struct(b)) => a == b,
            (Type::Ptr(a), Type::Ptr(b)) => a == b || **a == Type::Void || **b == Type::Void,
            (Type::Ptr(_), _) => is_zero_literal(e),
            _ => false,
        };
        if !ok {
            return self.err(e.line, format!("cannot use {} as {}", src.a(), dst.a()));
        }
        Ok(())
    }

    /// Evaluate `e` into a new temporary; its type, arrays decayed.
    fn expr(&mut self, e: &Expr) -> R<Type> {
        let line = e.line;
        match &e.kind {
            ExprKind::Int(v) => {
                let r = self.push(line)?;
                self.emit(format!("li    {r}, {v}"));
                Ok(Type::Int)
            }
            ExprKind::Bool(v) => {
                let r = self.push(line)?;
                self.emit(format!("li    {r}, {v}"));
                Ok(Type::Bool)
            }
            ExprKind::Str(s) => {
                let label = self.string(s);
                let r = self.push(line)?;
                self.emit(format!("la    {r}, {label}"));
                Ok(Type::Ptr(Box::new(Type::Tryte)))
            }
            ExprKind::Name(n) => self.name_value(n, line),
            ExprKind::Unary(op, x) => self.unary(op, x, line),
            ExprKind::Binary(op, a, b) => self.binary(op, a, b, line),
            ExprKind::Assign(op, l, r) => self.assign(op, l, r, line),
            ExprKind::Cond(c, a, b) => {
                let (other, end) = (self.new_label(), self.new_label());
                self.branch_false(c, &other)?;
                let base = self.stack.len();
                let ta = self.expr(a)?;
                let result = self.top();
                self.emit(format!("j     {end}"));
                self.stack.truncate(base);
                self.label(&other);
                let tb = self.expr(b)?;
                // Both sides leave the value in the same register.
                let rb = self.top();
                self.emit(format!("mv    {result}, {rb}"));
                self.stack.pop();
                self.stack.push(result);
                self.label(&end);
                if ta == tb {
                    Ok(ta)
                } else if ta.numeric() && tb.numeric() {
                    Ok(Type::Int)
                } else if ta.is_ptr() && is_zero_literal(b) {
                    Ok(ta)
                } else if tb.is_ptr() && is_zero_literal(a) {
                    Ok(tb)
                } else {
                    self.err(line, format!("the two sides are {} and {}", ta.a(), tb.a()))
                }
            }
            ExprKind::Call(name, args) => self.call(name, args, line),
            ExprKind::List(_) => {
                self.err(line, "a list in braces only initialises an array or a struct")
            }
            ExprKind::Index(..) | ExprKind::Field(..) => {
                let t = self.addr(e)?;
                if !t.aggregate() {
                    let r = self.top();
                    self.load(&t, r, 0, r);
                }
                Ok(t.decay())
            }
            ExprKind::Cast(t, x) => self.cast(t, x, line),
            ExprKind::SizeofType(t) => {
                let r = self.push(line)?;
                self.emit(format!("li    {r}, {}", t.size()));
                Ok(Type::Int)
            }
            ExprKind::SizeofExpr(x) => {
                let t = self.type_of(x)?;
                let r = self.push(line)?;
                self.emit(format!("li    {r}, {}", t.size()));
                Ok(Type::Int)
            }
            ExprKind::Step { op, post, target } => self.step(op, *post, target, line),
        }
    }

    /// The type of `x`, without arrays decaying and without its code.
    fn type_of(&mut self, x: &Expr) -> R<Type> {
        let (code, depth, strings, data, warnings) =
            (self.code.len(), self.stack.len(), self.strings, self.data.len(), self.warnings.len());
        if let ExprKind::Name(n) = &x.kind
            && let Some(Place::Local(l)) = self.lookup(n)
        {
            return Ok(l.ty);
        }
        let t = match &x.kind {
            ExprKind::Name(_)
            | ExprKind::Index(..)
            | ExprKind::Unary("*", _)
            | ExprKind::Field(..) => self.addr(x),
            _ => self.expr(x),
        };
        self.code.truncate(code);
        self.data.truncate(data);
        self.stack.truncate(depth);
        self.strings = strings;
        self.warnings.truncate(warnings);
        t
    }

    fn name_value(&mut self, n: &str, line: usize) -> R<Type> {
        let Some(place) = self.lookup(n) else {
            if self.funcs.contains_key(n) {
                return self.err(line, format!("{n} is a function: call it"));
            }
            return self.err(line, format!("unknown name {n}"));
        };
        let r = self.push(line)?;
        match place {
            Place::Local(Local { ty, reg: Some(reg), .. }) => {
                self.emit(format!("mv    {r}, {reg}"));
                Ok(ty)
            }
            Place::Local(Local { ty, off, .. }) => {
                if ty.aggregate() {
                    self.emit(format!("addi  {r}, s0, {off}"));
                } else {
                    self.load(&ty, r, off, "s0");
                }
                Ok(ty.decay())
            }
            Place::Global(ty) => {
                if self.gp && !ty.aggregate() {
                    let op = if ty.size() == 1 { "lt" } else { "lw" };
                    self.emit(format!("{op}    {r}, {n}"));
                    return Ok(ty);
                }
                self.emit(format!("la    {r}, {n}"));
                if !ty.aggregate() {
                    self.load(&ty, r, 0, r);
                }
                Ok(ty.decay())
            }
            Place::Const(v) => {
                self.emit(format!("li    {r}, {v}"));
                Ok(Type::Int)
            }
            Place::Imported(name) => {
                self.emit(format!("li    {r}, {name}"));
                Ok(Type::Int)
            }
        }
    }

    /// Push the address of the object `e` names; the object's type.
    fn addr(&mut self, e: &Expr) -> R<Type> {
        let line = e.line;
        match &e.kind {
            ExprKind::Name(n) => match self.lookup(n) {
                Some(Place::Local(Local { reg: Some(_), .. })) => {
                    self.err(line, format!("internal error: {n} is in a register"))
                }
                Some(Place::Local(Local { ty, off, .. })) => {
                    let r = self.push(line)?;
                    self.emit(format!("addi  {r}, s0, {off}"));
                    Ok(ty)
                }
                Some(Place::Global(ty)) => {
                    let r = self.push(line)?;
                    self.emit(format!("la    {r}, {n}"));
                    Ok(ty)
                }
                Some(_) => self.err(line, format!("{n} is a constant, not a variable")),
                None => self.err(line, format!("unknown name {n}")),
            },
            ExprKind::Unary("*", x) => {
                let t = self.expr(x)?;
                match t {
                    Type::Ptr(inner) if *inner != Type::Void => Ok(*inner),
                    Type::Ptr(_) => self.err(line, "cannot follow a void*"),
                    t => self.err(line, format!("cannot follow {}: it is not a pointer", t.a())),
                }
            }
            ExprKind::Index(a, i) => {
                let base = self.stack.len();
                let (ta, rb) = self.operand(a, Some(i))?;
                let Type::Ptr(elem) = ta else {
                    return self.err(line, format!("cannot index {}", ta.a()));
                };
                if *elem == Type::Void {
                    return self.err(line, "cannot index a void*");
                }
                let size = elem.size();
                if let Some(k) = self.try_const(i).filter(|k| (k * size).abs() <= IMM_MAX) {
                    if !(k == 0 && self.is_temp(rb)) {
                        let d = self.result_reg(base, line)?;
                        self.emit(format!("addi  {d}, {rb}, {}", k * size));
                    }
                } else {
                    let (ti, ri) = self.operand(i, None)?;
                    if !ti.numeric() {
                        return self.err(line, format!("an index is a number, not {}", ti.a()));
                    }
                    let scaled = if size == 1 {
                        ri
                    } else {
                        let tmp = if self.is_temp(ri) { ri } else { self.push(line)? };
                        self.scale_into(tmp, ri, size, line)?;
                        tmp
                    };
                    let d = self.result_reg(base, line)?;
                    self.emit(format!("add   {d}, {rb}, {scaled}"));
                }
                Ok(*elem)
            }
            ExprKind::Field(base, field, arrow) => {
                let t = self.expr(base)?;
                let name = match (&t, arrow) {
                    (Type::Struct(n), false) => n.clone(),
                    (Type::Ptr(inner), true) => match &**inner {
                        Type::Struct(n) if inner.complete() => n.clone(),
                        Type::Struct(n) => {
                            return self.err(line, format!("struct {n} is not defined"));
                        }
                        _ => {
                            return self.err(
                                line,
                                format!("-> follows a pointer to a struct, not {}", t.a()),
                            );
                        }
                    },
                    (Type::Ptr(inner), false) if matches!(**inner, Type::Struct(_)) => {
                        return self.err(line, format!("{} is a pointer: use ->{field}", t.name()));
                    }
                    (_, false) => {
                        return self.err(line, format!(".{field} needs a struct, not {}", t.a()));
                    }
                    (_, true) => {
                        return self
                            .err(line, format!("-> follows a pointer to a struct, not {}", t.a()));
                    }
                };
                let fields = layout(&name).map(|l| l.fields).unwrap_or_default();
                let Some((_, fty, off)) = fields.into_iter().find(|(n, _, _)| n == field) else {
                    return self.err(line, format!("struct {name} has no field {field}"));
                };
                if off != 0 {
                    let r = self.top();
                    self.emit(format!("addi  {r}, {r}, {off}"));
                }
                Ok(fty)
            }
            _ => self.err(line, "this is not something with an address"),
        }
    }

    /// `dst` = `src` times an element size.
    fn scale_into(&mut self, dst: &str, src: &str, size: i128, line: usize) -> R<()> {
        match size {
            1 => self.emit(format!("mv    {dst}, {src}")),
            3 => self.emit(format!("shi   {dst}, {src}, 1")),
            _ => {
                let k = self.push(line)?;
                self.emit(format!("li    {k}, {size}"));
                self.emit(format!("mul   {dst}, {src}, {k}"));
                self.pop();
            }
        }
        Ok(())
    }

    /// Put two operand registers, from `operand` after `base`, on the
    /// temporaries in order: the left at `base`, the right above it.
    fn onto_stack(
        &mut self,
        base: usize,
        ra: &'static str,
        rb: &'static str,
        line: usize,
    ) -> R<()> {
        // Fresh registers: the moves cost nothing once allocation joins
        // them to their sources.
        self.stack.truncate(base);
        let t0 = self.push(line)?;
        let t1 = self.push(line)?;
        self.emit(format!("mv    {t1}, {rb}"));
        self.emit(format!("mv    {t0}, {ra}"));
        Ok(())
    }

    /// Multiply the top temporary by an element size.
    fn scale(&mut self, size: i128, line: usize) -> R<()> {
        let r = self.top();
        match size {
            1 => {}
            3 => self.emit(format!("shi   {r}, {r}, 1")),
            _ => {
                let k = self.push(line)?;
                self.emit(format!("li    {k}, {size}"));
                self.emit(format!("mul   {r}, {r}, {k}"));
                self.pop();
            }
        }
        Ok(())
    }

    fn unary(&mut self, op: &str, x: &Expr, line: usize) -> R<Type> {
        match op {
            "-" => {
                if let Some(v) = self.try_const(x) {
                    let r = self.push(line)?;
                    self.emit(format!("li    {r}, {}", -v));
                    return Ok(Type::Int);
                }
                let t = self.expr(x)?;
                if !t.numeric() {
                    let hint = if t == Type::Bool { ": use ! on a bool" } else { "" };
                    return self.err(line, format!("cannot negate {}{hint}", t.a()));
                }
                let r = self.top();
                self.emit(format!("neg   {r}, {r}"));
                Ok(if t == Type::Trit { Type::Trit } else { Type::Int })
            }
            "!" => {
                let t = self.expr(x)?;
                if t != Type::Bool {
                    return self.err(
                        line,
                        format!("! takes a bool, not {}: use - to negate a number", t.a()),
                    );
                }
                let r = self.top();
                self.emit(format!("neg   {r}, {r}"));
                Ok(Type::Bool)
            }
            "*" => {
                let e = Expr { kind: ExprKind::Unary("*", Box::new(x.clone())), line };
                let t = self.addr(&e)?;
                if !t.aggregate() {
                    let r = self.top();
                    self.load(&t, r, 0, r);
                }
                Ok(t.decay())
            }
            "&" => {
                let t = self.addr(x)?;
                Ok(Type::Ptr(Box::new(t)))
            }
            _ => self.err(line, format!("unknown operator {op}")),
        }
    }

    fn binary(&mut self, op: &'static str, a: &Expr, b: &Expr, line: usize) -> R<Type> {
        if Self::comparison(op).is_some() {
            let (x, y, base) = self.compare_operands(op, a, b, line)?;
            let d = self.result_reg(base, line)?;
            self.materialise(op, d, x, y);
            return Ok(Type::Bool);
        }
        match op {
            "&&" | "||" => {
                self.bool_value(a)?;
                let skip = self.new_label();
                let r = self.top();
                // Kleene: false && x is false, true || x is true.
                let br = if op == "&&" { "bltz" } else { "bgtz" };
                self.emit(format!("{br}  {r}, {skip}"));
                self.bool_value(b)?;
                let ins = if op == "&&" { "min" } else { "max" };
                let (x, y) = (self.second(), self.top());
                self.emit(format!("{ins}   {x}, {x}, {y}"));
                self.pop();
                self.label(&skip);
                Ok(Type::Bool)
            }
            "<=>" => {
                let ta = self.expr(a)?;
                let tb = self.expr(b)?;
                if !((ta.numeric() && tb.numeric()) || (ta.is_ptr() && ta == tb)) {
                    return self.err(line, format!("cannot compare {} with {}", ta.a(), tb.a()));
                }
                let (x, y) = (self.second(), self.top());
                self.emit(format!("cmp   {x}, {x}, {y}"));
                self.pop();
                Ok(Type::Trit)
            }
            _ => {
                let base = self.stack.len();
                let (ta, mut ra) = self.operand(a, Some(b))?;
                if let Some(k) = self.try_const(b)
                    && ta.numeric()
                {
                    let form = match op {
                        "+" if k.abs() <= IMM_MAX => Some(("addi", k)),
                        "-" if k.abs() <= IMM_MAX => Some(("addi", -k)),
                        "<<" if (0..=27).contains(&k) => Some(("shi ", k)),
                        ">>" if (0..=27).contains(&k) => Some(("shi ", -k)),
                        _ => None,
                    };
                    if let Some((ins, v)) = form {
                        let d = self.result_reg(base, line)?;
                        self.emit(format!("{ins}  {d}, {ra}, {v}"));
                        return Ok(Type::Int);
                    }
                }
                if ta.is_ptr() && self.try_const(b).is_some() {
                    if !self.is_temp(ra) {
                        let t = self.result_reg(base, line)?;
                        self.emit(format!("mv    {t}, {ra}"));
                        ra = t;
                    }
                    if let Some(t) = self.immediate(op, &ta, b, line)? {
                        return Ok(t);
                    }
                }
                let (tb, rb) = self.operand(b, None)?;
                let plain = ["+", "-", "*", "/", "%", "<<", ">>", "&", "|"];
                if ta.numeric() && tb.numeric() && plain.contains(&op) {
                    let rb = if op == ">>" {
                        let n = if self.is_temp(rb) { rb } else { self.push(line)? };
                        self.emit(format!("neg   {n}, {rb}"));
                        n
                    } else {
                        rb
                    };
                    let ins = match op {
                        "+" => "add",
                        "-" => "sub",
                        "*" => "mul",
                        "/" => "div",
                        "%" => "rem",
                        "<<" | ">>" => "sh",
                        "&" => "tmin",
                        _ => "tmax",
                    };
                    let d = self.result_reg(base, line)?;
                    self.emit(format!("{ins:<5} {d}, {ra}, {rb}"));
                    return Ok(Type::Int);
                }
                self.onto_stack(base, ra, rb, line)?;
                self.arith(op, &ta, &tb, line)
            }
        }
    }

    /// `op` with a constant right side that fits an immediate, when
    /// one instruction does it: the left side is already in the top
    /// temporary.
    fn immediate(&mut self, op: &str, ta: &Type, b: &Expr, _line: usize) -> R<Option<Type>> {
        let Some(k) = self.try_const(b) else { return Ok(None) };
        let r = self.top();
        let scaled = match ta {
            Type::Ptr(e) if (op == "+" || op == "-") && **e != Type::Void => k * e.size(),
            t if t.numeric() => k,
            _ => return Ok(None),
        };
        let result = if ta.is_ptr() { ta.clone() } else { Type::Int };
        match op {
            "+" | "-" if scaled.abs() <= IMM_MAX => {
                let v = if op == "+" { scaled } else { -scaled };
                self.emit(format!("addi  {r}, {r}, {v}"));
            }
            "<<" if ta.numeric() && (0..=27).contains(&k) => {
                self.emit(format!("shi   {r}, {r}, {k}"))
            }
            ">>" if ta.numeric() && (0..=27).contains(&k) => {
                self.emit(format!("shi   {r}, {r}, {}", -k))
            }
            _ => return Ok(None),
        }
        Ok(Some(result))
    }

    /// `op` on the top two temporaries, left below right: the result
    /// replaces the left.
    fn arith(&mut self, op: &str, ta: &Type, tb: &Type, line: usize) -> R<Type> {
        let bad = |g: &Self| g.err(line, format!("{op} does not take {} and {}", ta.a(), tb.a()));
        let result = match op {
            "+" | "-" if ta.numeric() && tb.numeric() => Type::Int,
            "+" if ta.is_ptr() && tb.numeric() => {
                self.scale(pointee_size(ta), line)?;
                ta.clone()
            }
            "+" if ta.numeric() && tb.is_ptr() => {
                let size = pointee_size(tb);
                let x = self.second();
                match size {
                    0 => return bad(self),
                    1 => {}
                    3 => self.emit(format!("shi   {x}, {x}, 1")),
                    _ => {
                        let k = self.push(line)?;
                        self.emit(format!("li    {k}, {size}"));
                        self.emit(format!("mul   {x}, {x}, {k}"));
                        self.pop();
                    }
                }
                tb.clone()
            }
            "-" if ta.is_ptr() && tb.numeric() => {
                self.scale(pointee_size(ta), line)?;
                ta.clone()
            }
            "-" if ta.is_ptr() && ta == tb => {
                let (x, y) = (self.second(), self.top());
                self.emit(format!("sub   {x}, {x}, {y}"));
                self.pop();
                match pointee_size(ta) {
                    0 => return bad(self),
                    1 => {}
                    3 => self.emit(format!("shi   {x}, {x}, -1")),
                    size => {
                        let k = self.push(line)?;
                        self.emit(format!("li    {k}, {size}"));
                        self.emit(format!("div   {x}, {x}, {k}"));
                        self.pop();
                    }
                }
                return Ok(Type::Int);
            }
            "*" | "/" | "%" | "<<" | ">>" if ta.numeric() && tb.numeric() => Type::Int,
            "&" | "|" if ta.numeric() && tb.numeric() => Type::Int,
            "&" | "|" if *ta == Type::Bool && *tb == Type::Bool => Type::Bool,
            _ => return bad(self),
        };
        if pointee_size(&result) == 0 && result.is_ptr() {
            return self.err(line, "no arithmetic on a void*");
        }
        let (x, y) = (self.second(), self.top());
        let ins = match op {
            "+" => "add",
            "-" => "sub",
            "*" => "mul",
            "/" => "div",
            "%" => "rem",
            "<<" => "sh",
            ">>" => {
                self.emit(format!("neg   {y}, {y}"));
                "sh"
            }
            "&" if result == Type::Bool => "min",
            "|" if result == Type::Bool => "max",
            "&" => "tmin",
            _ => "tmax",
        };
        self.emit(format!("{ins:<5} {x}, {x}, {y}"));
        self.pop();
        Ok(result)
    }

    /// The top two temporaries compared, as a bool in the left one,
    /// with no branch (so the assembler can still bundle it).
    fn materialise(&mut self, op: &str, d: &str, x: &str, y: &str) {
        match op {
            // c = cmp(x, y); x > y is sign(2c - 1), x >= y is sign(2c + 1).
            ">" | ">=" | "<" | "<=" => {
                let (l, r) = if op.starts_with('>') { (x, y) } else { (y, x) };
                self.emit(format!("cmp   {d}, {l}, {r}"));
                self.emit(format!("add   {d}, {d}, {d}"));
                let k = if op.ends_with('=') { -1 } else { 1 };
                self.emit(format!("cmpi  {d}, {d}, {k}"));
            }
            // |c| is 1 when they differ: != is sign(2|c| - 1).
            _ => {
                self.emit(format!("cmp   {d}, {x}, {y}"));
                self.emit(format!("tabs  {d}, {d}"));
                self.emit(format!("add   {d}, {d}, {d}"));
                self.emit(format!("cmpi  {d}, {d}, 1"));
                if op == "==" {
                    self.emit(format!("neg   {d}, {d}"));
                }
            }
        }
    }

    fn assign(&mut self, op: &str, l: &Expr, r: &Expr, line: usize) -> R<Type> {
        if op == "=" {
            if let ExprKind::Name(n) = &l.kind
                && let Some(Place::Local(local)) = self.lookup(n)
                && !local.ty.aggregate()
            {
                let t = self.expr(r)?;
                self.check_assign(&local.ty, &t, r)?;
                let v = self.top();
                self.store_local(&local, v, &t);
                if let Some(reg) = local.reg
                    && local.ty != t
                {
                    self.emit(format!("mv    {v}, {reg}"));
                }
                return Ok(local.ty);
            }
            if let Some(ty) = self.gp_global(l) {
                let base = self.stack.len();
                let (t, mut v) = self.operand(r, None)?;
                self.check_assign(&ty, &t, r)?;
                if ty == Type::Trit && t != Type::Trit && !self.is_temp(v) && v != "zero" {
                    let d = self.result_reg(base, line)?;
                    self.emit(format!("mv    {d}, {v}"));
                    v = d;
                }
                if self.is_temp(v) {
                    self.narrow(&ty, &t, v);
                }
                let op = if ty.size() == 1 { "st" } else { "sw" };
                let ExprKind::Name(n) = &l.kind else { unreachable!() };
                self.emit(format!("{op}    {v}, {n}"));
                if !self.is_temp(v) {
                    let d = self.result_reg(base, line)?;
                    self.emit(format!("mv    {d}, {v}"));
                }
                return Ok(ty);
            }
            let tl = self.addr(l)?;
            if matches!(tl, Type::Array(..)) {
                return self.err(line, "cannot assign to a whole array");
            }
            let t = self.expr(r)?;
            self.check_assign(&tl, &t, r)?;
            let (a, v) = (self.second(), self.top());
            if matches!(tl, Type::Struct(_)) {
                // The value is the address of the struct to copy.
                self.copy(&tl, a, v, line)?;
                self.pop();
                return Ok(tl);
            }
            self.store(&tl, v, 0, a);
            self.emit(format!("mv    {a}, {v}"));
            self.pop();
            return Ok(tl);
        }
        let base = &op[..op.len() - 1];
        if let ExprKind::Name(n) = &l.kind
            && let Some(Place::Local(local)) = self.lookup(n)
            && let Some(reg) = local.reg
        {
            let v = self.push(line)?;
            self.emit(format!("mv    {v}, {reg}"));
            let t = match self.immediate(base, &local.ty, r, line)? {
                Some(t) => t,
                None => {
                    let tr = self.expr(r)?;
                    self.arith(base, &local.ty, &tr, line)?
                }
            };
            self.check_assign(&local.ty, &t, r)?;
            self.narrow(&local.ty, &t, v);
            self.emit(format!("mv    {reg}, {v}"));
            return Ok(local.ty);
        }
        if let Some(ty) = self.gp_global(l) {
            let ExprKind::Name(n) = &l.kind else { unreachable!() };
            let (lo, so) = if ty.size() == 1 { ("lt", "st") } else { ("lw", "sw") };
            let v = self.push(line)?;
            self.emit(format!("{lo}    {v}, {n}"));
            let t = match self.immediate(base, &ty, r, line)? {
                Some(t) => t,
                None => {
                    let tr = self.expr(r)?;
                    self.arith(base, &ty, &tr, line)?
                }
            };
            self.check_assign(&ty, &t, r)?;
            self.emit(format!("{so}    {v}, {n}"));
            return Ok(ty);
        }
        let tl = self.addr(l)?;
        if tl.aggregate() {
            return self.err(line, format!("{op} does not take {}", tl.a()));
        }
        let a = self.top();
        let v = self.push(line)?;
        self.load(&tl, v, 0, a);
        let t = match self.immediate(base, &tl, r, line)? {
            Some(t) => t,
            None => {
                let tr = self.expr(r)?;
                self.arith(base, &tl, &tr, line)?
            }
        };
        self.check_assign(&tl, &t, r)?;
        self.store(&tl, v, 0, a);
        self.emit(format!("mv    {a}, {v}"));
        self.pop();
        Ok(tl)
    }

    /// Copy the struct of type `ty` at the address in `src` to the
    /// address in `dst`, a word or a tryte at a time.
    fn copy(&mut self, ty: &Type, dst: &str, src: &str, line: usize) -> R<()> {
        let (unit, op_l, op_s) = if ty.word_aligned() { (3, "lw", "sw") } else { (1, "lt", "st") };
        let n = ty.size() / unit;
        let tmp = self.push(line)?;
        if n <= 9 {
            for k in 0..n {
                self.emit(format!("{op_l}    {tmp}, {}({src})", k * unit));
                self.emit(format!("{op_s}    {tmp}, {}({dst})", k * unit));
            }
        } else {
            // A loop for a large struct, moving copies of the addresses.
            let (d, s2, left) = (self.push(line)?, self.push(line)?, self.push(line)?);
            self.emit(format!("mv    {d}, {dst}"));
            self.emit(format!("mv    {s2}, {src}"));
            self.emit(format!("li    {left}, {n}"));
            let top = self.new_label();
            self.label(&top);
            self.emit(format!("{op_l}    {tmp}, 0({s2})"));
            self.emit(format!("{op_s}    {tmp}, 0({d})"));
            self.emit(format!("addi  {s2}, {s2}, {unit}"));
            self.emit(format!("addi  {d}, {d}, {unit}"));
            self.emit(format!("addi  {left}, {left}, -1"));
            self.emit(format!("bgtz  {left}, {top}"));
            self.pop();
            self.pop();
            self.pop();
        }
        self.pop();
        Ok(())
    }

    /// Initialise the local of type `ty` at `off` from `s0` with `init`:
    /// what a list leaves out becomes 0.
    fn init_local(&mut self, ty: &Type, off: i128, init: &Expr, line: usize) -> R<()> {
        match (ty, &init.kind) {
            (Type::Array(elem, n), ExprKind::List(items)) => {
                if items.len() as i128 > *n {
                    return self
                        .err(line, format!("{} initialisers for {} elements", items.len(), n));
                }
                for k in 0..*n {
                    let at = off + k * elem.size();
                    match items.get(k as usize) {
                        Some(item) => self.init_local(elem, at, item, line)?,
                        None => self.zero_fill(elem, at),
                    }
                }
            }
            (Type::Array(elem, n), ExprKind::Str(cps)) if **elem == Type::Tryte => {
                if cps.len() as i128 >= *n {
                    return self.err(
                        line,
                        format!("the string needs {} trytes with its 0", cps.len() + 1),
                    );
                }
                let r = self.push(line)?;
                for k in 0..*n {
                    match cps.get(k as usize) {
                        Some(cp) => {
                            self.emit(format!("li    {r}, {cp}"));
                            self.emit(format!("st    {r}, {}(s0)", off + k));
                        }
                        None => self.emit(format!("st    zero, {}(s0)", off + k)),
                    }
                }
                self.pop();
            }
            (Type::Struct(name), ExprKind::List(items)) => {
                let fields = layout(name).map(|l| l.fields).unwrap_or_default();
                if items.len() > fields.len() {
                    return self.err(
                        line,
                        format!("{} initialisers for {} fields", items.len(), fields.len()),
                    );
                }
                for (k, (_, fty, foff)) in fields.iter().enumerate() {
                    match items.get(k) {
                        Some(item) => self.init_local(fty, off + foff, item, line)?,
                        None => self.zero_fill(fty, off + foff),
                    }
                }
            }
            (Type::Struct(_), _) => {
                let t = self.expr(init)?;
                self.check_assign(ty, &t, init)?;
                let dst = self.push(line)?;
                self.emit(format!("addi  {dst}, s0, {off}"));
                let src = self.second();
                self.copy(ty, dst, src, line)?;
                self.pop();
                self.pop();
            }
            (t, _) if t.aggregate() => {
                return self.err(line, format!("{} is initialised with a list in braces", t.a()));
            }
            (_, ExprKind::List(_)) => {
                return self
                    .err(line, format!("{} is initialised with a value, not a list", ty.a()));
            }
            _ => {
                let t = self.expr(init)?;
                self.check_assign(ty, &t, init)?;
                let r = self.top();
                self.store(ty, r, off, "s0");
                self.pop();
            }
        }
        Ok(())
    }

    /// Zero the object of type `ty` at `off` from `s0`.
    fn zero_fill(&mut self, ty: &Type, off: i128) {
        if ty.word_aligned() {
            for k in 0..(ty.size() / 3).max(1) {
                self.emit(format!("sw    zero, {}(s0)", off + 3 * k));
            }
        } else {
            for k in 0..ty.size() {
                self.emit(format!("st    zero, {}(s0)", off + k));
            }
        }
    }

    /// The type of a global scalar named by `e`, when `.gp` lets it be
    /// loaded and stored by name.
    fn gp_global(&self, e: &Expr) -> Option<Type> {
        match &e.kind {
            ExprKind::Name(n) if self.gp => match self.lookup(n) {
                Some(Place::Global(ty)) if !ty.aggregate() => Some(ty),
                _ => None,
            },
            _ => None,
        }
    }

    fn step(&mut self, op: &str, post: bool, target: &Expr, line: usize) -> R<Type> {
        if let Some(t) = self.gp_global(target) {
            let ExprKind::Name(n) = &target.kind else { unreachable!() };
            let amount = match &t {
                t if t.numeric() => 1,
                Type::Ptr(e) if **e != Type::Void => e.size(),
                t => return self.err(line, format!("cannot {op} {}", t.a())),
            };
            let amount = if op == "++" { amount } else { -amount };
            let (lo, so) = if t.size() == 1 { ("lt", "st") } else { ("lw", "sw") };
            let old = self.push(line)?;
            self.emit(format!("{lo}    {old}, {n}"));
            let new = self.push(line)?;
            self.emit(format!("addi  {new}, {old}, {amount}"));
            self.emit(format!("{so}    {new}, {n}"));
            if !post {
                self.emit(format!("mv    {old}, {new}"));
            }
            self.pop();
            return Ok(t);
        }
        if let ExprKind::Name(n) = &target.kind
            && let Some(Place::Local(local)) = self.lookup(n)
            && let Some(reg) = local.reg
        {
            let t = local.ty;
            let amount = match &t {
                t if t.numeric() => 1,
                Type::Ptr(e) if **e != Type::Void => e.size(),
                t => return self.err(line, format!("cannot {op} {}", t.a())),
            };
            let amount = if op == "++" { amount } else { -amount };
            let r = self.push(line)?;
            if post {
                self.emit(format!("mv    {r}, {reg}"));
            }
            self.emit(format!("addi  {reg}, {reg}, {amount}"));
            self.narrow(&t, &Type::Int, reg);
            if !post {
                self.emit(format!("mv    {r}, {reg}"));
            }
            return Ok(t);
        }
        let t = self.addr(target)?;
        let amount = match &t {
            t if t.numeric() => 1,
            Type::Ptr(e) if **e != Type::Void => e.size(),
            t => return self.err(line, format!("cannot {op} {}", t.a())),
        };
        let amount = if op == "++" { amount } else { -amount };
        let a = self.top();
        let old = self.push(line)?;
        self.load(&t, old, 0, a);
        let new = self.push(line)?;
        self.emit(format!("addi  {new}, {old}, {amount}"));
        self.store(&t, new, 0, a);
        let result = if post { old } else { new };
        self.emit(format!("mv    {a}, {result}"));
        self.pop();
        self.pop();
        Ok(t)
    }

    /// `tlut(a, b, T)`, `tex(x, pos, len)` and `tins(x, v, pos, len)`:
    /// one instruction each, the constants in its immediate.
    fn field_builtin(&mut self, name: &str, args: &[Expr], line: usize) -> R<Type> {
        let (values, form) = match name {
            "tlut" => (2, "tlut(a, b, T)"),
            "tex" => (1, "tex(x, pos, len)"),
            _ => (2, "tins(x, v, pos, len)"),
        };
        let consts = if name == "tlut" { 1 } else { 2 };
        if args.len() != values + consts {
            return self.err(line, format!("{name} takes {} arguments: {form}", values + consts));
        }
        let mut imm = Vec::new();
        for a in &args[values..] {
            imm.push(self.const_eval(a)?);
        }
        if name == "tlut" {
            if imm[0].abs() > TABLE_MAX {
                return self.err(args[2].line, "a tlut table is nine trits, -9841 to 9841");
            }
        } else {
            let (pos, len) = (imm[0], imm[1]);
            if pos < 0 || len < 1 || pos + len > 27 {
                return self.err(
                    args[values].line,
                    format!("{name} needs 0 <= pos, 1 <= len and pos + len <= 27"),
                );
            }
        }
        for a in &args[..values] {
            let t = self.expr(a)?;
            if !t.numeric() {
                return self.err(a.line, format!("{name} takes numbers, not {}", t.a()));
            }
        }
        let imm = imm.iter().map(i128::to_string).collect::<Vec<_>>().join(", ");
        if values == 2 {
            // tins writes into its rd, keeping the trits outside the
            // field: x is already there.
            let (x, y) = (self.second(), self.top());
            let srcs = if name == "tins" { y.to_string() } else { format!("{x}, {y}") };
            self.emit(format!("{name:<5} {x}, {srcs}, {imm}"));
            self.pop();
        } else {
            let r = self.top();
            self.emit(format!("{name:<5} {r}, {r}, {imm}"));
        }
        Ok(Type::Int)
    }

    fn cast(&mut self, to: &Type, x: &Expr, line: usize) -> R<Type> {
        let from = self.expr(x)?;
        let r = self.top();
        let ok = match to {
            Type::Void => true,
            Type::Int => from.numeric() || from == Type::Bool || from.is_ptr(),
            Type::Tryte | Type::Trit => {
                let wide = match to {
                    Type::Tryte => from.size() == 3,
                    _ => from.size() == 3 || from == Type::Tryte,
                };
                if wide {
                    let len = if *to == Type::Tryte { 9 } else { 1 };
                    self.emit(format!("tex   {r}, {r}, 0, {len}"));
                }
                from.numeric() || from == Type::Bool || from.is_ptr()
            }
            Type::Bool => matches!(from, Type::Bool | Type::Trit),
            Type::Ptr(_) => from.is_ptr() || from == Type::Int,
            Type::Array(..) | Type::Struct(_) => false,
        };
        if !ok {
            let hint = if *to == Type::Bool { ": write a comparison, as in x != 0" } else { "" };
            return self.err(line, format!("cannot cast {} to {}{hint}", from.a(), to.a()));
        }
        Ok(to.clone())
    }

    fn call(&mut self, name: &str, args: &[Expr], line: usize) -> R<Type> {
        if FIELD.contains(&name) {
            return self.field_builtin(name, args, line);
        }
        if BUILTIN2.contains(&name) || BUILTIN1.contains(&name) {
            let want = if BUILTIN2.contains(&name) { 2 } else { 1 };
            if args.len() != want {
                return self.err(
                    line,
                    format!("{name} takes {want} argument{}", if want == 1 { "" } else { "s" }),
                );
            }
            for a in args {
                let t = self.expr(a)?;
                if !t.numeric() {
                    return self.err(a.line, format!("{name} takes numbers, not {}", t.a()));
                }
            }
            if want == 2 {
                let (x, y) = (self.second(), self.top());
                self.emit(format!("{name:<5} {x}, {x}, {y}"));
                self.pop();
            } else {
                let r = self.top();
                self.emit(format!("{name:<5} {r}, {r}"));
            }
            return Ok(Type::Int);
        }
        let (ret, params) = if name == "sys" {
            if args.is_empty() || args.len() > ARGS.len() {
                return self.err(line, "sys takes one to six arguments");
            }
            (Type::Int, None)
        } else {
            let Some(sig) = self.funcs.get(name).cloned() else {
                return self.err(line, format!("unknown function {name}"));
            };
            if sig.params.len() != args.len() {
                return self.err(
                    line,
                    format!("{name} takes {} arguments, not {}", sig.params.len(), args.len()),
                );
            }
            (sig.ret, Some(sig.params))
        };
        if args.len() > ARGS.len() {
            return self.long_call(name, args, &ret, params.as_deref().unwrap_or_default(), line);
        }
        let base = self.stack.len();
        let check = |g: &Self, i: usize, t: &Type, a: &Expr| match &params {
            Some(p) => g.check_assign(&p[i], t, a),
            None if *t == Type::Void => g.err(a.line, "sys takes values"),
            None => Ok(()),
        };
        // Arguments that need code go to temporaries first, in order;
        // constants and variables go straight to their registers after.
        // A variable read later must not change in between: a register
        // local only by an assignment, memory by a call too.
        let effects = args.iter().any(|a| !self.is_simple(a) && has_effects(a));
        let mut types = vec![Type::Void; args.len()];
        let simple: Vec<bool> = args
            .iter()
            .map(|a| {
                self.is_simple(a)
                    && match &a.kind {
                        ExprKind::Name(n) => match self.lookup(n) {
                            Some(Place::Const(_) | Place::Imported(_)) => true,
                            Some(Place::Local(Local { reg: Some(_), .. })) => {
                                !args.iter().any(|b| writes(b, n))
                            }
                            _ => !effects,
                        },
                        _ => true,
                    }
            })
            .collect();
        for (i, a) in args.iter().enumerate() {
            if !simple[i] {
                let t = self.expr(a)?;
                check(self, i, &t, a)?;
                types[i] = t;
            }
        }
        // The last argument first: a value moving down one register, as
        // when a function passes its own parameters on shifted, then
        // leaves its old register before that register is written.
        for (i, a) in args.iter().enumerate().rev() {
            let next = base + simple[..i].iter().filter(|s| !**s).count();
            if simple[i] {
                types[i] = self.simple_into(a, ARGS[i])?;
                check(self, i, &types[i], a)?;
            } else {
                self.emit(format!("mv    {}, {}", ARGS[i], self.stack[next]));
            }
            // A tryte or trit argument travels cut to its type (abi.md 2.2).
            if let Some(p) = &params {
                self.narrow(&p[i], &types[i], ARGS[i]);
            }
        }
        // The argument count tells the allocator which registers the
        // call reads; it is dropped from the output.
        if name == "sys" {
            self.emit(format!("sys   {}", args.len()));
        } else {
            self.emit(format!("call  {name}, {}", args.len()));
        }
        self.stack.truncate(base);
        let r = self.push(line)?;
        if ret != Type::Void {
            self.emit(format!("mv    {r}, a0"));
        }
        Ok(ret)
    }
}

/// The functions a program uses: those `main` reaches by calls, with
/// those that `asm` blocks or imported assembly name, which may call
/// them, and the ones they reach. None, to keep them all, when there is
/// no `main`.
fn used_functions(units: &[Unit]) -> Option<HashSet<String>> {
    let mut bodies: HashMap<&str, &[Stmt]> = HashMap::new();
    let mut roots = vec!["main".to_string()];
    let words = |text: &str| -> Vec<String> {
        text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .filter(|w| !w.is_empty())
            .map(str::to_string)
            .collect()
    };
    for unit in units {
        match unit {
            Unit::Include { operands, .. } => roots.extend(words(operands)),
            Unit::Item { item: Item::Func(f), .. } => {
                if let Some(body) = &f.body {
                    bodies.insert(&f.name, body);
                }
            }
            Unit::Item { .. } => {}
        }
    }
    if !bodies.contains_key("main") {
        return None;
    }
    let mut used = HashSet::new();
    while let Some(name) = roots.pop() {
        let Some(body) = bodies.get(name.as_str()) else { continue };
        if !used.insert(name) {
            continue;
        }
        for s in body.iter() {
            stmt_calls(s, &mut |n| roots.push(n.to_string()), &words);
        }
    }
    Some(used)
}

/// The most instructions, besides copies, of a function copied into
/// every caller; one called from a single place is copied whatever its
/// size. A call costs about two (the call and, in a leaf, the frame for
/// ra), so copying a function this small rarely makes the program bigger.
const INLINE_MAX: usize = 4;

/// How many calls each function used has in the functions used.
fn call_sites(units: &[Unit], used: Option<&HashSet<String>>) -> HashMap<String, usize> {
    let words = |text: &str| -> Vec<String> {
        text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .filter(|w| !w.is_empty())
            .map(str::to_string)
            .collect()
    };
    let mut sites = HashMap::new();
    for unit in units {
        match unit {
            Unit::Item { item: Item::Func(f), own, .. } => {
                if !own || used.is_some_and(|u| !u.contains(&f.name)) {
                    continue;
                }
                for s in f.body.iter().flatten() {
                    stmt_calls(s, &mut |n| *sites.entry(n.to_string()).or_insert(0) += 1, &words);
                }
            }
            Unit::Include { operands, own: true, .. } => {
                for w in words(operands) {
                    *sites.entry(w).or_insert(0) += 2;
                }
            }
            Unit::Item { .. } | Unit::Include { .. } => {}
        }
    }
    sites
}

/// Leave out the functions that every caller got a copy of: those no
/// other kept code, nor imported assembly, names any more.
fn drop_inlined(
    units: &[Unit],
    code: Vec<String>,
    ranges: Vec<(String, usize, usize)>,
) -> Vec<String> {
    let words = |text: &str| -> Vec<String> {
        text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '$'))
            .filter(|w| !w.is_empty())
            .map(str::to_string)
            .collect()
    };
    let mut gone = vec![false; ranges.len()];
    loop {
        let mut named: HashSet<String> = HashSet::new();
        for unit in units {
            if let Unit::Include { operands, .. } = unit {
                named.extend(words(operands));
            }
        }
        let mut owner = vec![None; code.len()];
        for (k, (_, start, end)) in ranges.iter().enumerate() {
            for o in owner.iter_mut().take(*end).skip(*start) {
                *o = Some(k);
            }
        }
        for (i, line) in code.iter().enumerate() {
            if owner[i].is_some_and(|k| gone[k]) || line.trim_start().starts_with('#') {
                continue;
            }
            let own = owner[i].map(|k| ranges[k].0.as_str());
            named.extend(words(line).into_iter().filter(|w| Some(w.as_str()) != own));
        }
        let mut more = false;
        for (k, (name, ..)) in ranges.iter().enumerate() {
            if !gone[k] && name != "main" && !named.contains(name) {
                gone[k] = true;
                more = true;
            }
        }
        if !more {
            break;
        }
    }
    let mut drop = vec![false; code.len()];
    for (k, (_, start, end)) in ranges.iter().enumerate() {
        if gone[k] {
            drop[*start..*end].iter_mut().for_each(|d| *d = true);
        }
    }
    code.into_iter().zip(drop).filter_map(|(l, d)| (!d).then_some(l)).collect()
}

/// The data a program uses: the globals and strings its code or
/// imported assembly names, and those their initialisers name.
fn used_data(units: &[Unit], code: &[String], data: Vec<String>) -> Vec<String> {
    let words = |text: &str| -> Vec<String> {
        text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '$'))
            .filter(|w| !w.is_empty())
            .map(str::to_string)
            .collect()
    };
    let mut named: HashSet<String> = HashSet::new();
    for line in code {
        if !line.trim_start().starts_with('#') {
            named.extend(words(line));
        }
    }
    for unit in units {
        if let Unit::Include { operands, .. } = unit {
            named.extend(words(operands));
        }
    }
    let label = |entry: &String| entry.split(':').next().unwrap_or("").trim().to_string();
    let mut keep = vec![false; data.len()];
    loop {
        let mut grew = false;
        for (k, entry) in data.iter().enumerate() {
            if !keep[k] && named.contains(&label(entry)) {
                keep[k] = true;
                grew = true;
                let body = entry.split_once(':').map_or("", |(_, b)| b);
                named.extend(words(body));
            }
        }
        if !grew {
            break;
        }
    }
    data.into_iter().zip(keep).filter_map(|(entry, k)| k.then_some(entry)).collect()
}

/// Report each name `s` calls, or names in an `asm` block.
fn stmt_calls(s: &Stmt, f: &mut dyn FnMut(&str), words: &dyn Fn(&str) -> Vec<String>) {
    let e = |x: &Expr, f: &mut dyn FnMut(&str)| expr_calls(x, f);
    match s {
        Stmt::Block(body) => body.iter().for_each(|s| stmt_calls(s, f, words)),
        Stmt::Local { array, init, .. } => {
            array.iter().chain(init).for_each(|x| e(x, f));
        }
        Stmt::Expr(x) => e(x, f),
        Stmt::If(c, a, b) => {
            e(c, f);
            stmt_calls(a, f, words);
            if let Some(b) = b {
                stmt_calls(b, f, words);
            }
        }
        Stmt::While(c, body) => {
            e(c, f);
            stmt_calls(body, f, words);
        }
        Stmt::For(init, c, step, body) => {
            if let Some(init) = init {
                stmt_calls(init, f, words);
            }
            c.iter().chain(step).for_each(|x| e(x, f));
            stmt_calls(body, f, words);
        }
        Stmt::Return(x, _) => x.iter().for_each(|x| e(x, f)),
        Stmt::Which { value, arms, .. } => {
            e(value, f);
            for (_, body, _) in arms {
                body.iter().for_each(|s| stmt_calls(s, f, words));
            }
        }
        Stmt::Asm(text, _) => {
            for line in text.lines() {
                let code = line.split('#').next().unwrap_or("");
                words(code).iter().for_each(|w| f(w));
            }
        }
        Stmt::Break(_) | Stmt::Continue(_) => {}
    }
}

fn expr_calls(e: &Expr, f: &mut dyn FnMut(&str)) {
    match &e.kind {
        ExprKind::Call(name, args) => {
            f(name);
            args.iter().for_each(|a| expr_calls(a, f));
        }
        ExprKind::Unary(_, x)
        | ExprKind::Cast(_, x)
        | ExprKind::SizeofExpr(x)
        | ExprKind::Field(x, _, _) => expr_calls(x, f),
        ExprKind::List(items) => items.iter().for_each(|x| expr_calls(x, f)),
        ExprKind::Step { target, .. } => expr_calls(target, f),
        ExprKind::Binary(_, a, b) | ExprKind::Assign(_, a, b) | ExprKind::Index(a, b) => {
            expr_calls(a, f);
            expr_calls(b, f);
        }
        ExprKind::Cond(c, a, b) => {
            expr_calls(c, f);
            expr_calls(a, f);
            expr_calls(b, f);
        }
        ExprKind::Int(_)
        | ExprKind::Str(_)
        | ExprKind::Bool(_)
        | ExprKind::Name(_)
        | ExprKind::SizeofType(_) => {}
    }
}

impl Gen {
    /// A call with more than six arguments. Each is evaluated in order
    /// and stored at once, so the temporaries never run out: the
    /// seventh onward where the callee finds them, at 0(sp) and up
    /// (abi.md 2.2), the first six above them, loaded into a0..a5
    /// before the call.
    fn long_call(
        &mut self,
        name: &str,
        args: &[Expr],
        ret: &Type,
        params: &[Type],
        line: usize,
    ) -> R<Type> {
        let n = args.len() as i128;
        let extra = n - ARGS.len() as i128;
        let room = 3 * n;
        self.emit(format!("addi  sp, sp, {}", -room));
        let slot = |i: i128| if i < ARGS.len() as i128 { 3 * (extra + i) } else { 3 * (i - 6) };
        for (i, a) in args.iter().enumerate() {
            let t = self.expr(a)?;
            self.check_assign(&params[i], &t, a)?;
            let r = self.top();
            self.narrow(&params[i], &t, r);
            self.emit(format!("sw    {r}, {}(sp)", slot(i as i128)));
            self.pop();
        }
        for (i, reg) in ARGS.iter().enumerate() {
            self.emit(format!("lw    {reg}, {}(sp)", slot(i as i128)));
        }
        self.emit(format!("call  {name}, {}", ARGS.len()));
        self.emit(format!("addi  sp, sp, {room}"));
        let r = self.push(line)?;
        if *ret != Type::Void {
            self.emit(format!("mv    {r}, a0"));
        }
        Ok(ret.clone())
    }
}

/// Whether evaluating `e` may assign to the variable `name`.
fn writes(e: &Expr, name: &str) -> bool {
    let w = |x: &Expr| writes(x, name);
    match &e.kind {
        ExprKind::Assign(_, l, r) => {
            matches!(&l.kind, ExprKind::Name(n) if n == name) || w(l) || w(r)
        }
        ExprKind::Step { target, .. } => {
            matches!(&target.kind, ExprKind::Name(n) if n == name) || w(target)
        }
        ExprKind::Unary(_, x)
        | ExprKind::Cast(_, x)
        | ExprKind::SizeofExpr(x)
        | ExprKind::Field(x, _, _) => w(x),
        ExprKind::Binary(_, a, b) | ExprKind::Index(a, b) => w(a) || w(b),
        ExprKind::Cond(c, a, b) => w(c) || w(a) || w(b),
        ExprKind::Call(_, args) | ExprKind::List(args) => args.iter().any(w),
        _ => false,
    }
}

/// Whether evaluating `e` may change a variable: a call or an assignment.
fn has_effects(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Assign(..) | ExprKind::Step { .. } | ExprKind::Call(..) => true,
        ExprKind::Unary(_, x)
        | ExprKind::Cast(_, x)
        | ExprKind::SizeofExpr(x)
        | ExprKind::Field(x, _, _) => has_effects(x),
        ExprKind::Binary(_, a, b) | ExprKind::Index(a, b) => has_effects(a) || has_effects(b),
        ExprKind::Cond(c, a, b) => has_effects(c) || has_effects(a) || has_effects(b),
        ExprKind::List(items) => items.iter().any(has_effects),
        _ => false,
    }
}

/// Collect the names whose address `s` takes with `&`.
fn stmt_addressed(s: &Stmt, out: &mut HashSet<String>) {
    let mut each = |x: &Expr| expr_addressed(x, out);
    match s {
        Stmt::Block(body) => body.iter().for_each(|s| stmt_addressed(s, out)),
        Stmt::Local { array, init, .. } => array.iter().chain(init).for_each(&mut each),
        Stmt::Expr(x) => each(x),
        Stmt::If(c, a, b) => {
            each(c);
            stmt_addressed(a, out);
            if let Some(b) = b {
                stmt_addressed(b, out);
            }
        }
        Stmt::While(c, body) => {
            each(c);
            stmt_addressed(body, out);
        }
        Stmt::For(init, c, step, body) => {
            c.iter().chain(step).for_each(&mut each);
            if let Some(init) = init {
                stmt_addressed(init, out);
            }
            stmt_addressed(body, out);
        }
        Stmt::Return(x, _) => x.iter().for_each(each),
        Stmt::Which { value, arms, .. } => {
            each(value);
            for (_, body, _) in arms {
                body.iter().for_each(|s| stmt_addressed(s, out));
            }
        }
        Stmt::Asm(..) | Stmt::Break(_) | Stmt::Continue(_) => {}
    }
}

fn expr_addressed(e: &Expr, out: &mut HashSet<String>) {
    match &e.kind {
        ExprKind::Unary("&", x) => {
            if let ExprKind::Name(n) = &x.kind {
                out.insert(n.clone());
            }
            expr_addressed(x, out);
        }
        ExprKind::Unary(_, x)
        | ExprKind::Cast(_, x)
        | ExprKind::SizeofExpr(x)
        | ExprKind::Field(x, _, _)
        | ExprKind::Step { target: x, .. } => expr_addressed(x, out),
        ExprKind::Binary(_, a, b) | ExprKind::Assign(_, a, b) | ExprKind::Index(a, b) => {
            expr_addressed(a, out);
            expr_addressed(b, out);
        }
        ExprKind::Cond(c, a, b) => {
            expr_addressed(c, out);
            expr_addressed(a, out);
            expr_addressed(b, out);
        }
        ExprKind::Call(_, args) | ExprKind::List(args) => {
            args.iter().for_each(|a| expr_addressed(a, out))
        }
        ExprKind::Int(_)
        | ExprKind::Str(_)
        | ExprKind::Bool(_)
        | ExprKind::Name(_)
        | ExprKind::SizeofType(_) => {}
    }
}

fn pointee_size(t: &Type) -> i128 {
    match t {
        Type::Ptr(e) => e.size(),
        _ => 0,
    }
}

fn ty_max(t: &Type) -> i128 {
    match t {
        Type::Trit => 1,
        _ => TRYTE_MAX,
    }
}

fn stmt_line(s: &Stmt) -> Option<usize> {
    match s {
        Stmt::Local { line, .. } | Stmt::Break(line) | Stmt::Continue(line) => Some(*line),
        Stmt::Return(_, line) | Stmt::Asm(_, line) => Some(*line),
        Stmt::Expr(e) | Stmt::If(e, ..) | Stmt::While(e, _) => Some(e.line),
        Stmt::Which { value, .. } => Some(value.line),
        Stmt::For(..) | Stmt::Block(_) => None,
    }
}
