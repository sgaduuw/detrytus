//! LLVM IR, the textual form rustc writes with `--emit=llvm-ir`: the
//! part of it a translator needs, read line by line.
//!
//! Each global, declaration and instruction is one line (a `switch`
//! continues to its closing `]`). What a line says beyond what lowering
//! uses (attributes, alignment, metadata) is skipped. A line inside a
//! function that cannot be read becomes [`Op::Other`], an error only if
//! the function turns out to be used: core has far more than any one
//! program needs.

use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub enum Ty {
    Void,
    Int(u32),
    Ptr,
    /// `half`, `float`, `double` and the rest: read, never lowered.
    Float(String),
    Array(u64, Box<Ty>),
    /// Fields, and whether packed (`<{ ... }>`).
    Struct(Vec<Ty>, bool),
    Vector(u64, Box<Ty>),
    Named(String),
    Label,
    Metadata,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Const {
    Int(i128),
    /// A floating-point constant, as the bits of the double that LLVM
    /// writes for it (also for a `float`, whose value it holds exactly).
    Float(u64),
    Null,
    /// `undef` and `poison`: any value will do.
    Undef,
    Zero,
    Global(String),
    /// `c"..."`
    Bytes(Vec<u8>),
    /// An array or struct literal: its elements with their types.
    Agg(Vec<(Ty, Const)>),
    Gep {
        src: Ty,
        base: Box<(Ty, Const)>,
        idx: Vec<(Ty, Const)>,
    },
    Cast {
        op: String,
        from: Box<(Ty, Const)>,
        to: Ty,
    },
    Bin {
        op: String,
        a: Box<(Ty, Const)>,
        b: Box<(Ty, Const)>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Val {
    Local(String),
    Const(Const),
}

#[derive(Clone, Debug)]
pub enum Op {
    Bin {
        op: String,
        flags: Vec<String>,
        ty: Ty,
        a: Val,
        b: Val,
    },
    Icmp {
        pred: String,
        ty: Ty,
        a: Val,
        b: Val,
    },
    Cast {
        op: String,
        flags: Vec<String>,
        from: Ty,
        v: Val,
        to: Ty,
    },
    Select {
        c: Val,
        ty: Ty,
        a: Val,
        b: Val,
    },
    Phi {
        ty: Ty,
        incoming: Vec<(Val, String)>,
    },
    Load {
        ty: Ty,
        p: Val,
    },
    Store {
        ty: Ty,
        v: Val,
        p: Val,
    },
    Alloca {
        ty: Ty,
        n: Option<(Ty, Val)>,
        /// The alignment asked for, 1 when none is.
        align: u64,
    },
    Gep {
        flags: Vec<String>,
        src: Ty,
        base: Val,
        idx: Vec<(Ty, Val)>,
    },
    Call {
        ret: Ty,
        callee: Val,
        args: Vec<(Ty, Val)>,
    },
    Br {
        c: Val,
        t: String,
        f: String,
    },
    Jump(String),
    Switch {
        ty: Ty,
        v: Val,
        default: String,
        cases: Vec<(i128, String)>,
    },
    Ret(Option<(Ty, Val)>),
    Unreachable,
    Extract {
        ty: Ty,
        agg: Val,
        idx: Vec<u64>,
    },
    Insert {
        ty: Ty,
        agg: Val,
        vty: Ty,
        v: Val,
        idx: Vec<u64>,
    },
    Freeze {
        ty: Ty,
        v: Val,
    },
    /// Take the lock that makes atomic operations atomic (docs/rust.md,
    /// Atomics); the parser brackets each one's plain operations with it.
    Lock,
    /// Give the lock back.
    Unlock,
    /// An atomic operation as read, before [`expand_atomic`] turns it
    /// into plain ones: `load atomic` (`op` "load"), `store atomic`
    /// ("store", `a` the value), `atomicrmw OP` (OP, `a` the operand),
    /// `cmpxchg` ("cmpxchg", `a` expected, `b` new) and `fence`.
    Atomic {
        op: String,
        ty: Ty,
        p: Val,
        a: Val,
        b: Val,
    },
    /// A floating-point operation as read, before [`expand_float`] turns
    /// it into calls and integer operations: `fadd` and the rest with
    /// `a` and `b`, `fneg` with `a`, `fcmp PRED` (`op` holds both
    /// words), and the casts from `ty` to `to`.
    Float {
        op: String,
        ty: Ty,
        a: Val,
        b: Val,
        to: Ty,
    },
    /// An operation on the binary floating point coprocessor
    /// (docs/platform.md, page −15): `args` into A, B and C, `code` into
    /// OP, the result from R. A comparison's result is FPU_CMP's plus
    /// one: 0 less, 1 equal, 2 greater, 3 unordered.
    Fpu {
        code: i64,
        args: Vec<(Ty, Val)>,
        ret: Ty,
    },
    /// A line that could not be read, or an instruction not handled:
    /// what to say if the function is used.
    Other(String),
}

#[derive(Clone, Debug)]
pub struct Instr {
    pub res: Option<String>,
    pub op: Op,
    pub line: usize,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub label: String,
    pub instrs: Vec<Instr>,
}

#[derive(Clone, Debug)]
pub struct Function {
    pub name: String,
    pub ret: Ty,
    pub params: Vec<(Ty, String)>,
    pub varargs: bool,
    pub blocks: Vec<Block>,
    pub file: String,
    pub line: usize,
}

#[derive(Clone, Debug)]
pub struct Global {
    pub name: String,
    pub ty: Ty,
    /// None for an external one; an error for an initialiser that could
    /// not be read.
    pub init: Option<Result<Const, String>>,
    /// The alignment asked for, 1 when none is.
    pub align: u64,
    pub file: String,
    pub line: usize,
}

#[derive(Default, Debug)]
pub struct Module {
    pub types: HashMap<String, Ty>,
    pub globals: HashMap<String, Global>,
    pub funcs: HashMap<String, Function>,
    /// Functions declared but not defined.
    pub decls: HashSet<String>,
    /// Aliases: another name for a function or global.
    pub aliases: HashMap<String, String>,
    /// The text of each definition, keyed by kind ('f' function, 'g'
    /// global, 't' type, 'd' declaration) and name, for [`prune`].
    pub source: HashMap<(char, String), String>,
}

// --- tokens ---------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Local(String),
    Global(String),
    Word(String),
    Int(i128),
    Bytes(Vec<u8>),
    Str(String),
    Punct(char),
    Meta(String),
    Attr(String),
}

fn hexval(c: u8) -> Option<u8> {
    (c as char).to_digit(16).map(|d| d as u8)
}

/// The bytes of a quoted string's inside, with `\XX` and `\\` escapes.
fn unescape(s: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < s.len() {
        if s[i] == b'\\' && i + 2 < s.len() + 1 {
            if s.get(i + 1) == Some(&b'\\') {
                out.push(b'\\');
                i += 2;
                continue;
            }
            if let (Some(h), Some(l)) =
                (s.get(i + 1).and_then(|&c| hexval(c)), s.get(i + 2).and_then(|&c| hexval(c)))
            {
                out.push(h * 16 + l);
                i += 3;
                continue;
            }
        }
        out.push(s[i]);
        i += 1;
    }
    out
}

fn word_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, b'_' | b'.' | b'$' | b'-')
}

/// A name after `%`, `@` or `!`: bare, a number, or quoted.
fn name_at(s: &[u8], i: &mut usize) -> Result<String, String> {
    if s.get(*i) == Some(&b'"') {
        let start = *i + 1;
        let mut j = start;
        while j < s.len() && s[j] != b'"' {
            j += 1;
        }
        if j >= s.len() {
            return Err("unterminated name".into());
        }
        *i = j + 1;
        return Ok(String::from_utf8_lossy(&unescape(&s[start..j])).into_owned());
    }
    let start = *i;
    while *i < s.len() && word_char(s[*i]) {
        *i += 1;
    }
    if *i == start {
        return Err("a name is missing".into());
    }
    Ok(String::from_utf8_lossy(&s[start..*i]).into_owned())
}

