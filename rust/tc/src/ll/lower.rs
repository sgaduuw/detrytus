//! LLVM IR into the back end's machine IR: binary integers on a
//! ternary machine (docs/rust.md).
//!
//! An integer of N bits lives in a word as its unsigned value, 0 to
//! 2^N - 1, and every operation that could leave that range wraps back
//! into it with `band`. A pointer is an address. A byte is a tryte
//! holding 0..255, and wider values are stored little-endian, a byte a
//! tryte, as LLVM's layout for a 32-bit target expects.

use std::collections::{BTreeMap, HashMap, HashSet};

use super::ir::*;
use crate::Error;
use crate::alloc;

const ARGS: [&str; 6] = ["a0", "a1", "a2", "a3", "a4", "a5"];

/// The widest integer handled: wider ones (u64, u128) need more than a
/// word, which comes later.
const MAX_BITS: u32 = 32;

type R<T> = Result<T, String>;

/// A phi: its result, type and (value, predecessor) pairs.
type Phi = (String, Ty, Vec<(Val, String)>);

#[derive(Clone)]
enum V {
    /// One register per scalar (aggregates are flattened).
    Regs(Vec<&'static str>),
    /// A pointer known as a base register and an offset.
    Addr(&'static str, i128),
}

pub struct Lower<'m> {
    m: &'m Module,
    syms: HashMap<String, String>,
    vregs: usize,
    // The function being lowered.
    fsym: String,
    body: Vec<String>,
    env: HashMap<String, V>,
    slots: i128,
    labels: HashMap<String, String>,
    /// For blocks with phis: their phis' (result, type, incoming).
    phis: HashMap<String, Vec<Phi>>,
    /// Compares whose only use is the branch ending their block.
    fused: HashMap<String, (String, Ty, Val, Val)>,
    /// Edge stubs made for the current block: label, phi copies, target.
    stubs: Vec<(String, String, String)>,
    stub_labels: HashMap<(String, String), String>,
    local_labels: usize,
    block: String,
}

fn mask(bits: u32) -> i128 {
    (1i128 << bits) - 1
}

const REGISTERS: [&str; 33] = [
    "zero", "a0", "a1", "a2", "a3", "a4", "a5", "t0", "t1", "t2", "t3", "t4", "t5", "t6", "s0",
    "s1", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "fp", "gp", "tp", "ra", "sp", "pc", "x", "r0",
    "r-0", "r+0",
];

/// An assembly name for an LLVM name: itself when the assembler
/// accepts it, else escaped.
fn symbol(name: &str) -> String {
    let ok = name.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '$'));
    let lower = name.to_ascii_lowercase();
    let register = REGISTERS.contains(&lower.as_str())
        || (lower.starts_with('r') && lower[1..].trim_start_matches('-').parse::<i32>().is_ok());
    if ok && !register {
        return name.to_string();
    }
    let mut s = String::from("ll$");
    for c in name.chars() {
        if c.is_ascii_alphanumeric() || c == '_' || c == '.' {
            s.push(c);
        } else {
            s.push_str(&format!("${:x}$", c as u32));
        }
    }
    s
}

/// What a value refers to among globals and functions.
fn const_refs(c: &Const, out: &mut Vec<String>) {
    match c {
        Const::Global(n) => out.push(n.clone()),
        Const::Agg(v) => v.iter().for_each(|(_, c)| const_refs(c, out)),
        Const::Gep { base, idx, .. } => {
            const_refs(&base.1, out);
            idx.iter().for_each(|(_, c)| const_refs(c, out));
        }
        Const::Cast { from, .. } => const_refs(&from.1, out),
        Const::Bin { a, b, .. } => {
            const_refs(&a.1, out);
            const_refs(&b.1, out);
        }
        _ => {}
    }
}

fn val_refs(v: &Val, out: &mut Vec<String>) {
    if let Val::Const(c) = v {
        const_refs(c, out)
    }
}

fn instr_refs(op: &Op, out: &mut Vec<String>) {
    match op {
        Op::Bin { a, b, .. } | Op::Icmp { a, b, .. } => {
            val_refs(a, out);
            val_refs(b, out);
        }
        Op::Cast { v, .. } | Op::Freeze { v, .. } => val_refs(v, out),
        Op::Select { c, a, b, .. } => {
            val_refs(c, out);
            val_refs(a, out);
            val_refs(b, out);
        }
        Op::Phi { incoming, .. } => incoming.iter().for_each(|(v, _)| val_refs(v, out)),
        Op::Load { p, .. } => val_refs(p, out),
        Op::Store { v, p, .. } => {
            val_refs(v, out);
            val_refs(p, out);
        }
        Op::Alloca { n: Some((_, v)), .. } => val_refs(v, out),
        Op::Gep { base, idx, .. } => {
            val_refs(base, out);
            idx.iter().for_each(|(_, v)| val_refs(v, out));
        }
        Op::Call { callee, args, .. } => {
            val_refs(callee, out);
            args.iter().for_each(|(_, v)| val_refs(v, out));
        }
        Op::Br { c, .. } => val_refs(c, out),
        Op::Switch { v, .. } => val_refs(v, out),
        Op::Ret(Some((_, v))) => val_refs(v, out),
        Op::Extract { agg, .. } => val_refs(agg, out),
        Op::Insert { agg, v, .. } => {
            val_refs(agg, out);
            val_refs(v, out);
        }
        _ => {}
    }
}

fn val_locals<'a>(v: &'a Val, out: &mut Vec<&'a str>) {
    if let Val::Local(n) = v {
        out.push(n)
    }
}

/// The locals an instruction reads.
fn instr_uses(op: &Op) -> Vec<&str> {
    let mut out = Vec::new();
    match op {
        Op::Bin { a, b, .. } | Op::Icmp { a, b, .. } => {
            val_locals(a, &mut out);
            val_locals(b, &mut out);
        }
        Op::Cast { v, .. } | Op::Freeze { v, .. } => val_locals(v, &mut out),
        Op::Select { c, a, b, .. } => {
            val_locals(c, &mut out);
            val_locals(a, &mut out);
            val_locals(b, &mut out);
        }
        Op::Phi { incoming, .. } => incoming.iter().for_each(|(v, _)| val_locals(v, &mut out)),
        Op::Load { p, .. } => val_locals(p, &mut out),
        Op::Store { v, p, .. } => {
            val_locals(v, &mut out);
            val_locals(p, &mut out);
        }
        Op::Alloca { n: Some((_, v)), .. } => val_locals(v, &mut out),
        Op::Gep { base, idx, .. } => {
            val_locals(base, &mut out);
            idx.iter().for_each(|(_, v)| val_locals(v, &mut out));
        }
        Op::Call { callee, args, .. } => {
            val_locals(callee, &mut out);
            args.iter().for_each(|(_, v)| val_locals(v, &mut out));
        }
        Op::Br { c, .. } => val_locals(c, &mut out),
        Op::Switch { v, .. } => val_locals(v, &mut out),
        Op::Ret(Some((_, v))) => val_locals(v, &mut out),
        Op::Extract { agg, .. } => val_locals(agg, &mut out),
        Op::Insert { agg, v, .. } => {
            val_locals(agg, &mut out);
            val_locals(v, &mut out);
        }
        _ => {}
    }
    out
}

