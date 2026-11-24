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
                    // A decimal float: a word, never lowered.
                    while i < s.len() && word_char(s[i]) {
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
                    return Err("atomic loads are not supported".into());
                }
                self.eat_word("volatile");
                let ty = self.ty()?;
                self.punct(',')?;
                let (_, p) = self.typed()?;
                Ok(Op::Load { ty, p })
            }
            "store" => {
                if self.eat_word("atomic") {
                    return Err("atomic stores are not supported".into());
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
                Ok(Op::Alloca { ty, n })
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
            "fence" => Ok(Op::Other("fence".into())),
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

fn global(p: &mut P, name: String) -> R<(Ty, Option<Result<Const, String>>)> {
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
    // Alignment and the rest are not needed: see docs/rust.md.
    Ok((ty, init))
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
                cur.instrs.push(instr);
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
            m.source.insert(('g', name.clone()), line.to_string());
            match global(&mut p, name.clone()) {
                Ok((ty, init)) => {
                    m.globals.insert(
                        name.clone(),
                        Global { name, ty, init, file: file.to_string(), line: no },
                    );
                }
                Err(e) => {
                    m.globals.insert(
                        name.clone(),
                        Global {
                            name,
                            ty: Ty::Int(8),
                            init: Some(Err(e)),
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