fn tokens(line: &str) -> Result<Vec<Tok>, String> {
    let s = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < s.len() {
        let c = s[i];
        match c {
            b' ' | b'\t' | b'\r' => i += 1,
            b';' => break,
            b'%' | b'@' | b'!' => {
                i += 1;
                if c == b'!' && matches!(s.get(i), Some(b'{') | Some(b'"')) {
                    // A metadata node or string: the rest is metadata.
                    out.push(Tok::Meta(String::new()));
                    break;
                }
                let n = name_at(s, &mut i)?;
                out.push(match c {
                    b'%' => Tok::Local(n),
                    b'@' => Tok::Global(n),
                    _ => Tok::Meta(n),
                });
            }
            b'#' => {
                i += 1;
                let n = name_at(s, &mut i)?;
                out.push(Tok::Attr(n));
            }
            b'c' if s.get(i + 1) == Some(&b'"') => {
                let start = i + 2;
                let mut j = start;
                while j < s.len() && s[j] != b'"' {
                    j += 1;
                }
                out.push(Tok::Bytes(unescape(&s[start..j])));
                i = j + 1;
            }
            b'"' => {
                let start = i + 1;
                let mut j = start;
                while j < s.len() && s[j] != b'"' {
                    j += 1;
                }
                out.push(Tok::Str(String::from_utf8_lossy(&s[start..j]).into_owned()));
                i = j + 1;
            }
            b'-' | b'0'..=b'9'
                if (c != b'-' || s.get(i + 1).is_some_and(|d| d.is_ascii_digit()))
                    && !(c == b'0' && matches!(s.get(i + 1), Some(b'x'))) =>
            {
                let start = i;
                i += 1;
                while i < s.len() && s[i].is_ascii_digit() {
                    i += 1;
                }
                let text = &line[start..i];
                if i < s.len() && (s[i] == b'.' || s[i] == b'e') {
                    // A decimal float, `1.500000e+00`: a word.
                    while i < s.len() && (word_char(s[i]) || (s[i] == b'+' && s[i - 1] == b'e')) {
                        i += 1;
                    }
                    out.push(Tok::Word(line[start..i].to_string()));
                } else {
                    out.push(Tok::Int(text.parse().map_err(|_| format!("bad number {text}"))?));
                }
            }
            b'(' | b')' | b'[' | b']' | b'{' | b'}' | b'<' | b'>' | b',' | b'=' | b'*' | b':'
            | b'|' => {
                out.push(Tok::Punct(c as char));
                i += 1;
            }
            // `+inf`, `+qnan`: floating-point constants as newer LLVM
            // writes them.
            b'+' if s.get(i + 1).is_some_and(|d| d.is_ascii_alphabetic()) => {
                let start = i;
                i += 1;
                while i < s.len() && word_char(s[i]) {
                    i += 1;
                }
                out.push(Tok::Word(line[start..i].to_string()));
            }
            _ if word_char(c) => {
                let start = i;
                while i < s.len() && word_char(s[i]) {
                    i += 1;
                }
                out.push(Tok::Word(line[start..i].to_string()));
            }
            _ => return Err(format!("unexpected '{}'", c as char)),
        }
    }
    Ok(out)
}

// --- parsing a line -------------------------------------------------------

struct P {
    t: Vec<Tok>,
    i: usize,
}

type R<T> = Result<T, String>;

/// Whether a word is a floating-point constant.
fn is_float_word(w: &str) -> bool {
    w.starts_with("0x")
        || w.starts_with("f0x")
        || w.starts_with(|c: char| c.is_ascii_digit())
        || matches!(w.trim_start_matches(['+', '-']), "inf" | "qnan")
        || (w.starts_with('-') && w[1..].starts_with(|c: char| c.is_ascii_digit()))
}

/// A floating-point constant word: decimal (`1.500000e+00`) or the
/// bits of a double in hex (`0x3FF8000000000000`), or of a float
/// (`f0x3E99999B` for a float, `f0x...` with 16 digits for a double, as
/// newer LLVM writes them).
fn float_const(w: &str) -> R<Const> {
    if let Some(h) = w.strip_prefix("f0x") {
        if h.len() == 8
            && let Ok(b) = u32::from_str_radix(h, 16)
        {
            return Ok(Const::Float((f32::from_bits(b) as f64).to_bits()));
        }
        if h.len() == 16
            && let Ok(b) = u64::from_str_radix(h, 16)
        {
            return Ok(Const::Float(b));
        }
    }
    let negative = w.starts_with('-');
    match w.trim_start_matches(['+', '-']) {
        "inf" => {
            return Ok(Const::Float(
                if negative { f64::NEG_INFINITY } else { f64::INFINITY }.to_bits(),
            ));
        }
        "qnan" => return Ok(Const::Float(0x7FF8_0000_0000_0000 | (negative as u64) << 63)),
        _ => {}
    }
    if let Some(h) = w.strip_prefix("0x") {
        if h.len() == 16 && h.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Ok(Const::Float(u64::from_str_radix(h, 16).map_err(|e| e.to_string())?));
        }
        return Err(format!("the floating-point constant {w} is not supported"));
    }
    w.parse::<f64>()
        .map(|v| Const::Float(v.to_bits()))
        .map_err(|_| format!("constant {w} is not supported"))
}

/// A type as LLVM writes the simple ones.
fn shown(t: &Ty) -> String {
    match t {
        Ty::Float(f) => f.clone(),
        Ty::Int(n) => format!("i{n}"),
        Ty::Ptr => "ptr".into(),
        t => format!("{t:?}"),
    }
}

/// The integer type as wide as a float type, and the suffix of the
/// conversion functions for it (`__fixdfti`), if it is one supported.
fn float_kind(t: &Ty) -> Option<(u32, &'static str)> {
    match t {
        Ty::Float(f) if f == "float" => Some((32, "sf")),
        Ty::Float(f) if f == "double" => Some((64, "df")),
        _ => None,
    }
}

/// Builds the instructions that replace one floating-point operation.
struct FloatOps {
    res: String,
    line: usize,
    n: usize,
    out: Vec<Instr>,
}

impl FloatOps {
    fn fresh(&mut self) -> String {
        self.n += 1;
        format!("{}$f{}", self.res, self.n)
    }

    /// `op` into a new local, or into the result when `last`.
    fn put(&mut self, op: Op, last: bool) -> Val {
        let r = if last { self.res.clone() } else { self.fresh() };
        self.out.push(Instr { res: Some(r.clone()), op, line: self.line });
        Val::Local(r)
    }

    fn call(&mut self, f: &str, ret: Ty, args: Vec<(Ty, Val)>, last: bool) -> Val {
        let callee = Val::Const(Const::Global(f.to_string()));
        self.put(Op::Call { ret, callee, args }, last)
    }

    fn cast(&mut self, op: &str, from: &Ty, v: Val, to: &Ty, last: bool) -> Val {
        let op = Op::Cast { op: op.into(), flags: vec![], from: from.clone(), v, to: to.clone() };
        self.put(op, last)
    }

    fn bin(&mut self, op: &str, ty: &Ty, a: Val, b: Val, last: bool) -> Val {
        self.put(Op::Bin { op: op.into(), flags: vec![], ty: ty.clone(), a, b }, last)
    }

    /// Operation `n` of the coprocessor on `args` of float type `ty`,
    /// with the sign that picks its precision.
    fn fpu(&mut self, n: i64, ty: &Ty, args: Vec<(Ty, Val)>, ret: Ty, last: bool) -> Val {
        let code = if float_kind(ty).is_some_and(|(bits, _)| bits == 32) { -n } else { n };
        self.put(Op::Fpu { code, args, ret }, last)
    }

    fn icmp(&mut self, pred: &str, a: Val, k: i128, last: bool) -> Val {
        let op = Op::Icmp { pred: pred.into(), ty: Ty::Int(32), a, b: Val::Const(Const::Int(k)) };
        self.put(op, last)
    }
}

/// A floating-point operation as coprocessor operations, calls to the
/// functions of lower/float.ll and integer operations on the bits
/// (docs/rust.md, Floating point). `float` and `double` are held as the
/// integers of their bits. Anything else is itself.
fn expand_float(i: Instr) -> Vec<Instr> {
    let line = i.line;
    let res = i.res.clone().unwrap_or_else(|| format!("fp${line}"));
    let mut f = FloatOps { res, line, n: 0, out: Vec::new() };
    let done = match &i.op {
        Op::Float { op, ty, a, b, to } => float_op(&mut f, op, ty, a, b, to),
        Op::Call { ret, callee: Val::Const(Const::Global(name)), args }
            if name.starts_with("llvm.") =>
        {
            float_intrinsic(&mut f, name, ret, args)
        }
        _ => return vec![i],
    };
    match done {
        Ok(true) => f.out.into_iter().flat_map(expand_float).collect(),
        Ok(false) => vec![i],
        // Op::Other names what is not supported.
        Err(e) => {
            let what = e.trim_end_matches(" is not supported").to_string();
            vec![Instr { res: i.res, op: Op::Other(what), line }]
        }
    }
}

