//! Code generation: the checked program into assembly (abi.md).
//!
//! The first version walks the tree. An expression's values live in
//! `t0`..`t6`, used as a stack; locals live in the frame, reached from
//! the frame pointer `s0`, so calls can push the temporaries they must
//! keep without moving the locals. Types are checked on the way.

use std::collections::HashMap;

use crate::ast::*;
use crate::lex::{TRYTE_MAX, WORD_MAX};
use crate::{Error, Output, Unit};

const TEMPS: [&str; 7] = ["t0", "t1", "t2", "t3", "t4", "t5", "t6"];
const ARGS: [&str; 6] = ["a0", "a1", "a2", "a3", "a4", "a5"];

/// Builtins of two arguments, each the instruction of the same name.
const BUILTIN2: [&str; 12] =
    ["min", "max", "divr", "remr", "tmul", "tsum", "tcons", "tany", "band", "bor", "bxor", "bsh"];
/// Builtins of one argument.
const BUILTIN1: [&str; 5] = ["abs", "tabs", "lzt", "tzt", "nzt"];

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
    depth: usize,
    loops: Vec<(String, String)>,
    ret: Type,
}

type R<T> = Result<T, Error>;

pub(crate) fn generate(units: &[Unit], main_file: &str) -> R<Output> {
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
        depth: 0,
        loops: Vec::new(),
        ret: Type::Void,
    };
    g.declare(units)?;
    g.code.push(format!("# Compiled by detrytus tc from {main_file}: docs/trice.md, docs/abi.md."));
    let mut compressing = false;
    for unit in units {
        match unit {
            Unit::Include { path, .. } => g.code.push(format!("        .include \"{path}\"")),
            Unit::Item { file, item: Item::Func(f) } if f.body.is_some() => {
                if !compressing {
                    g.code.push("        .compress".into());
                    compressing = true;
                }
                g.file = file.clone();
                g.function(f)?;
            }
            Unit::Item { .. } => {}
        }
    }
    if !compressing {
        g.code.push("        .compress".into());
    }
    let mut assembly = g.code.join("\n");
    assembly.push('\n');
    for line in &g.data {
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
    name == "sys" || BUILTIN1.contains(&name) || BUILTIN2.contains(&name)
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
                Unit::Item { file, item } => {
                    self.file = file.clone();
                    self.declare_item(item)?;
                }
            }
        }
        Ok(())
    }

    fn declare_item(&mut self, item: &Item) -> R<()> {
        match item {
            Item::Import(..) => Ok(()),
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
                self.data.push(data);
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
                if f.params.len() > ARGS.len() {
                    return self.err(f.line, "a function takes at most six arguments for now");
                }
                let mut params = Vec::new();
                for (t, n) in &f.params {
                    if *t == Type::Void {
                        return self.err(f.line, format!("parameter {n} cannot be void"));
                    }
                    if is_register(n) {
                        return self.err(f.line, format!("{n} is a register name"));
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
        if let Type::Array(elem, n) = ty {
            if init.is_some() {
                return self.err(line, "array initialisers are not supported yet");
            }
            return Ok(if elem.word_aligned() {
                format!("{name}: .words {n}")
            } else {
                format!("{name}: .zero {}", ty.size())
            });
        }
        let value = match init {
            None => "0".to_string(),
            Some(e) => match &e.kind {
                ExprKind::Str(s) if *ty == Type::Ptr(Box::new(Type::Tryte)) => self.string(s),
                ExprKind::Str(_) => return self.err(line, "a string initialises a tryte*"),
                ExprKind::Bool(v) if *ty == Type::Bool => v.to_string(),
                ExprKind::Unary("&", x) if ty.is_ptr() => match &x.kind {
                    ExprKind::Name(g) if self.globals.contains_key(g) => g.clone(),
                    _ => return self.err(line, "a global's initialiser must be a constant"),
                },
                _ if *ty == Type::Bool => {
                    return self.err(line, "a bool is true, false or unknown");
                }
                _ if ty.is_ptr() && !is_zero_literal(e) => {
                    return self.err(line, "a pointer starts as 0, a string or a global's address");
                }
                _ => self.const_eval(e)?.to_string(),
            },
        };
        let directive = if ty.size() == 1 { ".tryte" } else { ".word" };
        Ok(format!("{name}: {directive} {value}"))
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

    fn push(&mut self, line: usize) -> R<&'static str> {
        if self.depth == TEMPS.len() {
            return self.err(line, "expression too complex: more than 7 values at once");
        }
        self.depth += 1;
        Ok(TEMPS[self.depth - 1])
    }

    fn top(&self) -> &'static str {
        TEMPS[self.depth - 1]
    }

    fn second(&self) -> &'static str {
        TEMPS[self.depth - 2]
    }

    fn pop(&mut self) {
        self.depth -= 1;
    }

    fn load(&mut self, ty: &Type, dst: &str, off: i128, base: &str) {
        let op = if ty.size() == 1 { "lt" } else { "lw" };
        self.emit(format!("{op}    {dst}, {off}({base})"));
    }

    fn store(&mut self, ty: &Type, src: &str, off: i128, base: &str) {
        let op = if ty.size() == 1 { "st" } else { "sw" };
        self.emit(format!("{op}    {src}, {off}({base})"));
    }

    // --- functions -------------------------------------------------------

    fn function(&mut self, f: &Func) -> R<()> {
        self.fname = f.name.clone();
        self.labels = 0;
        self.slots = 0;
        self.depth = 0;
        self.loops.clear();
        self.ret = f.ret.clone();
        self.scopes = vec![HashMap::new()];
        let outer = std::mem::take(&mut self.code);
        for (i, (ty, name)) in f.params.iter().enumerate() {
            if self.scopes[0].contains_key(name) {
                return self.err(f.line, format!("two parameters are called {name}"));
            }
            let off = self.allocate(ty);
            self.store(ty, ARGS[i], off, "s0");
            self.scopes[0].insert(name.clone(), Local { ty: ty.clone(), off });
        }
        for s in f.body.as_ref().unwrap() {
            self.stmt(s)?;
        }
        let ret_label = format!("{}$ret", f.name);
        if self.code.last().is_some_and(|l| l.trim() == format!("j     {ret_label}")) {
            self.code.pop();
        }
        let body = std::mem::replace(&mut self.code, outer);
        let frame = 3 * (2 + self.slots);
        self.code.push(String::new());
        self.code.push(format!("# {} {}(...), {}:{}", f.ret.name(), f.name, self.file, f.line));
        self.label(&f.name);
        self.emit(format!("addi  sp, sp, {}", -frame));
        self.emit(format!("sw    ra, {}(sp)", frame - 3));
        self.emit(format!("sw    s0, {}(sp)", frame - 6));
        self.emit(format!("addi  s0, sp, {frame}"));
        self.code.extend(body);
        self.label(&ret_label);
        self.emit("mv    sp, s0");
        self.emit("lw    ra, -3(sp)");
        self.emit("lw    s0, -6(sp)");
        self.emit("ret");
        Ok(())
    }

    /// Frame room for a local: its offset from `s0` (abi.md 2.4: `ra`
    /// at -3, the caller's `s0` at -6, locals below).
    fn allocate(&mut self, ty: &Type) -> i128 {
        let words = (ty.size() + 2) / 3;
        self.slots += words;
        let lowest_centre = -3 * (2 + self.slots);
        if ty.word_aligned() || !matches!(ty, Type::Array(..)) {
            lowest_centre
        } else {
            lowest_centre - 1
        }
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
        debug_assert_eq!(self.depth, 0);
        if let Some(line) = stmt_line(s) {
            self.code.push(format!("# {}:{line}", self.file));
        }
        match s {
            Stmt::Block(body) => {
                self.scopes.push(HashMap::new());
                for s in body {
                    self.stmt(s)?;
                }
                self.scopes.pop();
            }
            Stmt::Local { ty, name, array, init, line } => {
                if is_register(name) {
                    return self.err(*line, format!("{name} is a register name"));
                }
                if self.scopes.last().unwrap().contains_key(name) {
                    return self.err(*line, format!("{name} is already declared here"));
                }
                let ty = self.decl_type(ty, array, *line)?;
                if let Some(init) = init {
                    if matches!(ty, Type::Array(..)) {
                        return self.err(*line, "array initialisers are not supported yet");
                    }
                    let t = self.expr(init)?;
                    self.check_assign(&ty, &t, init)?;
                    let off = self.allocate(&ty);
                    let r = self.top();
                    self.store(&ty, r, off, "s0");
                    self.pop();
                    self.scopes.last_mut().unwrap().insert(name.clone(), Local { ty, off });
                } else {
                    let off = self.allocate(&ty);
                    self.scopes.last_mut().unwrap().insert(name.clone(), Local { ty, off });
                }
            }
            Stmt::Expr(e) => {
                self.expr(e)?;
                self.pop();
            }
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
                self.scopes.push(HashMap::new());
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
                    self.expr(step)?;
                    self.pop();
                }
                self.emit(format!("j     {top}"));
                self.label(&end);
                self.scopes.pop();
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
                    (Some(e), ret) => {
                        let t = self.expr(e)?;
                        self.check_assign(&ret, &t, e)?;
                        self.emit("mv    a0, t0");
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
                for line in text.lines().filter(|l| !l.trim().is_empty()) {
                    self.code.push(line.to_string());
                }
            }
        }
        Ok(())
    }

    /// A statement in a scope of its own, as the body of an if or a loop.
    fn scoped(&mut self, s: &Stmt) -> R<()> {
        self.scopes.push(HashMap::new());
        let r = self.stmt(s);
        self.scopes.pop();
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
            self.scopes.push(HashMap::new());
            for s in body {
                self.stmt(s)?;
            }
            self.scopes.pop();
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
                let (x, y) = self.compare_operands(op, a, b, e.line)?;
                let br = Self::comparison(Self::inverse(op)).unwrap();
                self.emit(format!("{br}   {x}, {y}, {to}"));
                self.pop_operands(y);
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
                self.bool_value(x)?;
                let r = self.top();
                self.emit(format!("bgez  {r}, {to}"));
                self.pop();
            }
            _ => {
                self.bool_value(e)?;
                let r = self.top();
                self.emit(format!("blez  {r}, {to}"));
                self.pop();
            }
        }
        Ok(())
    }

    /// Go to `to` when `e` is true.
    fn branch_true(&mut self, e: &Expr, to: &str) -> R<()> {
        match &e.kind {
            ExprKind::Binary(op, a, b) if Self::comparison(op).is_some() => {
                let (x, y) = self.compare_operands(op, a, b, e.line)?;
                let br = Self::comparison(op).unwrap();
                self.emit(format!("{br}   {x}, {y}, {to}"));
                self.pop_operands(y);
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
                self.bool_value(x)?;
                let r = self.top();
                self.emit(format!("bltz  {r}, {to}"));
                self.pop();
            }
            _ => {
                self.bool_value(e)?;
                let r = self.top();
                self.emit(format!("bgtz  {r}, {to}"));
                self.pop();
            }
        }
        Ok(())
    }

    fn bool_value(&mut self, e: &Expr) -> R<()> {
        let t = self.expr(e)?;
        if t != Type::Bool {
            let hint = if t.numeric() { ": compare it, as in x != 0" } else { "" };
            return self.err(e.line, format!("a condition must be a bool, not {}{hint}", t.a()));
        }
        Ok(())
    }

    /// Both sides of a comparison, checked: their registers, the right
    /// one `zero` (and not pushed) when it is the constant 0.
    fn compare_operands(
        &mut self,
        op: &str,
        a: &Expr,
        b: &Expr,
        line: usize,
    ) -> R<(&'static str, &'static str)> {
        if literal_minus_one(a) || literal_minus_one(b) {
            self.warnings.push(format!(
                "{}:{line}: warning: -1 is an ordinary value: test the sign or compare with a named constant",
                self.file
            ));
        }
        let ta = self.expr(a)?;
        let zero = ta.numeric() && self.try_const(b) == Some(0);
        let tb = if zero { Type::Int } else { self.expr(b)? };
        let ok = (ta.numeric() && tb.numeric())
            || (ta == Type::Bool && tb == Type::Bool && (op == "==" || op == "!="))
            || (ta.is_ptr() && (ta == tb || is_zero_literal(b)))
            || (tb.is_ptr() && is_zero_literal(a));
        if !ok {
            return self.err(line, format!("cannot compare {} with {}", ta.a(), tb.a()));
        }
        Ok(if zero { (self.top(), "zero") } else { (self.second(), self.top()) })
    }

    /// Drop a comparison's operands.
    fn pop_operands(&mut self, y: &str) {
        self.pop();
        if y != "zero" {
            self.pop();
        }
    }

    // --- expressions -------------------------------------------------------

    fn check_assign(&self, dst: &Type, src: &Type, e: &Expr) -> R<()> {
        let ok = match (dst, src) {
            (d, s) if d.numeric() && s.numeric() => true,
            (Type::Bool, Type::Bool) => true,
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
                let base = self.depth;
                let ta = self.expr(a)?;
                self.emit(format!("j     {end}"));
                self.depth = base;
                self.label(&other);
                let tb = self.expr(b)?;
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
            ExprKind::Index(..) => {
                let t = self.addr(e)?;
                if !matches!(t, Type::Array(..)) {
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
        let (code, depth, strings, data) =
            (self.code.len(), self.depth, self.strings, self.data.len());
        let t = match &x.kind {
            ExprKind::Name(_) | ExprKind::Index(..) | ExprKind::Unary("*", _) => self.addr(x),
            _ => self.expr(x),
        };
        self.code.truncate(code);
        self.data.truncate(data);
        self.depth = depth;
        self.strings = strings;
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
            Place::Local(Local { ty, off }) => {
                if matches!(ty, Type::Array(..)) {
                    self.emit(format!("addi  {r}, s0, {off}"));
                } else {
                    self.load(&ty, r, off, "s0");
                }
                Ok(ty.decay())
            }
            Place::Global(ty) => {
                self.emit(format!("la    {r}, {n}"));
                if !matches!(ty, Type::Array(..)) {
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
                Some(Place::Local(Local { ty, off })) => {
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
                let ta = self.expr(a)?;
                let Type::Ptr(elem) = ta else {
                    return self.err(line, format!("cannot index {}", ta.a()));
                };
                if *elem == Type::Void {
                    return self.err(line, "cannot index a void*");
                }
                let size = elem.size();
                if let Some(k) = self.try_const(i).filter(|k| (k * size).abs() <= IMM_MAX) {
                    let r = self.top();
                    if k != 0 {
                        self.emit(format!("addi  {r}, {r}, {}", k * size));
                    }
                } else {
                    let ti = self.expr(i)?;
                    if !ti.numeric() {
                        return self.err(line, format!("an index is a number, not {}", ti.a()));
                    }
                    self.scale(size, line)?;
                    let (x, y) = (self.second(), self.top());
                    self.emit(format!("add   {x}, {x}, {y}"));
                    self.pop();
                }
                Ok(*elem)
            }
            _ => self.err(line, "this is not something with an address"),
        }
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
                if !matches!(t, Type::Array(..)) {
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
            let (x, y) = self.compare_operands(op, a, b, line)?;
            self.materialise(op, x, y);
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
                let ta = self.expr(a)?;
                if let Some(t) = self.immediate(op, &ta, b, line)? {
                    return Ok(t);
                }
                let tb = self.expr(b)?;
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
                if size == 3 {
                    self.emit(format!("shi   {x}, {x}, 1"));
                } else if size != 1 {
                    return bad(self);
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
                    1 => {}
                    3 => self.emit(format!("shi   {x}, {x}, -1")),
                    _ => return bad(self),
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
    fn materialise(&mut self, op: &str, x: &str, y: &str) {
        match op {
            // c = cmp(x, y); x > y is sign(2c - 1), x >= y is sign(2c + 1).
            ">" | ">=" | "<" | "<=" => {
                let (l, r) = if op.starts_with('>') { (x, y) } else { (y, x) };
                self.emit(format!("cmp   {x}, {l}, {r}"));
                self.emit(format!("add   {x}, {x}, {x}"));
                let k = if op.ends_with('=') { -1 } else { 1 };
                self.emit(format!("cmpi  {x}, {x}, {k}"));
            }
            // |c| is 1 when they differ: != is sign(2|c| - 1).
            _ => {
                self.emit(format!("cmp   {x}, {x}, {y}"));
                self.emit(format!("tabs  {x}, {x}"));
                self.emit(format!("add   {x}, {x}, {x}"));
                self.emit(format!("cmpi  {x}, {x}, 1"));
                if op == "==" {
                    self.emit(format!("neg   {x}, {x}"));
                }
            }
        }
        if y != "zero" {
            self.pop();
        }
    }

    fn assign(&mut self, op: &str, l: &Expr, r: &Expr, line: usize) -> R<Type> {
        if op == "=" {
            if let ExprKind::Name(n) = &l.kind
                && let Some(Place::Local(local)) = self.lookup(n)
                && !matches!(local.ty, Type::Array(..))
            {
                let t = self.expr(r)?;
                self.check_assign(&local.ty, &t, r)?;
                let v = self.top();
                self.store(&local.ty, v, local.off, "s0");
                return Ok(local.ty);
            }
            let tl = self.addr(l)?;
            if matches!(tl, Type::Array(..)) {
                return self.err(line, "cannot assign to a whole array");
            }
            let t = self.expr(r)?;
            self.check_assign(&tl, &t, r)?;
            let (a, v) = (self.second(), self.top());
            self.store(&tl, v, 0, a);
            self.emit(format!("mv    {a}, {v}"));
            self.pop();
            return Ok(tl);
        }
        let base = &op[..op.len() - 1];
        let tl = self.addr(l)?;
        if matches!(tl, Type::Array(..)) {
            return self.err(line, "cannot assign to a whole array");
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

    fn step(&mut self, op: &str, post: bool, target: &Expr, line: usize) -> R<Type> {
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
            Type::Array(..) => false,
        };
        if !ok {
            let hint = if *to == Type::Bool { ": write a comparison, as in x != 0" } else { "" };
            return self.err(line, format!("cannot cast {} to {}{hint}", from.a(), to.a()));
        }
        Ok(to.clone())
    }

    fn call(&mut self, name: &str, args: &[Expr], line: usize) -> R<Type> {
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
        let base = self.depth;
        for (i, a) in args.iter().enumerate() {
            let t = self.expr(a)?;
            match &params {
                Some(p) => self.check_assign(&p[i], &t, a)?,
                None if t == Type::Void => return self.err(a.line, "sys takes values"),
                None => {}
            }
        }
        // Keep the temporaries an enclosing expression still needs.
        if base > 0 {
            self.emit(format!("addi  sp, sp, {}", -3 * base as i128));
            for (i, t) in TEMPS.iter().enumerate().take(base) {
                self.emit(format!("sw    {t}, {}(sp)", 3 * i));
            }
        }
        for i in 0..args.len() {
            self.emit(format!("mv    {}, {}", ARGS[i], TEMPS[base + i]));
        }
        if name == "sys" {
            self.emit("sys");
        } else {
            self.emit(format!("call  {name}"));
        }
        if base > 0 {
            for (i, t) in TEMPS.iter().enumerate().take(base) {
                self.emit(format!("lw    {t}, {}(sp)", 3 * i));
            }
            self.emit(format!("addi  sp, sp, {}", 3 * base));
        }
        self.depth = base;
        let r = self.push(line)?;
        if ret != Type::Void {
            self.emit(format!("mv    {r}, a0"));
        }
        Ok(ret)
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