/// Intrinsics that do nothing here.
const NOPS: [&str; 7] = [
    "llvm.lifetime.",
    "llvm.assume",
    "llvm.experimental.noalias.scope.decl",
    "llvm.dbg.",
    "llvm.sideeffect",
    "llvm.donothing",
    "llvm.var.annotation",
];

impl<'m> Lower<'m> {
    pub fn new(m: &'m Module) -> Self {
        Lower {
            m,
            syms: HashMap::new(),
            vregs: 0,
            fsym: String::new(),
            body: Vec::new(),
            env: HashMap::new(),
            slots: 0,
            labels: HashMap::new(),
            phis: HashMap::new(),
            fused: HashMap::new(),
            stubs: Vec::new(),
            stub_labels: HashMap::new(),
            local_labels: 0,
            block: String::new(),
        }
    }

    fn sym(&mut self, name: &str) -> String {
        if let Some(s) = self.syms.get(name) {
            return s.clone();
        }
        let s = symbol(name);
        self.syms.insert(name.to_string(), s.clone());
        s
    }

    // --- types ---------------------------------------------------------

    fn resolve(&self, t: &Ty) -> R<Ty> {
        match t {
            Ty::Named(n) => match self.m.types.get(n) {
                Some(t) => self.resolve(t),
                None => Err(format!("type %{n} is opaque or unknown")),
            },
            t => Ok(t.clone()),
        }
    }

    fn align(&self, t: &Ty) -> R<u64> {
        Ok(match self.resolve(t)? {
            Ty::Int(b) => (b.div_ceil(8) as u64).next_power_of_two().clamp(1, 16),
            Ty::Ptr => 4,
            Ty::Float(f) => match f.as_str() {
                "half" | "bfloat" => 2,
                "float" => 4,
                _ => 8,
            },
            Ty::Array(_, e) | Ty::Vector(_, e) => self.align(&e)?,
            Ty::Struct(fs, packed) => {
                if packed {
                    1
                } else {
                    let mut a = 1;
                    for f in &fs {
                        a = a.max(self.align(f)?);
                    }
                    a
                }
            }
            t => return Err(format!("{t:?} has no size")),
        })
    }

    fn size(&self, t: &Ty) -> R<u64> {
        Ok(match self.resolve(t)? {
            Ty::Int(b) => {
                let bytes = b.div_ceil(8) as u64;
                bytes.next_multiple_of(self.align(&Ty::Int(b))?)
            }
            Ty::Ptr => 4,
            Ty::Float(_) => self.align(t)?,
            Ty::Array(n, e) | Ty::Vector(n, e) => n * self.size(&e)?,
            Ty::Struct(fs, packed) => {
                let mut off: u64 = 0;
                for f in &fs {
                    if !packed {
                        off = off.next_multiple_of(self.align(f)?);
                    }
                    off += self.size(f)?;
                }
                if packed { off } else { off.next_multiple_of(self.align(t)?) }
            }
            t => return Err(format!("{t:?} has no size")),
        })
    }

    fn field_offset(&self, fs: &[Ty], packed: bool, k: usize) -> R<u64> {
        let mut off: u64 = 0;
        for (i, f) in fs.iter().enumerate() {
            if !packed {
                off = off.next_multiple_of(self.align(f)?);
            }
            if i == k {
                return Ok(off);
            }
            off += self.size(f)?;
        }
        Err(format!("field {k} of a struct of {}", fs.len()))
    }

    /// The scalars of a type, with their byte offsets.
    fn leaves(&self, t: &Ty, base: u64, out: &mut Vec<(u64, Ty)>) -> R<()> {
        match self.resolve(t)? {
            Ty::Int(b) => out.push((base, Ty::Int(self.bits(&Ty::Int(b))?))),
            Ty::Ptr => out.push((base, Ty::Ptr)),
            Ty::Array(n, e) => {
                let s = self.size(&e)?;
                for i in 0..n {
                    self.leaves(&e, base + i * s, out)?;
                }
            }
            Ty::Struct(fs, packed) => {
                for (k, f) in fs.iter().enumerate() {
                    let off = self.field_offset(&fs, packed, k)?;
                    self.leaves(f, base + off, out)?;
                }
            }
            Ty::Float(f) => return Err(format!("floating point ({f}) is not supported yet")),
            Ty::Vector(..) => return Err("vectors are not supported".into()),
            Ty::Void => {}
            t => return Err(format!("{t:?} is not a value type")),
        }
        Ok(())
    }

    fn leaf_types(&self, t: &Ty) -> R<Vec<(u64, Ty)>> {
        let mut out = Vec::new();
        self.leaves(t, 0, &mut out)?;
        Ok(out)
    }

    /// Where the element at an extractvalue/insertvalue path starts among
    /// the leaves, how many it has, and its type.
    fn leaf_range(&self, t: &Ty, idx: &[u64]) -> R<(usize, usize, Ty)> {
        let mut start = 0;
        let mut cur = self.resolve(t)?;
        for &i in idx {
            match cur.clone() {
                Ty::Struct(fs, _) => {
                    for f in &fs[..i as usize] {
                        start += self.leaf_types(f)?.len();
                    }
                    cur = self.resolve(&fs[i as usize])?;
                }
                Ty::Array(_, e) => {
                    start += i as usize * self.leaf_types(&e)?.len();
                    cur = self.resolve(&e)?;
                }
                t => return Err(format!("cannot index into {t:?}")),
            }
        }
        let n = self.leaf_types(&cur)?.len();
        Ok((start, n, cur))
    }

    fn bits(&self, t: &Ty) -> R<u32> {
        match self.resolve(t)? {
            Ty::Int(b) if b <= MAX_BITS => Ok(b),
            Ty::Int(b) => Err(format!("{b}-bit integers are not supported yet")),
            Ty::Ptr => Ok(32),
            t => Err(format!("{t:?} is not an integer")),
        }
    }

    // --- emitting --------------------------------------------------------

    fn emit(&mut self, s: impl AsRef<str>) {
        self.body.push(format!("        {}", s.as_ref()));
    }

    fn label(&mut self, l: &str) {
        self.body.push(format!("{l}:"));
    }