fn float_op(f: &mut FloatOps, op: &str, ty: &Ty, a: &Val, b: &Val, to: &Ty) -> R<bool> {
    let i32t = Ty::Int(32);
    let fty = ty.clone();
    match op {
        "fadd" | "fsub" | "fmul" | "fdiv" | "frem" | "fneg" => {
            let (bits, _) =
                float_kind(ty).ok_or(format!("{op} on {} is not supported", shown(ty)))?;
            match op {
                "fneg" => {
                    let it = Ty::Int(bits);
                    let x = f.cast("bitcast", ty, a.clone(), &it, false);
                    let sign = Val::Const(Const::Int(1i128 << (bits - 1)));
                    let n = f.bin("xor", &it, x, sign, false);
                    f.cast("bitcast", &it, n, ty, true);
                }
                "frem" => {
                    let name = if bits == 32 { "fmodf" } else { "fmod" };
                    f.call(
                        name,
                        fty.clone(),
                        vec![(fty.clone(), a.clone()), (fty, b.clone())],
                        true,
                    );
                }
                _ => {
                    let n = match op {
                        "fadd" => FPU_ADD,
                        "fsub" => FPU_SUB,
                        "fmul" => FPU_MUL,
                        _ => FPU_DIV,
                    };
                    let args = vec![(fty.clone(), a.clone()), (fty.clone(), b.clone())];
                    f.fpu(n, &fty, args, fty.clone(), true);
                }
            }
        }
        _ if op.starts_with("fcmp ") => {
            float_kind(ty).ok_or(format!("{op} on {} is not supported", shown(ty)))?;
            let pred = &op[5..];
            if pred == "false" || pred == "true" {
                let p = if pred == "true" { "eq" } else { "ne" };
                f.icmp(p, Val::Const(Const::Int(0)), 0, true);
                return Ok(true);
            }
            // The coprocessor's answer plus one: 0 less, 1 equal, 2
            // greater, 3 unordered; each predicate is a set of those.
            let args = vec![(fty.clone(), a.clone()), (fty.clone(), b.clone())];
            let c = f.fpu(FPU_CMP, &fty, args, i32t.clone(), false);
            let k = |v: i128| Val::Const(Const::Int(v));
            match pred {
                "oeq" => f.icmp("eq", c, 1, true),
                "olt" => f.icmp("eq", c, 0, true),
                "ogt" => f.icmp("eq", c, 2, true),
                "uno" => f.icmp("eq", c, 3, true),
                "une" => f.icmp("ne", c, 1, true),
                "uge" => f.icmp("ne", c, 0, true),
                "ule" => f.icmp("ne", c, 2, true),
                "ord" => f.icmp("ult", c, 3, true),
                "ole" => f.icmp("ult", c, 2, true),
                "ugt" => f.icmp("ugt", c, 1, true),
                "oge" | "ult" => {
                    // 1 or 2: c − 1 below 2.
                    let d = f.bin("add", &i32t, c, k(-1), false);
                    f.icmp(if pred == "oge" { "ult" } else { "uge" }, d, 2, true)
                }
                "one" | "ueq" => {
                    // 0 or 2 have the low bit clear; 1 or 3 set.
                    let lowbit = f.bin("and", &i32t, c, k(1), false);
                    f.icmp(if pred == "one" { "eq" } else { "ne" }, lowbit, 0, true)
                }
                p => return Err(format!("fcmp {p} is not supported")),
            };
        }
        "fptosi" | "fptoui" => {
            let (_, sfx) =
                float_kind(ty).ok_or(format!("{op} from {} is not supported", shown(ty)))?;
            let Ty::Int(n) = to else {
                return Err(format!("{op} to {} is not supported", shown(to)));
            };
            let (w, size) = int_width(*n)?;
            let u = if op == "fptoui" { "uns" } else { "" };
            let wt = Ty::Int(w);
            let v = match (w, u.is_empty()) {
                (128, _) => f.call(
                    &format!("__fix{u}{sfx}{size}"),
                    wt.clone(),
                    vec![(fty, a.clone())],
                    w == *n,
                ),
                (32, true) => {
                    f.fpu(FPU_TO_I32, &fty, vec![(fty.clone(), a.clone())], wt.clone(), w == *n)
                }
                (32, false) => {
                    f.fpu(FPU_TO_U32, &fty, vec![(fty.clone(), a.clone())], wt.clone(), w == *n)
                }
                (_, true) => {
                    f.fpu(FPU_TO_I64, &fty, vec![(fty.clone(), a.clone())], wt.clone(), w == *n)
                }
                (_, false) => {
                    f.fpu(FPU_TO_U64, &fty, vec![(fty.clone(), a.clone())], wt.clone(), w == *n)
                }
            };
            if w != *n {
                f.cast("trunc", &wt, v, to, true);
            }
        }
        "sitofp" | "uitofp" => {
            let (_, sfx) =
                float_kind(to).ok_or(format!("{op} to {} is not supported", shown(to)))?;
            let Ty::Int(n) = ty else {
                return Err(format!("{op} from {} is not supported", shown(ty)));
            };
            let (w, size) = int_width(*n)?;
            let (u, ext) = if op == "uitofp" { ("un", "zext") } else { ("", "sext") };
            let wt = Ty::Int(w);
            let v = if w != *n { f.cast(ext, ty, a.clone(), &wt, false) } else { a.clone() };
            let unsigned = !u.is_empty();
            let code = match (w, unsigned) {
                (128, _) => {
                    f.call(&format!("__float{u}{size}{sfx}"), to.clone(), vec![(wt, v)], true);
                    return Ok(true);
                }
                (32, false) => FPU_FROM_I32,
                (32, true) => FPU_FROM_U32,
                (_, false) => FPU_FROM_I64,
                (_, true) => FPU_FROM_U64,
            };
            f.fpu(code, to, vec![(wt, v)], to.clone(), true);
        }
        "fpext" | "fptrunc" => {
            match (float_kind(ty), float_kind(to)) {
                (Some((32, _)), Some((64, _))) | (Some((64, _)), Some((32, _))) => {
                    // The sign of FPU_CVT is the precision converted from.
                    f.fpu(FPU_CVT, &fty, vec![(fty.clone(), a.clone())], to.clone(), true);
                }
                _ => {
                    return Err(format!(
                        "{op} from {} to {} is not supported",
                        shown(ty),
                        shown(to)
                    ));
                }
            }
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// The binary floating point coprocessor's operations (docs/platform.md,
/// page −15); the sign of OP picks binary32 (−) or binary64 (+).
pub const FPU_ADD: i64 = 1;
pub const FPU_SUB: i64 = 2;
pub const FPU_MUL: i64 = 3;
pub const FPU_DIV: i64 = 4;
pub const FPU_SQRT: i64 = 5;
pub const FPU_FMA: i64 = 6;
pub const FPU_MIN: i64 = 7;
pub const FPU_MAX: i64 = 8;
pub const FPU_CMP: i64 = 9;
pub const FPU_CVT: i64 = 10;
pub const FPU_TO_I32: i64 = 11;
pub const FPU_TO_U32: i64 = 12;
pub const FPU_TO_I64: i64 = 13;
pub const FPU_TO_U64: i64 = 14;
pub const FPU_FROM_I32: i64 = 15;
pub const FPU_FROM_U32: i64 = 16;
pub const FPU_FROM_I64: i64 = 17;
pub const FPU_FROM_U64: i64 = 18;

/// The width an integer of `n` bits goes through a conversion as, and
/// the suffix for it: `si` (32), `di` (64) or `ti` (128).
fn int_width(n: u32) -> R<(u32, &'static str)> {
    Ok(match n {
        0..=32 => (32, "si"),
        33..=64 => (64, "di"),
        65..=128 => (128, "ti"),
        _ => return Err(format!("{n}-bit integers cannot be converted to floating point")),
    })
}

fn float_intrinsic(f: &mut FloatOps, name: &str, ret: &Ty, args: &[(Ty, Val)]) -> R<bool> {
    let parts: Vec<&str> = name.split('.').collect();
    let base = parts.get(1).copied().unwrap_or("");
    // Three-way comparisons, -1, 0 or 1: (a > b) - (a < b).
    if matches!(base, "ucmp" | "scmp") {
        let (u, it) = (if base == "ucmp" { "u" } else { "s" }, args[0].0.clone());
        let (a, b) = (args[0].1.clone(), args[1].1.clone());
        let lt = f.put(
            Op::Icmp { pred: format!("{u}lt"), ty: it.clone(), a: a.clone(), b: b.clone() },
            false,
        );
        let gt = f.put(Op::Icmp { pred: format!("{u}gt"), ty: it, a, b }, false);
        let lt = f.cast("zext", &Ty::Int(1), lt, ret, false);
        let gt = f.cast("zext", &Ty::Int(1), gt, ret, false);
        f.bin("sub", ret, gt, lt, true);
        return Ok(true);
    }
    // Minimum, maximum and saturating add and subtract of integers
    // wider than a word: a comparison and a select.
    if let Ty::Int(w) = ret
        && *w > 32
        && matches!(base, "umin" | "umax" | "smin" | "smax" | "usub" | "uadd")
    {
        let (a, b) = (args[0].1.clone(), args[1].1.clone());
        let icmp = |f: &mut FloatOps, pred: &str, x: Val, y: Val| {
            f.put(Op::Icmp { pred: pred.into(), ty: ret.clone(), a: x, b: y }, false)
        };
        let select = |f: &mut FloatOps, c: Val, x: Val, y: Val| {
            f.put(Op::Select { c, ty: ret.clone(), a: x, b: y }, true)
        };
        match base {
            "usub" => {
                let less = icmp(f, "ult", a.clone(), b.clone());
                let d = f.bin("sub", ret, a, b, false);
                select(f, less, Val::Const(Const::Int(0)), d);
            }
            "uadd" => {
                let s = f.bin("add", ret, a.clone(), b, false);
                let over = icmp(f, "ult", s.clone(), a);
                select(f, over, Val::Const(Const::Int(-1)), s);
            }
            _ => {
                let pred = match base {
                    "umin" => "ult",
                    "umax" => "ugt",
                    "smin" => "slt",
                    _ => "sgt",
                };
                let c = icmp(f, pred, a.clone(), b.clone());
                select(f, c, a, b);
            }
        }
        return Ok(true);
    }
    // Funnel shifts, which core uses: shifts and an or,
    // the amount mod the width, folded when it is a constant.
    if let ("fshl" | "fshr", Ty::Int(w)) = (base, ret) {
        let k = |v: i128| Val::Const(Const::Int(v));
        let (a, b) = (args[0].1.clone(), args[1].1.clone());
        let wm = *w as i128 - 1;
        let (c, inv) = match &args[2].1 {
            Val::Const(Const::Int(c)) => (k(c & wm), k(wm - (c & wm))),
            c => {
                let c = f.bin("and", ret, c.clone(), k(wm), false);
                let inv = f.bin("xor", ret, c.clone(), k(wm), false);
                (c, inv)
            }
        };
        let (hi, lo) = if base == "fshl" {
            let hi = f.bin("shl", ret, a, c, false);
            let b1 = f.bin("lshr", ret, b, k(1), false);
            (hi, f.bin("lshr", ret, b1, inv, false))
        } else {
            let lo = f.bin("lshr", ret, b, c, false);
            let a1 = f.bin("shl", ret, a, k(1), false);
            (f.bin("shl", ret, a1, inv, false), lo)
        };
        f.bin("or", ret, hi, lo, true);
        return Ok(true);
    }
    // `llvm.fptosi.sat.i32.f64` and its unsigned twin: the coprocessor's
    // conversions and lower/float.ll's already saturate to 32, 64 or 128 bits and turn a NaN
    // into 0, as Rust's `as` asks; narrower ones are clamped after.
    if (base == "fptosi" || base == "fptoui") && parts.get(2) == Some(&"sat") {
        let Ty::Int(n) = ret else { return Err(format!("{name} is not supported")) };
        let (w, _) = int_width(*n)?;
        let wt = Ty::Int(w);
        let (op, signed) = if base == "fptosi" { ("fptosi", true) } else { ("fptoui", false) };
        if w == *n {
            f.put(
                Op::Float {
                    op: op.into(),
                    ty: args[0].0.clone(),
                    a: args[0].1.clone(),
                    b: Val::Const(Const::Undef),
                    to: wt,
                },
                true,
            );
            return Ok(true);
        }
        let v = f.put(
            Op::Float {
                op: op.into(),
                ty: args[0].0.clone(),
                a: args[0].1.clone(),
                b: Val::Const(Const::Undef),
                to: wt.clone(),
            },
            false,
        );
        let clamp = |f: &mut FloatOps, which: &str, v: Val, k: i128| {
            let callee = format!("llvm.{which}.i{w}");
            f.call(
                &callee,
                wt.clone(),
                vec![(wt.clone(), v), (wt.clone(), Val::Const(Const::Int(k)))],
                false,
            )
        };
        let v = if signed {
            let v = clamp(f, "smax", v, -(1i128 << (n - 1)));
            clamp(f, "smin", v, (1i128 << (n - 1)) - 1)
        } else {
            clamp(f, "umin", v, (1i128 << n) - 1)
        };
        f.cast("trunc", &wt, v, ret, true);
        return Ok(true);
    }
    let Some(&last) = parts.last() else { return Ok(false) };
    let ft = match last {
        "f32" => Ty::Float("float".into()),
        "f64" => Ty::Float("double".into()),
        "f16" | "f80" | "f128" | "ppcf128" | "bf16" => {
            return Err(format!("{name}: only float and double are supported"));
        }
        _ => return Ok(false),
    };
    let (bits, _) = float_kind(&ft).unwrap_or((64, "df"));
    let it = Ty::Int(bits);
    let sign = 1i128 << (bits - 1);
    let suffix = if bits == 32 { "f" } else { "" };
    let hardware = match base {
        "sqrt" => Some(FPU_SQRT),
        "fma" | "fmuladd" => Some(FPU_FMA),
        "minnum" | "minimumnum" => Some(FPU_MIN),
        "maxnum" | "maximumnum" => Some(FPU_MAX),
        _ => None,
    };
    if let Some(n) = hardware {
        f.fpu(n, &ft, args.to_vec(), ret.clone(), true);
        return Ok(true);
    }
    let lib = match base {
        "floor" | "ceil" | "trunc" | "round" | "rint" | "roundeven" => base,
        "nearbyint" => "rint",
        "minimum" => "fminimum",
        "maximum" => "fmaximum",
        "fabs" => {
            let x = f.cast("bitcast", &ft, args[0].1.clone(), &it, false);
            let m = f.bin("and", &it, x, Val::Const(Const::Int(!sign)), false);
            f.cast("bitcast", &it, m, &ft, true);
            return Ok(true);
        }
        "copysign" => {
            let x = f.cast("bitcast", &ft, args[0].1.clone(), &it, false);
            let y = f.cast("bitcast", &ft, args[1].1.clone(), &it, false);
            let m = f.bin("and", &it, x, Val::Const(Const::Int(!sign)), false);
            let s = f.bin("and", &it, y, Val::Const(Const::Int(sign)), false);
            let r = f.bin("or", &it, m, s, false);
            f.cast("bitcast", &it, r, &ft, true);
            return Ok(true);
        }
        _ => return Err(format!("{name} is not supported")),
    };
    f.call(&format!("{lib}{suffix}"), ret.clone(), args.to_vec(), true);
    Ok(true)
}

/// An atomic operation as plain ones between [`Op::Lock`] and
/// [`Op::Unlock`]: the lock is one word for the whole program, taken
/// with `amoswap`, so that every atomic access is one at a time while
/// the rest of memory is reached as usual (docs/rust.md, Atomics). A
/// fence becomes nothing, as a hart sees its own accesses in order and
/// the lock orders those of others. Any other instruction is itself.
fn expand_atomic(i: Instr) -> Vec<Instr> {
    let Op::Atomic { op, ty, p, a, b } = i.op else { return vec![i] };
    if op == "fence" {
        return vec![];
    }
    let line = i.line;
    let res = i.res.unwrap_or_else(|| format!("at${line}"));
    let name = |k: &str| format!("{res}$at{k}");
    let ins = |res: String, op: Op| Instr { res: Some(res), op, line };
    let bare = |op: Op| Instr { res: None, op, line };
    // The value in memory before, named as the result but for cmpxchg.
    let old = if op == "cmpxchg" { name("old") } else { res.clone() };
    let oldv = Val::Local(old.clone());
    let bin = |o: &str, x: Val, y: Val| Op::Bin {
        op: o.into(),
        flags: vec![],
        ty: ty.clone(),
        a: x,
        b: y,
    };
    let icmp =
        |pred: &str, x: Val, y: Val| Op::Icmp { pred: pred.into(), ty: ty.clone(), a: x, b: y };
    let select =
        |c: String, x: Val, y: Val| Op::Select { c: Val::Local(c), ty: ty.clone(), a: x, b: y };
    let mut out = vec![bare(Op::Lock)];
    if op != "store" {
        out.push(ins(old.clone(), Op::Load { ty: ty.clone(), p: p.clone() }));
    }
    // The value to store, if any.
    let new = match op.as_str() {
        "load" => None,
        "store" | "xchg" => Some(a),
        "add" | "sub" | "and" | "or" | "xor" => {
            out.push(ins(name("new"), bin(&op, oldv.clone(), a)));
            Some(Val::Local(name("new")))
        }
        "nand" => {
            out.push(ins(name("and"), bin("and", oldv.clone(), a)));
            out.push(ins(
                name("new"),
                bin("xor", Val::Local(name("and")), Val::Const(Const::Int(-1))),
            ));
            Some(Val::Local(name("new")))
        }
        "max" | "min" | "umax" | "umin" => {
            let pred = match op.as_str() {
                "max" => "sgt",
                "min" => "slt",
                "umax" => "ugt",
                _ => "ult",
            };
            out.push(ins(name("keep"), icmp(pred, oldv.clone(), a.clone())));
            out.push(ins(name("new"), select(name("keep"), oldv.clone(), a)));
            Some(Val::Local(name("new")))
        }
        "cmpxchg" => {
            out.push(ins(name("eq"), icmp("eq", oldv.clone(), a)));
            out.push(ins(name("new"), select(name("eq"), b, oldv.clone())));
            Some(Val::Local(name("new")))
        }
        _ => return vec![ins(res, Op::Other(format!("atomicrmw {op}")))],
    };
    if let Some(v) = new {
        out.push(bare(Op::Store { ty: ty.clone(), v, p }));
    }
    out.push(bare(Op::Unlock));
    if op == "cmpxchg" {
        // `{ old, whether it was the one expected }`.
        let pair = Ty::Struct(vec![ty.clone(), Ty::Int(1)], false);
        let half = Op::Insert {
            ty: pair.clone(),
            agg: Val::Const(Const::Undef),
            vty: ty.clone(),
            v: oldv,
            idx: vec![0],
        };
        out.push(ins(name("half"), half));
        let whole = Op::Insert {
            ty: pair,
            agg: Val::Local(name("half")),
            vty: Ty::Int(1),
            v: Val::Local(name("eq")),
            idx: vec![1],
        };
        out.push(ins(res, whole));
    }
    out
}

const CONST_EXPRS: [&str; 20] = [
    "getelementptr",
    "ptrtoint",
    "inttoptr",
    "bitcast",
    "addrspacecast",
    "trunc",
    "zext",
    "sext",
    "add",
    "sub",
    "mul",
    "and",
    "or",
    "xor",
    "shl",
    "lshr",
    "ashr",
    "icmp",
    "select",
    "extractvalue",
];

/// Attributes that take one following number, not a parenthesised one.
const NUMBERED_ATTRS: [&str; 3] = ["align", "addrspace", "cc"];

impl P {
    /// The `align N` among the tokens still to read, or 1.
    fn align_after(&self) -> u64 {
        let rest = &self.t[self.i.min(self.t.len())..];
        rest.windows(2)
            .find_map(|w| match w {
                [Tok::Word(a), Tok::Int(k)] if a == "align" => Some(*k as u64),
                _ => None,
            })
            .unwrap_or(1)
    }

    fn peek(&self) -> Option<&Tok> {
        self.t.get(self.i)
    }

    fn peek_at(&self, k: usize) -> Option<&Tok> {
        self.t.get(self.i + k)
    }

    fn next(&mut self) -> R<Tok> {
        let t = self.t.get(self.i).cloned().ok_or("the line ends too soon")?;
        self.i += 1;
        Ok(t)
    }

    fn is_punct(&self, c: char) -> bool {
        self.peek() == Some(&Tok::Punct(c))
    }

    fn is_word(&self, w: &str) -> bool {
        matches!(self.peek(), Some(Tok::Word(x)) if x == w)
    }

    fn eat_punct(&mut self, c: char) -> bool {
        if self.is_punct(c) {
            self.i += 1;
            true
        } else {
            false
        }
    }

    fn eat_word(&mut self, w: &str) -> bool {
        if self.is_word(w) {
            self.i += 1;
            true
        } else {
            false
        }
    }

    fn punct(&mut self, c: char) -> R<()> {
        if self.eat_punct(c) {
            Ok(())
        } else {
            Err(format!("expected '{c}', found {}", self.shown()))
        }
    }

    fn word(&mut self, w: &str) -> R<()> {
        if self.eat_word(w) { Ok(()) } else { Err(format!("expected {w}, found {}", self.shown())) }
    }

    fn shown(&self) -> String {
        match self.peek() {
            None => "the end of the line".into(),
            Some(Tok::Word(w)) => w.clone(),
            Some(Tok::Local(n)) => format!("%{n}"),
            Some(Tok::Global(n)) => format!("@{n}"),
            Some(Tok::Int(v)) => v.to_string(),
            Some(Tok::Punct(c)) => format!("'{c}'"),
            Some(t) => format!("{t:?}"),
        }
    }

    fn int(&mut self) -> R<i128> {
        match self.next()? {
            Tok::Int(v) => Ok(v),
            t => Err(format!("expected a number, found {t:?}")),
        }
    }

    fn local(&mut self) -> R<String> {
        match self.next()? {
            Tok::Local(n) => Ok(n),
            t => Err(format!("expected a %name, found {t:?}")),
        }
    }

    /// Skip a balanced group starting at the current opening bracket.
    fn skip_group(&mut self) -> R<()> {
        let mut depth = 0;
        loop {
            match self.next()? {
                Tok::Punct('(' | '[' | '{' | '<') => depth += 1,
                Tok::Punct(')' | ']' | '}' | '>') => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(());
                    }
                }
                _ => {}
            }
        }
    }

    fn starts_type(&self) -> bool {
        match self.peek() {
            Some(Tok::Word(w)) => {
                matches!(
                    w.as_str(),
                    "void"
                        | "ptr"
                        | "half"
                        | "bfloat"
                        | "float"
                        | "double"
                        | "fp128"
                        | "x86_fp80"
                        | "ppc_fp128"
                        | "label"
                        | "metadata"
                ) || (w.starts_with('i')
                    && w.len() > 1
                    && w[1..].bytes().all(|b| b.is_ascii_digit()))
            }
            Some(Tok::Local(_)) | Some(Tok::Punct('[' | '{' | '<')) => true,
            _ => false,
        }
    }

    fn ty(&mut self) -> R<Ty> {
        let t = match self.next()? {
            Tok::Word(w) => match w.as_str() {
                "void" => Ty::Void,
                "ptr" => {
                    if self.eat_word("addrspace") {
                        self.skip_group()?;
                    }
                    Ty::Ptr
                }
                "label" => Ty::Label,
                "metadata" => Ty::Metadata,
                w if w.starts_with('i') && w[1..].bytes().all(|b| b.is_ascii_digit()) => {
                    Ty::Int(w[1..].parse().map_err(|_| format!("bad type {w}"))?)
                }
                w => Ty::Float(w.to_string()),
            },
            Tok::Local(n) => Ty::Named(n),
            Tok::Punct('[') => {
                let n = self.int()? as u64;
                self.word("x")?;
                let e = self.ty()?;
                self.punct(']')?;
                Ty::Array(n, Box::new(e))
            }
            Tok::Punct('{') => Ty::Struct(self.field_types('}')?, false),
            Tok::Punct('<') => {
                if self.eat_punct('{') {
                    let f = self.field_types('}')?;
                    self.punct('>')?;
                    Ty::Struct(f, true)
                } else {
                    self.eat_word("vscale");
                    self.eat_word("x");
                    let n = self.int()? as u64;
                    self.word("x")?;
                    let e = self.ty()?;
                    self.punct('>')?;
                    Ty::Vector(n, Box::new(e))
                }
            }
            t => return Err(format!("expected a type, found {t:?}")),
        };
        Ok(t)
    }

    fn field_types(&mut self, close: char) -> R<Vec<Ty>> {
        let mut out = Vec::new();
        if self.eat_punct(close) {
            return Ok(out);
        }
        loop {
            out.push(self.ty()?);
            if self.eat_punct(close) {
                return Ok(out);
            }
            self.punct(',')?;
        }
    }

    /// Parameter or return attributes: words (with a parenthesised or
    /// numbered argument) up to what starts a value.
    fn skip_attrs(&mut self) -> R<()> {
        loop {
            if self.starts_type() && matches!(self.peek(), Some(Tok::Word(_))) {
                return Ok(());
            }
            match self.peek() {
                Some(Tok::Word(w))
                    if !CONST_EXPRS.contains(&w.as_str())
                        && !is_float_word(w)
                        && !matches!(
                            w.as_str(),
                            "true" | "false" | "null" | "undef" | "poison" | "zeroinitializer"
                        ) =>
                {
                    let numbered = NUMBERED_ATTRS.contains(&w.as_str());
                    self.i += 1;
                    if self.is_punct('(') {
                        self.skip_group()?;
                    } else if numbered && matches!(self.peek(), Some(Tok::Int(_))) {
                        self.i += 1;
                    }
                }
                Some(Tok::Str(_)) => {
                    self.i += 1;
                    if self.eat_punct('=') {
                        self.next()?;
                    }
                }
                _ => return Ok(()),
            }
        }
    }

    fn konst(&mut self) -> R<Const> {
        match self.next()? {
            Tok::Int(v) => Ok(Const::Int(v)),
            Tok::Global(n) => Ok(Const::Global(n)),
            Tok::Bytes(b) => Ok(Const::Bytes(b)),
            Tok::Word(w) => match w.as_str() {
                "true" => Ok(Const::Int(1)),
                "false" => Ok(Const::Int(0)),
                "null" => Ok(Const::Null),
                "undef" | "poison" => Ok(Const::Undef),
                "zeroinitializer" => Ok(Const::Zero),
                w if is_float_word(w) => float_const(w),
                "getelementptr" => {
                    while self.eat_word("inbounds")
                        || self.eat_word("nuw")
                        || self.eat_word("nusw")
                        || self.eat_word("inrange")
                    {
                        if self.is_punct('(') && matches!(self.peek_at(1), Some(Tok::Int(_))) {
                            self.skip_group()?;
                        }
                    }
                    self.punct('(')?;
                    let src = self.ty()?;
                    self.punct(',')?;
                    let bt = self.ty()?;
                    let base = self.konst()?;
                    let mut idx = Vec::new();
                    while self.eat_punct(',') {
                        self.eat_word("inrange");
                        let t = self.ty()?;
                        idx.push((t, self.konst()?));
                    }
                    self.punct(')')?;
                    Ok(Const::Gep { src, base: Box::new((bt, base)), idx })
                }
                "ptrtoint" | "inttoptr" | "bitcast" | "addrspacecast" | "trunc" | "zext"
                | "sext" => {
                    self.punct('(')?;
                    let ft = self.ty()?;
                    let v = self.konst()?;
                    self.word("to")?;
                    let to = self.ty()?;
                    self.punct(')')?;
                    Ok(Const::Cast { op: w, from: Box::new((ft, v)), to })
                }
                "add" | "sub" | "mul" | "and" | "or" | "xor" | "shl" | "lshr" | "ashr" => {
                    while self.eat_word("nuw") || self.eat_word("nsw") {}
                    self.punct('(')?;
                    let at = self.ty()?;
                    let a = self.konst()?;
                    self.punct(',')?;
                    let bt = self.ty()?;
                    let b = self.konst()?;
                    self.punct(')')?;
                    Ok(Const::Bin { op: w, a: Box::new((at, a)), b: Box::new((bt, b)) })
                }
                w => Err(format!("constant {w} is not supported")),
            },
            Tok::Punct('[') => Ok(Const::Agg(self.typed_consts(']')?)),
            Tok::Punct('{') => Ok(Const::Agg(self.typed_consts('}')?)),
            Tok::Punct('<') => {
                if self.eat_punct('{') {
                    let v = self.typed_consts('}')?;
                    self.punct('>')?;
                    Ok(Const::Agg(v))
                } else {
                    Err("vector constants are not supported".into())
                }
            }
            t => Err(format!("expected a constant, found {t:?}")),
        }
    }

    fn typed_consts(&mut self, close: char) -> R<Vec<(Ty, Const)>> {
        let mut out = Vec::new();
        if self.eat_punct(close) {
            return Ok(out);
        }
        loop {
            let t = self.ty()?;
            self.skip_attrs()?;
            out.push((t, self.konst()?));
            if self.eat_punct(close) {
                return Ok(out);
            }
            self.punct(',')?;
        }
    }

    fn val(&mut self) -> R<Val> {
        if let Some(Tok::Local(_)) = self.peek() {
            return Ok(Val::Local(self.local()?));
        }
        Ok(Val::Const(self.konst()?))
    }

    /// `type [attrs] value`
    fn typed(&mut self) -> R<(Ty, Val)> {
        let t = self.ty()?;
        self.skip_attrs()?;
        Ok((t, self.val()?))
    }

    fn label(&mut self) -> R<String> {
        self.word("label")?;
        self.local()
    }

    fn flags(&mut self, allowed: &[&str]) -> Vec<String> {
        let mut out = Vec::new();
        while let Some(Tok::Word(w)) = self.peek() {
            if allowed.contains(&w.as_str()) {
                out.push(w.clone());
                self.i += 1;
            } else {
                break;
            }
        }
        out
    }

    fn instr(&mut self) -> R<Op> {
        let w = match self.next()? {
            Tok::Word(w) => w,
            t => return Err(format!("expected an instruction, found {t:?}")),
        };
        let w = match w.as_str() {
            "tail" | "musttail" | "notail" => {
                self.word("call")?;
                "call".to_string()
            }
            _ => w,
        };
        let fast = ["fast", "nnan", "ninf", "nsz", "arcp", "contract", "afn", "reassoc"];
        match w.as_str() {
            "add" | "sub" | "mul" | "udiv" | "sdiv" | "urem" | "srem" | "shl" | "lshr" | "ashr"
            | "and" | "or" | "xor" => {
                let flags = self.flags(&["nuw", "nsw", "exact", "disjoint"]);
                let ty = self.ty()?;
                let a = self.val()?;
                self.punct(',')?;
                let b = self.val()?;
                Ok(Op::Bin { op: w, flags, ty, a, b })
            }
            "icmp" => {
                self.eat_word("samesign");
                let pred = match self.next()? {
                    Tok::Word(p) => p,
                    t => return Err(format!("expected a predicate, found {t:?}")),
                };
                let ty = self.ty()?;
                let a = self.val()?;
                self.punct(',')?;
                let b = self.val()?;
                Ok(Op::Icmp { pred, ty, a, b })
            }
            "trunc" | "zext" | "sext" | "ptrtoint" | "inttoptr" | "bitcast" | "addrspacecast" => {
                let flags = self.flags(&["nuw", "nsw", "nneg"]);
                let from = self.ty()?;
                let v = self.val()?;
                self.word("to")?;
                let to = self.ty()?;
                Ok(Op::Cast { op: w, flags, from, v, to })
            }
            "select" => {
                self.flags(&fast);
                let (_, c) = self.typed()?;
                self.punct(',')?;
                let (ty, a) = self.typed()?;
                self.punct(',')?;
                let (_, b) = self.typed()?;
                Ok(Op::Select { c, ty, a, b })
            }
            "phi" => {
                self.flags(&fast);
                let ty = self.ty()?;
                let mut incoming = Vec::new();
                loop {
                    self.punct('[')?;
                    let v = self.val()?;
                    self.punct(',')?;
                    let l = self.local()?;
                    self.punct(']')?;
                    incoming.push((v, l));
                    if !self.eat_punct(',') {
                        break;
                    }
                    if !self.is_punct('[') {
                        break;
                    }
                }
                Ok(Op::Phi { ty, incoming })
            }
            "load" => {
                if self.eat_word("atomic") {
                    self.eat_word("volatile");
                    let ty = self.ty()?;
                    self.punct(',')?;
                    let (_, p) = self.typed()?;
                    let none = Val::Const(Const::Undef);
                    return Ok(Op::Atomic { op: w, ty, p, a: none.clone(), b: none });
                }
                self.eat_word("volatile");
                let ty = self.ty()?;
                self.punct(',')?;
                let (_, p) = self.typed()?;
                Ok(Op::Load { ty, p })
            }
            "store" => {
                if self.eat_word("atomic") {
                    self.eat_word("volatile");
                    let (ty, a) = self.typed()?;
                    self.punct(',')?;
                    let (_, p) = self.typed()?;
                    return Ok(Op::Atomic { op: w, ty, p, a, b: Val::Const(Const::Undef) });
                }
                self.eat_word("volatile");
                let (ty, v) = self.typed()?;
                self.punct(',')?;
                let (_, p) = self.typed()?;
                Ok(Op::Store { ty, v, p })
            }
            "alloca" => {
                self.eat_word("inalloca");
                let ty = self.ty()?;
                let mut n = None;
                if self.eat_punct(',') && !self.is_word("align") && !self.is_word("addrspace") {
                    n = Some(self.typed()?);
                }
                let align = self.align_after();
                Ok(Op::Alloca { ty, n, align })
            }
            "getelementptr" => {
                let flags = self.flags(&["inbounds", "nuw", "nusw"]);
                let src = self.ty()?;
                self.punct(',')?;
                let (_, base) = self.typed()?;
                let mut idx = Vec::new();
                while self.eat_punct(',') {
                    if matches!(self.peek(), Some(Tok::Meta(_))) {
                        break;
                    }
                    self.eat_word("inrange");
                    idx.push(self.typed()?);
                }
                Ok(Op::Gep { flags, src, base, idx })
            }
            "call" => {
                self.flags(&fast);
                // Calling convention and return attributes.
                while !self.starts_type() {
                    let before = self.i;
                    self.skip_attrs()?;
                    if self.i == before {
                        return Err(format!("unexpected {} in a call", self.shown()));
                    }
                }
                let ret = self.ty()?;
                // `ret (params)` for a function with varargs.
                if self.is_punct('(') {
                    self.skip_group()?;
                }
                let callee = self.val()?;
                self.punct('(')?;
                let mut args = Vec::new();
                if !self.eat_punct(')') {
                    loop {
                        let t = self.ty()?;
                        let v = if t == Ty::Metadata {
                            // An intrinsic's metadata argument: skipped.
                            while !self.is_punct(',') && !self.is_punct(')') {
                                self.next()?;
                            }
                            Val::Const(Const::Undef)
                        } else {
                            self.skip_attrs()?;
                            self.val()?
                        };
                        args.push((t, v));
                        if self.eat_punct(')') {
                            break;
                        }
                        self.punct(',')?;
                    }
                }
                Ok(Op::Call { ret, callee, args })
            }
            "br" => {
                if self.is_word("label") {
                    return Ok(Op::Jump(self.label()?));
                }
                let (_, c) = self.typed()?;
                self.punct(',')?;
                let t = self.label()?;
                self.punct(',')?;
                let f = self.label()?;
                Ok(Op::Br { c, t, f })
            }
            "switch" => {
                let (ty, v) = self.typed()?;
                self.punct(',')?;
                let default = self.label()?;
                self.punct('[')?;
                let mut cases = Vec::new();
                while !self.eat_punct(']') {
                    let _ = self.ty()?;
                    let c = match self.konst()? {
                        Const::Int(c) => c,
                        k => return Err(format!("switch case {k:?} is not a number")),
                    };
                    self.punct(',')?;
                    cases.push((c, self.label()?));
                }
                Ok(Op::Switch { ty, v, default, cases })
            }
            "ret" => {
                if self.eat_word("void") {
                    return Ok(Op::Ret(None));
                }
                Ok(Op::Ret(Some(self.typed()?)))
            }
            "unreachable" => Ok(Op::Unreachable),
            "extractvalue" => {
                let (ty, agg) = self.typed()?;
                let mut idx = Vec::new();
                while self.eat_punct(',') {
                    match self.peek() {
                        Some(Tok::Int(_)) => idx.push(self.int()? as u64),
                        _ => break,
                    }
                }
                Ok(Op::Extract { ty, agg, idx })
            }
            "insertvalue" => {
                let (ty, agg) = self.typed()?;
                self.punct(',')?;
                let (vty, v) = self.typed()?;
                let mut idx = Vec::new();
                while self.eat_punct(',') {
                    match self.peek() {
                        Some(Tok::Int(_)) => idx.push(self.int()? as u64),
                        _ => break,
                    }
                }
                Ok(Op::Insert { ty, agg, vty, v, idx })
            }
            "freeze" => {
                let (ty, v) = self.typed()?;
                Ok(Op::Freeze { ty, v })
            }
            "fadd" | "fsub" | "fmul" | "fdiv" | "frem" | "fcmp" | "fneg" => {
                self.flags(&fast);
                let op = if w == "fcmp" {
                    match self.next()? {
                        Tok::Word(p) => format!("fcmp {p}"),
                        t => return Err(format!("expected a predicate, found {t:?}")),
                    }
                } else {
                    w
                };
                let ty = self.ty()?;
                let a = self.val()?;
                let b = if op == "fneg" {
                    Val::Const(Const::Undef)
                } else {
                    self.punct(',')?;
                    self.val()?
                };
                Ok(Op::Float { op, ty: ty.clone(), a, b, to: ty })
            }
            "fptosi" | "fptoui" | "sitofp" | "uitofp" | "fpext" | "fptrunc" => {
                self.flags(&["nneg"]);
                let from = self.ty()?;
                let a = self.val()?;
                self.word("to")?;
                let to = self.ty()?;
                Ok(Op::Float { op: w, ty: from, a, b: Val::Const(Const::Undef), to })
            }
            "fence" => {
                let none = Val::Const(Const::Undef);
                Ok(Op::Atomic { op: w, ty: Ty::Void, p: none.clone(), a: none.clone(), b: none })
            }
            "atomicrmw" => {
                self.eat_word("volatile");
                let op = match self.next()? {
                    Tok::Word(op) => op,
                    t => return Err(format!("expected an operation, found {t:?}")),
                };
                let (_, p) = self.typed()?;
                self.punct(',')?;
                let (ty, a) = self.typed()?;
                Ok(Op::Atomic { op, ty, p, a, b: Val::Const(Const::Undef) })
            }
            "cmpxchg" => {
                self.eat_word("weak");
                self.eat_word("volatile");
                let (_, p) = self.typed()?;
                self.punct(',')?;
                let (ty, a) = self.typed()?;
                self.punct(',')?;
                let (_, b) = self.typed()?;
                Ok(Op::Atomic { op: w, ty, p, a, b })
            }
            w => Ok(Op::Other(format!("the {w} instruction"))),
        }
    }
}

// --- the module ---------------------------------------------------------

/// The header of `define`/`declare`: return type, name, parameters.
/// Return type, name, parameters and whether it takes varargs.
type Header = (Ty, String, Vec<(Ty, String)>, bool);

fn header(p: &mut P) -> R<Header> {
    while !p.starts_type() {
        match p.peek() {
            None => return Err("no return type".into()),
            Some(Tok::Word(_)) | Some(Tok::Str(_)) => {
                let before = p.i;
                p.skip_attrs()?;
                if p.i == before {
                    p.i += 1;
                }
            }
            _ => p.i += 1,
        }
    }
    let ret = p.ty()?;
    let name = match p.next()? {
        Tok::Global(n) => n,
        t => return Err(format!("expected a function name, found {t:?}")),
    };
    p.punct('(')?;
    let mut params = Vec::new();
    let mut varargs = false;
    let mut k = 0;
    if !p.eat_punct(')') {
        loop {
            if p.eat_word("...") {
                varargs = true;
                p.punct(')')?;
                break;
            }
            let t = p.ty()?;
            p.skip_attrs()?;
            let n = match p.peek() {
                Some(Tok::Local(_)) => p.local()?,
                _ => format!("$arg{k}"),
            };
            k += 1;
            params.push((t, n));
            if p.eat_punct(')') {
                break;
            }
            p.punct(',')?;
        }
    }
    Ok((ret, name, params, varargs))
}

/// A global's type, initialiser (None when external) and alignment.
type GlobalParts = (Ty, Option<Result<Const, String>>, u64);

fn global(p: &mut P, name: String) -> R<GlobalParts> {
    let mut external = false;
    loop {
        match p.next()? {
            Tok::Word(w) if w == "global" || w == "constant" => break,
            Tok::Word(w) if w == "external" || w == "extern_weak" => external = true,
            Tok::Word(w) if w == "alias" || w == "ifunc" => {
                return Err(format!("@{name} is an {w}, which is not supported"));
            }
            Tok::Word(w) if w == "addrspace" => p.skip_group()?,
            Tok::Word(w) if w == "thread_local" && p.is_punct('(') => {
                p.skip_group()?;
            }
            _ => {}
        }
    }
    let ty = p.ty()?;
    let init =
        if external || p.peek().is_none() || p.is_punct(',') { None } else { Some(p.konst()) };
    let align = p.align_after();
    Ok((ty, init, align))
}

/// The names a module keeps to itself (`private` and `internal`).
fn module_locals(text: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    for line in text.lines() {
        let local = line.starts_with("define") || line.starts_with('@');
        if !local {
            continue;
        }
        let Ok(toks) = tokens(line) else { continue };
        let mut name = None;
        let mut hidden = false;
        for t in &toks {
            match t {
                Tok::Global(n) if name.is_none() => name = Some(n.clone()),
                Tok::Word(w) if w == "private" || w == "internal" => hidden = true,
                Tok::Word(w) if w == "global" || w == "constant" => break,
                Tok::Punct('(') => break,
                _ => {}
            }
        }
        if hidden && let Some(n) = name {
            out.insert(n);
        }
    }
    out
}

/// `text` with each of `locals` named with `suffix` added, so that two
/// modules' private names (`@alloc_...`, `@vtable.0`) stay apart.
fn rename(text: &str, locals: &HashSet<String>, suffix: &str) -> String {
    let s = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    let mut start = 0;
    while i < s.len() {
        match s[i] {
            b'c' if s.get(i + 1) == Some(&b'"') && (i == 0 || !word_char(s[i - 1])) => {
                i += 2;
                while i < s.len() && s[i] != b'"' {
                    i += 1;
                }
                i += 1;
            }
            b';' => {
                while i < s.len() && s[i] != b'\n' {
                    i += 1;
                }
            }
            b'@' => {
                let at = i;
                i += 1;
                let Ok(n) = name_at(s, &mut i) else { continue };
                if locals.contains(&n) {
                    out.push_str(&text[start..at]);
                    let new = format!("{n}{suffix}");
                    if new.bytes().all(word_char) {
                        out.push('@');
                        out.push_str(&new);
                    } else {
                        out.push_str(&format!("@\"{new}\""));
                    }
                    start = i;
                }
            }
            _ => i += 1,
        }
    }
    out.push_str(&text[start..]);
    out
}

/// Read module number `index` of a program into `m`. The private names
/// of every module after the first get a suffix of their own.
pub fn parse(text: &str, file: &str, index: usize, m: &mut Module) -> Result<(), crate::Error> {
    let renamed;
    let text = if index == 0 {
        text
    } else {
        renamed = rename(text, &module_locals(text), &format!(".m{index}"));
        &renamed
    };
    let err = |line: usize, msg: String| crate::Error::new(file, line, msg);
    let lines: Vec<&str> = text.lines().collect();
    let mut n = 0;
    while n < lines.len() {
        let line = lines[n];
        let no = n + 1;
        n += 1;
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with(';') {
            continue;
        }
        if trimmed.starts_with("define") {
            let mut p = P { t: tokens(line).map_err(|e| err(no, e))?, i: 0 };
            p.word("define").map_err(|e| err(no, e))?;
            let (ret, name, params, varargs) = header(&mut p).map_err(|e| err(no, e))?;
            let mut f = Function {
                name: name.clone(),
                ret,
                params,
                varargs,
                blocks: Vec::new(),
                file: file.to_string(),
                line: no,
            };
            let mut cur = Block { label: "$entry".into(), instrs: Vec::new() };
            loop {
                if n >= lines.len() {
                    return Err(err(no, format!("@{name} has no closing brace")));
                }
                let mut body = lines[n].to_string();
                let bno = n + 1;
                n += 1;
                let t = body.trim();
                if t == "}" {
                    break;
                }
                if t.is_empty() || t.starts_with(';') {
                    continue;
                }
                // A label: `name:` at the start of the line.
                if !body.starts_with(' ') && !body.starts_with('\t') {
                    let toks = tokens(&body).map_err(|e| err(bno, e))?;
                    let label = match toks.first() {
                        Some(Tok::Word(w)) => w.clone(),
                        Some(Tok::Int(v)) => v.to_string(),
                        Some(Tok::Str(s)) => s.clone(),
                        _ => return Err(err(bno, "expected a label".into())),
                    };
                    if !cur.instrs.is_empty() || cur.label != "$entry" {
                        f.blocks
                            .push(std::mem::replace(&mut cur, Block { label, instrs: Vec::new() }));
                    } else {
                        cur.label = label;
                    }
                    continue;
                }
                // A switch's cases continue to the closing bracket.
                if t.ends_with('[') {
                    while n < lines.len() {
                        let more = lines[n];
                        n += 1;
                        body.push(' ');
                        body.push_str(more.trim());
                        if more.trim_start().starts_with(']') {
                            break;
                        }
                    }
                }
                let instr = match tokens(&body) {
                    Err(e) => Instr { res: None, op: Op::Other(e), line: bno },
                    Ok(toks) => {
                        let mut p = P { t: toks, i: 0 };
                        let mut res = None;
                        if let (Some(Tok::Local(r)), Some(Tok::Punct('='))) =
                            (p.peek(), p.peek_at(1))
                        {
                            res = Some(r.clone());
                            p.i += 2;
                        }
                        let op = p.instr().unwrap_or_else(Op::Other);
                        Instr { res, op, line: bno }
                    }
                };
                for i in expand_atomic(instr) {
                    cur.instrs.extend(expand_float(i));
                }
            }
            f.blocks.push(cur);
            m.source.insert(('f', name.clone()), lines[no - 1..n].join("\n"));
            m.funcs.insert(name, f);
        } else if trimmed.starts_with("declare") {
            let mut p = P { t: tokens(line).map_err(|e| err(no, e))?, i: 0 };
            p.word("declare").map_err(|e| err(no, e))?;
            let (_, name, _, _) = header(&mut p).map_err(|e| err(no, e))?;
            m.source.insert(('d', name.clone()), line.to_string());
            m.decls.insert(name);
        } else if trimmed.starts_with('@') {
            let mut p = P { t: tokens(line).map_err(|e| err(no, e))?, i: 0 };
            let name = match p.next() {
                Ok(Tok::Global(n)) => n,
                _ => return Err(err(no, "expected a global".into())),
            };
            p.punct('=').map_err(|e| err(no, e))?;
            if let Some(k) = p.t.iter().position(|t| *t == Tok::Word("alias".into())) {
                // `@a = ... alias TYPE, ptr @target`
                m.source.insert(('a', name.clone()), line.to_string());
                match p.t.get(k + 1..).and_then(|ts| {
                    ts.iter().rev().find_map(|t| match t {
                        Tok::Global(n) => Some(n.clone()),
                        _ => None,
                    })
                }) {
                    Some(target) => {
                        m.aliases.insert(name, target);
                    }
                    None => return Err(err(no, format!("@{name}: an alias of what?"))),
                }
                continue;
            }
            m.source.insert(('g', name.clone()), line.to_string());
            match global(&mut p, name.clone()) {
                Ok((ty, init, align)) => {
                    m.globals.insert(
                        name.clone(),
                        Global { name, ty, init, align, file: file.to_string(), line: no },
                    );
                }
                Err(e) => {
                    m.globals.insert(
                        name.clone(),
                        Global {
                            name,
                            ty: Ty::Int(8),
                            init: Some(Err(e)),
                            align: 1,
                            file: file.to_string(),
                            line: no,
                        },
                    );
                }
            }
        } else if trimmed.starts_with('%') {
            let mut p = P { t: tokens(line).map_err(|e| err(no, e))?, i: 0 };
            let name = p.local().map_err(|e| err(no, e))?;
            p.punct('=').map_err(|e| err(no, e))?;
            p.word("type").map_err(|e| err(no, e))?;
            m.source.insert(('t', name.clone()), line.to_string());
            if p.eat_word("opaque") {
                continue;
            }
            let ty = p.ty().map_err(|e| err(no, e))?;
            m.types.insert(name, ty);
        }
        // target, attributes, metadata, source_filename: nothing needed.
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_small_module() {
        let src = r#"
%T = type { ptr, [1 x i32] }
@s = private unnamed_addr constant [2 x i8] c"a\0A", align 1
@p = private unnamed_addr constant <{ ptr, [4 x i8] }> <{ ptr @s, [4 x i8] c"\01\00\00\00" }>, align 4
define dso_local noundef range(i32 0, 256) i32 @main(i32 noundef %argc, ptr noundef %argv) unnamed_addr #2 {
start:
  %x = alloca [3 x i8], align 1
  %0 = icmp ne ptr %argv, null
  br i1 %0, label %a, label %b

a:                                    ; preds = %start
  %_0 = tail call noundef i32 @strlen(ptr noundef nonnull readonly dereferenceable(1) %argv) #7
  switch i8 0, label %b [
    i8 0, label %b
  ]

b:
  %r = phi i32 [ 0, %start ], [ %_0, %a ]
  ret i32 %r
}
declare i32 @strlen(ptr)
"#;
        let mut m = Module::default();
        parse(src, "t.ll", 0, &mut m).unwrap();
        let f = &m.funcs["main"];
        assert_eq!(f.params.len(), 2);
        assert_eq!(f.blocks.len(), 3);
        for b in &f.blocks {
            for i in &b.instrs {
                assert!(!matches!(i.op, Op::Other(_)), "{i:?}");
            }
        }
        assert!(m.decls.contains("strlen"));
        assert_eq!(m.globals["s"].init, Some(Ok(Const::Bytes(vec![b'a', 10]))));
        assert!(matches!(m.types["T"], Ty::Struct(_, false)));
    }
}