    fn vreg(&mut self) -> &'static str {
        self.vregs += 1;
        alloc::vreg(self.vregs)
    }

    fn new_label(&mut self) -> String {
        self.local_labels += 1;
        format!("{}$S{}", self.fsym, self.local_labels)
    }

    fn li(&mut self, v: i128) -> &'static str {
        if v == 0 {
            return "zero";
        }
        let r = self.vreg();
        self.emit(format!("li    {r}, {v}"));
        r
    }

    /// Wrap `r` in place to `bits` bits.
    fn wrap(&mut self, r: &'static str, bits: u32) {
        let m = self.li(mask(bits));
        self.emit(format!("band  {r}, {r}, {m}"));
    }

    /// The signed value of `r`, an unsigned `bits`-bit one.
    fn signed(&mut self, r: &'static str, bits: u32) -> &'static str {
        if r == "zero" {
            return r;
        }
        let h = self.li(1i128 << (bits - 1));
        let d = self.vreg();
        self.emit(format!("bxor  {d}, {r}, {h}"));
        self.emit(format!("sub   {d}, {d}, {h}"));
        d
    }

    fn op3(&mut self, op: &str, a: &str, b: &str) -> &'static str {
        let d = self.vreg();
        self.emit(format!("{op:5} {d}, {a}, {b}"));
        d
    }

    // --- constants -------------------------------------------------------

    /// A constant integer as its unsigned value in `bits` bits.
    fn canon(v: i128, bits: u32) -> i128 {
        v.rem_euclid(1i128 << bits)
    }

    /// A constant integer as a signed value, from its `bits`-bit form.
    fn as_signed(v: i128, bits: u32) -> i128 {
        let c = Self::canon(v, bits);
        if c >= 1i128 << (bits - 1) { c - (1i128 << bits) } else { c }
    }

    /// The assembler expression for a constant address, if it is one.
    fn const_addr(&mut self, c: &Const) -> R<Option<String>> {
        Ok(match c {
            Const::Global(n) => Some(self.sym(n)),
            Const::Gep { src, base, idx } => {
                let Some(b) = self.const_addr(&base.1)? else {
                    return Err("a constant getelementptr not based on a global".into());
                };
                let off = self.const_gep_offset(src, idx)?;
                Some(if off == 0 { b } else { format!("{b} + {off}") })
            }
            Const::Cast { op, from, .. }
                if matches!(op.as_str(), "ptrtoint" | "inttoptr" | "bitcast" | "addrspacecast") =>
            {
                self.const_addr(&from.1)?
            }
            _ => None,
        })
    }

    fn const_gep_offset(&self, src: &Ty, idx: &[(Ty, Const)]) -> R<i128> {
        let mut off = 0i128;
        let mut cur = src.clone();
        for (k, (it, ic)) in idx.iter().enumerate() {
            let Const::Int(i) = ic else {
                return Err(format!("constant index {ic:?}"));
            };
            let i = Self::as_signed(*i, self.bits(it).unwrap_or(32).max(1));
            if k == 0 {
                off += i * self.size(src)? as i128;
                continue;
            }
            match self.resolve(&cur)? {
                Ty::Struct(fs, packed) => {
                    off += self.field_offset(&fs, packed, i as usize)? as i128;
                    cur = fs[i as usize].clone();
                }
                Ty::Array(_, e) => {
                    off += i * self.size(&e)? as i128;
                    cur = *e;
                }
                t => return Err(format!("cannot index into {t:?}")),
            }
        }
        Ok(off)
    }

    /// A constant scalar's value, if it is a number.
    fn const_int(&self, c: &Const, t: &Ty) -> R<Option<i128>> {
        Ok(match c {
            Const::Int(v) => Some(Self::canon(*v, self.bits(t)?)),
            Const::Null | Const::Undef | Const::Zero => Some(0),
            _ => None,
        })
    }

    fn const_reg(&mut self, c: &Const, t: &Ty) -> R<&'static str> {
        if let Some(v) = self.const_int(c, t)? {
            return Ok(self.li(v));
        }
        if let Some(a) = self.const_addr(c)? {
            let r = self.vreg();
            self.emit(format!("la    {r}, {a}"));
            return Ok(r);
        }
        Err(format!("constant {c:?} is not supported"))
    }

    fn const_leaves(&mut self, c: &Const, t: &Ty) -> R<Vec<&'static str>> {
        let t = self.resolve(t)?;
        match (c, &t) {
            (Const::Undef | Const::Zero | Const::Null, _) => {
                Ok(vec!["zero"; self.leaf_types(&t)?.len()])
            }
            (Const::Agg(es), _) => {
                let mut out = Vec::new();
                for (et, ec) in es {
                    out.extend(self.const_leaves(ec, et)?);
                }
                Ok(out)
            }
            (Const::Bytes(bs), _) => Ok(bs.iter().map(|&b| self.li(b as i128)).collect()),
            _ => Ok(vec![self.const_reg(c, &t)?]),
        }
    }

    // --- values ----------------------------------------------------------

    fn leaves_of(&mut self, v: &Val, t: &Ty) -> R<Vec<&'static str>> {
        match v {
            Val::Const(c) => self.const_leaves(c, t),
            Val::Local(n) => match self.env.get(n).cloned() {
                Some(V::Regs(r)) => Ok(r),
                Some(V::Addr(base, off)) => Ok(vec![self.materialise(base, off)]),
                None => Err(format!("%{n} is used before it is defined")),
            },
        }
    }

    fn materialise(&mut self, base: &'static str, off: i128) -> &'static str {
        let d = self.vreg();
        if off == 0 {
            self.emit(format!("mv    {d}, {base}"));
        } else {
            self.emit(format!("addi  {d}, {base}, {off}"));
        }
        d
    }

    fn scalar(&mut self, v: &Val, t: &Ty) -> R<&'static str> {
        let l = self.leaves_of(v, t)?;
        if l.len() != 1 {
            return Err(format!("expected a single value, found {} parts", l.len()));
        }
        Ok(l[0])
    }

    /// A pointer as base and offset.
    fn addr(&mut self, p: &Val) -> R<(&'static str, i128)> {
        if let Val::Local(n) = p
            && let Some(V::Addr(b, o)) = self.env.get(n)
        {
            return Ok((b, *o));
        }
        Ok((self.scalar(p, &Ty::Ptr)?, 0))
    }

    fn set(&mut self, res: &Option<String>, regs: Vec<&'static str>) {
        if let Some(r) = res {
            self.env.insert(r.clone(), V::Regs(regs));
        }
    }

    // --- memory ----------------------------------------------------------

    fn load_scalar(&mut self, t: &Ty, base: &str, off: i128) -> R<&'static str> {
        let bytes = self.size(t)? as i128;
        if bytes > 4 {
            return Err(format!("{}-byte values are not supported yet", bytes));
        }
        let d = self.vreg();
        self.emit(format!("lt    {d}, {}({base})", off + bytes - 1));
        if bytes > 1 {
            let eight = self.li(8);
            for k in (0..bytes - 1).rev() {
                let t = self.vreg();
                self.emit(format!("bsh   {d}, {d}, {eight}"));
                self.emit(format!("lt    {t}, {}({base})", off + k));
                self.emit(format!("bor   {d}, {d}, {t}"));
            }
        }
        Ok(d)
    }

    fn store_scalar(&mut self, t: &Ty, v: &'static str, base: &str, off: i128) -> R<()> {
        let bytes = self.size(t)? as i128;
        if bytes > 4 {
            return Err(format!("{}-byte values are not supported yet", bytes));
        }
        if bytes == 1 || v == "zero" {
            for k in 0..bytes {
                self.emit(format!("st    {v}, {}({base})", off + k));
            }
            return Ok(());
        }
        let m = self.li(255);
        let back = self.li(-8);
        let mut cur = v;
        for k in 0..bytes {
            if k > 0 {
                let n = self.vreg();
                self.emit(format!("bsh   {n}, {cur}, {back}"));
                cur = n;
            }
            if k < bytes - 1 {
                let b = self.vreg();
                self.emit(format!("band  {b}, {cur}, {m}"));
                self.emit(format!("st    {b}, {}({base})", off + k));
            } else {
                self.emit(format!("st    {cur}, {}({base})", off + k));
            }
        }
        Ok(())
    }

    // --- instructions ----------------------------------------------------

    fn bin(&mut self, op: &str, flags: &[String], t: &Ty, a: &Val, b: &Val) -> R<&'static str> {
        let bits = self.bits(t)?;
        let nuw = flags.iter().any(|f| f == "nuw");
        let ra = self.scalar(a, t)?;
        let rb = self.scalar(b, t)?;
        let d = match op {
            "add" | "sub" => {
                let d = self.op3(op, ra, rb);
                if !nuw {
                    self.wrap(d, bits);
                }
                d
            }
            "mul" => self.mul(t, bits, nuw, a, b, ra, rb)?,
            "udiv" => self.op3("div", ra, rb),
            "urem" => self.op3("rem", ra, rb),
            "sdiv" | "srem" => {
                let sa = self.signed(ra, bits);
                let sb = self.signed(rb, bits);
                let d = self.op3(if op == "sdiv" { "div" } else { "rem" }, sa, sb);
                self.wrap(d, bits);
                d
            }
            "shl" => {
                let d = self.op3("bsh", ra, rb);
                if !nuw {
                    self.wrap(d, bits);
                }
                d
            }
            "lshr" | "ashr" => {
                let n = match b {
                    Val::Const(c) if self.const_int(c, t)?.is_some() => {
                        let v = self.const_int(c, t)?.unwrap();
                        self.li(-v)
                    }
                    _ => {
                        let n = self.vreg();
                        self.emit(format!("neg   {n}, {rb}"));
                        n
                    }
                };
                if op == "lshr" {
                    self.op3("bsh", ra, n)
                } else {
                    let sa = self.signed(ra, bits);
                    let d = self.op3("bsh", sa, n);
                    self.wrap(d, bits);
                    d
                }
            }
            "and" => self.op3("band", ra, rb),
            "or" => self.op3("bor", ra, rb),
            "xor" => self.op3("bxor", ra, rb),
            op => return Err(format!("the {op} instruction is not supported")),
        };
        Ok(d)
    }

    #[allow(clippy::too_many_arguments)]
    fn mul(
        &mut self,
        t: &Ty,
        bits: u32,
        nuw: bool,
        a: &Val,
        b: &Val,
        ra: &'static str,
        rb: &'static str,
    ) -> R<&'static str> {
        // A product that fits a word (about 2^41.8) wraps afterwards.
        let small = |v: Option<i128>| v.is_some_and(|v| v < 1 << (41 - bits.min(41)));
        let ka = match a {
            Val::Const(c) => self.const_int(c, t)?,
            _ => None,
        };
        let kb = match b {
            Val::Const(c) => self.const_int(c, t)?,
            _ => None,
        };
        if nuw || bits <= 20 || small(ka) || small(kb) {
            let d = self.op3("mul", ra, rb);
            if !nuw {
                self.wrap(d, bits);
            }
            return Ok(d);
        }
        // Halves of 16 bits: a*b = al*bl + ((ah*bl + al*bh) mod 2^16) * 2^16.
        let k = self.li(65536);
        let m = self.li(65535);
        let al = self.op3("band", ra, m);
        let ah = self.op3("div", ra, k);
        let bl = self.op3("band", rb, m);
        let bh = self.op3("div", rb, k);
        let p = self.op3("mul", al, bl);
        let q = self.op3("mul", ah, bl);
        let r = self.op3("mul", al, bh);
        self.emit(format!("add   {q}, {q}, {r}"));
        self.emit(format!("band  {q}, {q}, {m}"));
        self.emit(format!("mul   {q}, {q}, {k}"));
        self.emit(format!("add   {p}, {p}, {q}"));
        self.wrap(p, bits);
        Ok(p)
    }

    fn signed_pred(pred: &str) -> bool {
        pred.starts_with('s')
    }

    fn cmp_operands(
        &mut self,
        pred: &str,
        t: &Ty,
        a: &Val,
        b: &Val,
    ) -> R<(&'static str, &'static str)> {
        let bits = self.bits(t)?;
        let mut ra = self.scalar(a, t)?;
        let mut rb = self.scalar(b, t)?;
        if Self::signed_pred(pred) {
            ra = self.signed(ra, bits);
            rb = self.signed(rb, bits);
        }
        Ok((ra, rb))
    }

    /// The table that turns `cmp`'s -1/0/+1, with `rs2` = 1, into 1 when
    /// the predicate holds and 0 when not: entry 3(c+1)+2 for each c.
    fn icmp_table(pred: &str) -> R<i128> {
        let holds: &[i128] = match pred {
            "eq" => &[0],
            "ne" => &[-1, 1],
            "ult" | "slt" => &[-1],
            "ule" | "sle" => &[-1, 0],
            "ugt" | "sgt" => &[1],
            "uge" | "sge" => &[0, 1],
            p => return Err(format!("comparison {p} is not supported")),
        };
        Ok(holds.iter().map(|c| 3i128.pow((3 * (c + 1) + 2) as u32)).sum())
    }

    fn branch_op(pred: &str) -> R<&'static str> {
        Ok(match pred {
            "eq" => "beq",
            "ne" => "bne",
            "ult" | "slt" => "blt",
            "ule" | "sle" => "ble",
            "ugt" | "sgt" => "bgt",
            "uge" | "sge" => "bge",
            p => return Err(format!("comparison {p} is not supported")),
        })
    }

    fn icmp(&mut self, pred: &str, t: &Ty, a: &Val, b: &Val) -> R<&'static str> {
        let (ra, rb) = self.cmp_operands(pred, t, a, b)?;
        let table = Self::icmp_table(pred)?;
        let c = self.op3("cmp", ra, rb);
        let one = self.li(1);
        let d = self.vreg();
        self.emit(format!("tlut  {d}, {c}, {one}, {table}"));
        Ok(d)
    }

    fn cast(&mut self, op: &str, flags: &[String], from: &Ty, v: &Val, to: &Ty) -> R<&'static str> {
        self.bits(from)?;
        self.bits(to)?;
        let r = self.scalar(v, from)?;
        let d = self.vreg();
        self.emit(format!("mv    {d}, {r}"));
        match op {
            "trunc" => {
                if !flags.iter().any(|f| f == "nuw") {
                    let bits = self.bits(to)?;
                    self.wrap(d, bits);
                }
            }
            "zext" | "inttoptr" | "bitcast" | "addrspacecast" => {}
            "sext" => {
                let fb = self.bits(from)?;
                let tb = self.bits(to)?;
                let s = self.signed(r, fb);
                self.emit(format!("mv    {d}, {s}"));
                self.wrap(d, tb);
            }
            "ptrtoint" => {
                let tb = self.bits(to)?;
                if tb < 32 {
                    self.wrap(d, tb);
                }
            }
            op => return Err(format!("the {op} cast is not supported")),
        }
        Ok(d)
    }

    fn intrinsic(
        &mut self,
        name: &str,
        ret: &Ty,
        args: &[(Ty, Val)],
    ) -> R<Option<Vec<&'static str>>> {
        if NOPS.iter().any(|p| name.starts_with(p)) {
            return Ok(Some(vec![]));
        }
        let base = name.split('.').take(2).collect::<Vec<_>>().join(".");
        let arg = |s: &mut Self, k: usize| -> R<&'static str> {
            let (t, v) = &args[k];
            s.scalar(v, t)
        };
        let t = args.first().map(|a| a.0.clone()).unwrap_or(Ty::Void);
        Ok(Some(match base.as_str() {
            "llvm.umin" | "llvm.umax" => {
                let a = arg(self, 0)?;
                let b = arg(self, 1)?;
                vec![self.op3(if base == "llvm.umin" { "min" } else { "max" }, a, b)]
            }
            "llvm.smin" | "llvm.smax" => {
                let bits = self.bits(&t)?;
                let a = arg(self, 0)?;
                let b = arg(self, 1)?;
                let sa = self.signed(a, bits);
                let sb = self.signed(b, bits);
                let d = self.op3(if base == "llvm.smin" { "min" } else { "max" }, sa, sb);
                self.wrap(d, bits);
                vec![d]
            }
            "llvm.abs" => {
                let bits = self.bits(&t)?;
                let a = arg(self, 0)?;
                let s = self.signed(a, bits);
                let d = self.vreg();
                self.emit(format!("abs   {d}, {s}"));
                self.wrap(d, bits);
                vec![d]
            }
            "llvm.expect" => vec![arg(self, 0)?],
            "llvm.uadd" | "llvm.usub" if name.contains("with.overflow") => {
                let bits = self.bits(&t)?;
                let a = arg(self, 0)?;
                let b = arg(self, 1)?;
                let op = if base == "llvm.uadd" { "add" } else { "sub" };
                let d = self.op3(op, a, b);
                // Out of range above (add) or below zero (sub).
                let ov = self.vreg();
                if op == "add" {
                    let m = self.li(mask(bits));
                    let c = self.op3("cmp", d, m);
                    self.emit(format!("max   {ov}, {c}, zero"));
                } else {
                    let c = self.op3("cmp", d, "zero");
                    let n = self.op3("min", c, "zero");
                    self.emit(format!("neg   {ov}, {n}"));
                }
                self.wrap(d, bits);
                vec![d, ov]
            }
            "llvm.uadd" | "llvm.usub" if name.contains(".sat.") => {
                let bits = self.bits(&t)?;
                let a = arg(self, 0)?;
                let b = arg(self, 1)?;
                if base == "llvm.uadd" {
                    let d = self.op3("add", a, b);
                    let m = self.li(mask(bits));
                    vec![self.op3("min", d, m)]
                } else {
                    let d = self.op3("sub", a, b);
                    vec![self.op3("max", d, "zero")]
                }
            }
            "llvm.memcpy" | "llvm.memmove" | "llvm.memset" => {
                let f = &base[5..];
                self.call_named(f, &args[..3], ret)?
            }
            "llvm.trap" | "llvm.debugtrap" | "llvm.ubsantrap" => {
                self.emit("call  abort, 0");
                vec![]
            }
            _ => return Ok(None),
        }))
    }

    /// A call to `sym`, or through the register `via`.
    fn call_named(&mut self, sym: &str, args: &[(Ty, Val)], ret: &Ty) -> R<Vec<&'static str>> {
        self.call_to(Ok(sym), args, ret)
    }

    fn call_to(
        &mut self,
        callee: Result<&str, &'static str>,
        args: &[(Ty, Val)],
        ret: &Ty,
    ) -> R<Vec<&'static str>> {
        let mut regs = Vec::new();
        for (t, v) in args {
            regs.extend(self.leaves_of(v, t)?);
        }
        if regs.len() > ARGS.len() {
            return Err(format!(
                "a call with {} argument words (more than 6) is not supported yet",
                regs.len()
            ));
        }
        for (k, r) in regs.iter().enumerate() {
            self.emit(format!("mv    {}, {r}", ARGS[k]));
        }
        match callee {
            Ok(sym) => self.emit(format!("call  {sym}, {}", regs.len())),
            Err(via) => self.emit(format!("callr {via}, {}", regs.len())),
        }
        let n = self.leaf_types(ret)?.len();
        if n > 2 {
            return Err("a result of more than two words is not supported yet".into());
        }
        let mut out = Vec::new();
        for a in ARGS.iter().take(n) {
            let d = self.vreg();
            self.emit(format!("mv    {d}, {a}"));
            out.push(d);
        }
        Ok(out)
    }

    fn call(&mut self, ret: &Ty, callee: &Val, args: &[(Ty, Val)]) -> R<Vec<&'static str>> {
        let name = match callee {
            Val::Const(Const::Global(n)) => n,
            _ => {
                let r = self.scalar(callee, &Ty::Ptr)?;
                return self.call_to(Err(r), args, ret);
            }
        };
        if name.starts_with("llvm.") {
            return match self.intrinsic(name, ret, args)? {
                Some(r) => Ok(r),
                None => Err(format!("the intrinsic {name} is not supported yet")),
            };
        }
        let sym = self.sym(name);
        self.call_named(&sym, args, ret)
    }

    // --- control flow ----------------------------------------------------

    /// The label a branch from the current block to `to` goes to: the
    /// block's own, or a stub that first sets the block's phis.
    fn target(&mut self, to: &str) -> R<String> {
        let Some(label) = self.labels.get(to).cloned() else {
            return Err(format!("no block %{to}"));
        };
        if !self.phis.contains_key(to) {
            return Ok(label);
        }
        let key = (self.block.clone(), to.to_string());
        if let Some(s) = self.stub_labels.get(&key) {
            return Ok(s.clone());
        }
        let stub = self.new_label();
        self.stub_labels.insert(key, stub.clone());
        self.stubs.push((stub.clone(), to.to_string(), label));
        Ok(stub)
    }

    /// Emit the stubs the current block's terminator made.
    fn emit_stubs(&mut self) -> R<()> {
        let stubs = std::mem::take(&mut self.stubs);
        for (stub, to, label) in stubs {
            self.label(&stub);
            let phis = self.phis[&to].clone();
            let mut moves = Vec::new();
            for (res, t, incoming) in &phis {
                let Some((v, _)) = incoming.iter().find(|(_, from)| *from == self.block) else {
                    return Err(format!("phi %{res} has no value for %{}", self.block));
                };
                let src = self.leaves_of(v, t)?;
                let Some(V::Regs(dst)) = self.env.get(res).cloned() else { unreachable!() };
                for (d, s) in dst.iter().zip(src) {
                    let tmp = self.vreg();
                    self.emit(format!("mv    {tmp}, {s}"));
                    moves.push((*d, tmp));
                }
            }
            for (d, tmp) in moves {
                self.emit(format!("mv    {d}, {tmp}"));
            }
            self.emit(format!("j     {label}"));
        }
        Ok(())
    }

    fn terminator(&mut self, op: &Op) -> R<()> {
        match op {
            Op::Jump(l) => {
                let t = self.target(l)?;
                self.emit(format!("j     {t}"));
            }
            Op::Br { c, t, f } => {
                let tl = self.target(t)?;
                let fl = self.target(f)?;
                match c {
                    Val::Local(n) if self.fused.contains_key(n) => {
                        let (pred, ty, a, b) = self.fused[n].clone();
                        let (ra, rb) = self.cmp_operands(&pred, &ty, &a, &b)?;
                        let op = Self::branch_op(&pred)?;
                        self.emit(format!("{op:5} {ra}, {rb}, {tl}"));
                    }
                    Val::Const(k) => {
                        let v = self.const_int(k, &Ty::Int(1))?.unwrap_or(1);
                        let l = if v != 0 { &tl } else { &fl };
                        self.emit(format!("j     {l}"));
                        return self.emit_stubs();
                    }
                    _ => {
                        let r = self.scalar(c, &Ty::Int(1))?;
                        self.emit(format!("bnez  {r}, {tl}"));
                    }
                }
                self.emit(format!("j     {fl}"));
            }
            Op::Switch { ty, v, default, cases } => {
                let r = self.scalar(v, ty)?;
                let bits = self.bits(ty)?;
                for (c, l) in cases {
                    let t = self.target(l)?;
                    let k = self.li(Self::canon(*c, bits));
                    self.emit(format!("beq   {r}, {k}, {t}"));
                }
                let t = self.target(default)?;
                self.emit(format!("j     {t}"));
            }
            Op::Ret(v) => {
                if let Some((t, v)) = v {
                    let regs = self.leaves_of(v, t)?;
                    if regs.len() > 2 {
                        return Err("a result of more than two words is not supported yet".into());
                    }
                    for (k, r) in regs.iter().enumerate() {
                        self.emit(format!("mv    {}, {r}", ARGS[k]));
                    }
                }
                self.emit(format!("j     {}$ret", self.fsym));
            }
            Op::Unreachable => self.emit(format!("j     {}$ret", self.fsym)),
            _ => unreachable!(),
        }
        self.emit_stubs()
    }

    fn instr(&mut self, ins: &Instr) -> R<()> {
        let res = &ins.res;
        match &ins.op {
            Op::Bin { op, flags, ty, a, b } => {
                let d = self.bin(op, flags, ty, a, b)?;
                self.set(res, vec![d]);
            }
            Op::Icmp { pred, ty, a, b } => {
                if res.as_ref().is_some_and(|r| self.fused.contains_key(r)) {
                    return Ok(());
                }
                let d = self.icmp(pred, ty, a, b)?;
                self.set(res, vec![d]);
            }
            Op::Cast { op, flags, from, v, to } => {
                if op == "bitcast" || op == "addrspacecast" || op == "inttoptr" {
                    // A pointer stays what it was, an Addr included.
                    if let Val::Local(n) = v
                        && let Some(V::Addr(b, o)) = self.env.get(n).cloned()
                        && let Some(r) = res
                    {
                        self.env.insert(r.clone(), V::Addr(b, o));
                        return Ok(());
                    }
                }
                let d = self.cast(op, flags, from, v, to)?;
                self.set(res, vec![d]);
            }
            Op::Select { c, ty, a, b } => {
                let rc = self.scalar(c, &Ty::Int(1))?;
                let la = self.leaves_of(a, ty)?;
                let lb = self.leaves_of(b, ty)?;
                let ds: Vec<&'static str> = (0..la.len()).map(|_| self.vreg()).collect();
                for (d, s) in ds.iter().zip(&lb) {
                    self.emit(format!("mv    {d}, {s}"));
                }
                let skip = self.new_label();
                self.emit(format!("beqz  {rc}, {skip}"));
                for (d, s) in ds.iter().zip(&la) {
                    self.emit(format!("mv    {d}, {s}"));
                }
                self.label(&skip);
                self.set(res, ds);
            }
            Op::Phi { .. } => {}
            Op::Load { ty, p } => {
                let (base, off) = self.addr(p)?;
                let mut regs = Vec::new();
                for (loff, lt) in self.leaf_types(ty)? {
                    regs.push(self.load_scalar(&lt, base, off + loff as i128)?);
                }
                self.set(res, regs);
            }
            Op::Store { ty, v, p } => {
                let regs = self.leaves_of(v, ty)?;
                let (base, off) = self.addr(p)?;
                for ((loff, lt), r) in self.leaf_types(ty)?.into_iter().zip(regs) {
                    self.store_scalar(&lt, r, base, off + loff as i128)?;
                }
            }
            Op::Alloca { ty, n } => {
                let count = match n {
                    None => 1,
                    Some((nt, Val::Const(c))) => match self.const_int(c, nt)? {
                        Some(k) => k as u64,
                        None => return Err("a variable-sized alloca is not supported".into()),
                    },
                    Some(_) => return Err("a variable-sized alloca is not supported".into()),
                };
                let size = self.size(ty)? * count;
                let words = (size.div_ceil(3)).max(1) as i128;
                self.slots += words;
                let off = -3 * (2 + self.slots) - 1;
                if let Some(r) = res {
                    self.env.insert(r.clone(), V::Addr("s0", off));
                }
            }
            Op::Gep { flags, src, base, idx } => {
                let nuw = flags.iter().any(|f| f == "nuw");
                let (b, mut off) = self.addr(base)?;
                let mut terms: Vec<(&'static str, u64)> = Vec::new();
                let mut cur = src.clone();
                for (k, (it, iv)) in idx.iter().enumerate() {
                    let elem = if k == 0 {
                        src.clone()
                    } else {
                        match self.resolve(&cur)? {
                            Ty::Struct(fs, packed) => {
                                let Val::Const(Const::Int(i)) = iv else {
                                    return Err("a struct index that is not constant".into());
                                };
                                off += self.field_offset(&fs, packed, *i as usize)? as i128;
                                cur = fs[*i as usize].clone();
                                continue;
                            }
                            Ty::Array(_, e) => *e,
                            t => return Err(format!("cannot index into {t:?}")),
                        }
                    };
                    let scale = self.size(&elem)?;
                    let ibits = self.bits(it)?;
                    match iv {
                        Val::Const(c) if self.const_int(c, it)?.is_some() => {
                            let i = Self::as_signed(self.const_int(c, it)?.unwrap(), ibits);
                            off += i * scale as i128;
                        }
                        _ => {
                            let mut r = self.scalar(iv, it)?;
                            if !nuw {
                                r = self.signed(r, ibits);
                            }
                            terms.push((r, scale));
                        }
                    }
                    if k > 0 {
                        cur = elem;
                    }
                }
                let mut acc = b;
                for (r, s) in terms {
                    let term = if s == 1 {
                        r
                    } else {
                        let k = self.li(s as i128);
                        self.op3("mul", r, k)
                    };
                    acc = self.op3("add", acc, term);
                }
                if let Some(r) = res {
                    self.env.insert(r.clone(), V::Addr(acc, off));
                }
            }
            Op::Call { ret, callee, args } => {
                let regs = self.call(ret, callee, args)?;
                self.set(res, regs);
            }
            Op::Extract { ty, agg, idx } => {
                let all = self.leaves_of(agg, ty)?;
                let (s, n, _) = self.leaf_range(ty, idx)?;
                self.set(res, all[s..s + n].to_vec());
            }
            Op::Insert { ty, agg, vty, v, idx } => {
                let mut all = self.leaves_of(agg, ty)?;
                let new = self.leaves_of(v, vty)?;
                let (s, n, _) = self.leaf_range(ty, idx)?;
                all.splice(s..s + n, new);
                self.set(res, all);
            }
            Op::Freeze { ty, v } => {
                let src = self.leaves_of(v, ty)?;
                let mut out = Vec::new();
                for s in src {
                    let d = self.vreg();
                    self.emit(format!("mv    {d}, {s}"));
                    out.push(d);
                }
                self.set(res, out);
            }
            Op::Other(what) => return Err(format!("{what} is not supported")),
            Op::Br { .. } | Op::Jump(_) | Op::Switch { .. } | Op::Ret(_) | Op::Unreachable => {
                self.terminator(&ins.op)?
            }
        }
        Ok(())
    }

    pub fn function(&mut self, f: &Function) -> Result<Vec<String>, Error> {
        let fail =
            |line: usize, msg: String| Error::new(&f.file, line, format!("@{}: {msg}", f.name));
        self.fsym = self.sym(&f.name);
        self.body.clear();
        self.env.clear();
        self.slots = 0;
        self.labels.clear();
        self.phis.clear();
        self.fused.clear();
        self.stubs.clear();
        self.stub_labels.clear();
        self.local_labels = 0;
        if f.varargs {
            return Err(fail(f.line, "functions with varargs are not supported".into()));
        }
        for (i, b) in f.blocks.iter().enumerate() {
            self.labels.insert(b.label.clone(), format!("{}$L{i}", self.fsym));
        }
        // Uses, for fusing a compare into its branch.
        let mut uses: HashMap<&str, usize> = HashMap::new();
        for b in &f.blocks {
            for i in &b.instrs {
                for u in instr_uses(&i.op) {
                    *uses.entry(u).or_default() += 1;
                }
            }
        }
        for b in &f.blocks {
            if let Some(Instr { op: Op::Br { c: Val::Local(c), .. }, .. }) = b.instrs.last()
                && uses.get(c.as_str()) == Some(&1)
                && let Some(Instr { op: Op::Icmp { pred, ty, a, b: bb }, .. }) =
                    b.instrs.iter().find(|i| i.res.as_deref() == Some(c))
            {
                self.fused.insert(c.clone(), (pred.clone(), ty.clone(), a.clone(), bb.clone()));
            }
            for i in &b.instrs {
                if let Op::Phi { ty, incoming } = &i.op {
                    let res = i.res.clone().unwrap_or_default();
                    let n = self.leaf_types(ty).map_err(|e| fail(i.line, e))?.len();
                    let regs: Vec<&'static str> = (0..n).map(|_| self.vreg()).collect();
                    self.env.insert(res.clone(), V::Regs(regs));
                    self.phis.entry(b.label.clone()).or_default().push((
                        res,
                        ty.clone(),
                        incoming.clone(),
                    ));
                }
            }
        }
        // Parameters arrive in a0.. (abi.md 2.2).
        let mut k = 0;
        for (t, name) in &f.params {
            let n = self.leaf_types(t).map_err(|e| fail(f.line, e))?.len();
            let mut regs = Vec::new();
            for _ in 0..n {
                if k >= ARGS.len() {
                    return Err(fail(
                        f.line,
                        "more than 6 parameter words are not supported yet".into(),
                    ));
                }
                let r = self.vreg();
                self.emit(format!("mv    {r}, {}", ARGS[k]));
                regs.push(r);
                k += 1;
            }
            self.env.insert(name.clone(), V::Regs(regs));
        }
        let results = self.leaf_types(&f.ret).map_err(|e| fail(f.line, e))?.len();
        if results > 2 {
            return Err(fail(
                f.line,
                "a result of more than two words is not supported yet".into(),
            ));
        }
        // Blocks are lowered so that each comes after the blocks that
        // dominate it (reverse postorder), since LLVM may place a block
        // before the one defining a value it uses, but are written out
        // in their own order.
        let start = std::mem::take(&mut self.body);
        let mut code: Vec<Vec<String>> = vec![Vec::new(); f.blocks.len()];
        for k in reverse_postorder(f) {
            let b = &f.blocks[k];
            self.block = b.label.clone();
            let l = self.labels[&b.label].clone();
            self.label(&l);
            for i in &b.instrs {
                self.instr(i).map_err(|e| fail(i.line, e))?;
            }
            code[k] = std::mem::take(&mut self.body);
        }
        self.body = start;
        self.body.extend(code.into_iter().flatten());
        let ret = format!("        j     {}$ret", self.fsym);
        if self.body.last() == Some(&ret) {
            self.body.pop();
        }
        let body = std::mem::take(&mut self.body);
        let done = alloc::finish(alloc::Func {
            name: &self.fsym.clone(),
            results,
            slots: self.slots,
            stack_params: false,
            body,
        })
        .map_err(|m| fail(f.line, m))?;
        let mut out = vec![String::new(), format!("# @{}, {}:{}", f.name, f.file, f.line)];
        out.push(format!("{}:", self.fsym));
        out.extend(done);
        Ok(out)
    }

    // --- data --------------------------------------------------------------

    /// A constant's bytes, each a tryte expression.
    fn bytes(&mut self, c: &Const, t: &Ty, out: &mut Vec<String>) -> R<()> {
        let t = self.resolve(t)?;
        let size = self.size(&t)? as usize;
        let start = out.len();
        match (c, &t) {
            (Const::Zero | Const::Undef | Const::Null, _) => {
                out.extend(std::iter::repeat_n("0".to_string(), size));
            }
            (Const::Bytes(bs), _) => out.extend(bs.iter().map(|b| b.to_string())),
            (Const::Agg(es), Ty::Struct(fs, packed)) => {
                for (k, (et, ec)) in es.iter().enumerate() {
                    let off = self.field_offset(fs, *packed, k)? as usize;
                    while out.len() - start < off {
                        out.push("0".into());
                    }
                    self.bytes(ec, et, out)?;
                }
            }
            (Const::Agg(es), _) => {
                for (et, ec) in es {
                    self.bytes(ec, et, out)?;
                }
            }
            (Const::Int(v), Ty::Int(b)) => {
                if *b > 64 {
                    return Err(format!("{b}-bit constants are not supported"));
                }
                let mut v = Self::canon(*v, *b);
                for _ in 0..size {
                    out.push((v % 256).to_string());
                    v /= 256;
                }
            }
            _ => {
                let Some(a) = self.const_addr(c)? else {
                    return Err(format!("constant {c:?} is not supported in data"));
                };
                let mut div = 1i128;
                for _ in 0..size {
                    out.push(if div == 1 {
                        format!("({a}) % 256")
                    } else {
                        format!("({a}) / {div} % 256")
                    });
                    div *= 256;
                }
            }
        }
        while out.len() - start < size {
            out.push("0".into());
        }
        Ok(())
    }

    pub fn global(&mut self, g: &Global) -> Result<Vec<String>, Error> {
        let fail = |msg: String| Error::new(&g.file, g.line, format!("@{}: {msg}", g.name));
        let Some(init) = &g.init else { return Ok(vec![]) };
        let init = init.clone().map_err(fail)?;
        let mut bytes = Vec::new();
        self.bytes(&init, &g.ty, &mut bytes).map_err(fail)?;
        let mut out = vec![format!("{}:", self.sym(&g.name))];
        if bytes.is_empty() {
            return Ok(out);
        }
        for chunk in bytes.chunks(16) {
            out.push(format!("        .tryte {}", chunk.join(", ")));
        }
        Ok(out)
    }

    /// Functions and globals reachable from `roots`, in the order found;
    /// and the external names they use.
    pub fn reachable(&self, roots: &[&str]) -> (Vec<String>, Vec<String>, BTreeMap<String, ()>) {
        let mut seen = HashSet::new();
        let mut funcs = Vec::new();
        let mut globals = Vec::new();
        let mut external = BTreeMap::new();
        let mut todo: Vec<String> = roots.iter().rev().map(|s| s.to_string()).collect();
        while let Some(n) = todo.pop() {
            if !seen.insert(n.clone()) {
                continue;
            }
            let mut refs = Vec::new();
            if let Some(f) = self.m.funcs.get(&n) {
                funcs.push(n.clone());
                for b in &f.blocks {
                    for i in &b.instrs {
                        instr_refs(&i.op, &mut refs);
                    }
                }
            } else if let Some(g) = self.m.globals.get(&n) {
                globals.push(n.clone());
                if let Some(Ok(c)) = &g.init {
                    const_refs(c, &mut refs);
                }
            } else if !n.starts_with("llvm.") {
                external.insert(n.clone(), ());
            }
            for r in refs.into_iter().rev() {
                if !seen.contains(&r) {
                    todo.push(r);
                }
            }
        }
        (funcs, globals, external)
    }
}

/// The blocks of `f` that its entry reaches, in reverse postorder: every
/// block after those that dominate it.
fn reverse_postorder(f: &Function) -> Vec<usize> {
    let index: HashMap<&str, usize> =
        f.blocks.iter().enumerate().map(|(k, b)| (b.label.as_str(), k)).collect();
    let succs = |k: usize| -> Vec<usize> {
        let labels: Vec<&String> = match f.blocks[k].instrs.last().map(|i| &i.op) {
            Some(Op::Jump(l)) => vec![l],
            Some(Op::Br { t, f, .. }) => vec![t, f],
            Some(Op::Switch { default, cases, .. }) => {
                std::iter::once(default).chain(cases.iter().map(|(_, l)| l)).collect()
            }
            _ => Vec::new(),
        };
        labels.into_iter().filter_map(|l| index.get(l.as_str()).copied()).collect()
    };
    let mut seen = vec![false; f.blocks.len()];
    let mut post = Vec::new();
    // Depth first without recursion: (block, successors still to visit).
    let mut stack: Vec<(usize, Vec<usize>)> = Vec::new();
    if !f.blocks.is_empty() {
        seen[0] = true;
        let mut s = succs(0);
        s.reverse();
        stack.push((0, s));
    }
    while let Some((k, rest)) = stack.last_mut() {
        if let Some(n) = rest.pop() {
            if !seen[n] {
                seen[n] = true;
                let mut s = succs(n);
                s.reverse();
                stack.push((n, s));
            }
        } else {
            post.push(*k);
            stack.pop();
        }
    }
    post.reverse();
    post
}
