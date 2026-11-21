//! The instruction set as data: opcodes, formats, fields, registers and
//! trap causes, and decoding with the spec's legality rules.
//!
//! The table is `docs/isa.md` section 7, and `tests/spec_sync.rs`
//! checks the two agree.

use crate::trits::{field, low};

/// An instruction format (docs/isa.md 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    R,
    I,
    S,
    B,
    T,
    U,
    J,
}

/// What an instruction does: one variant per mnemonic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Add,
    Sub,
    Mul,
    Mulh,
    Div,
    Rem,
    Divr,
    Remr,
    Cmp,
    Min,
    Max,
    Abs,
    Sel,
    Sh,
    Rot,
    Shi,
    Roti,
    Tex,
    Tins,
    Lzt,
    Tzt,
    Nzt,
    Tlut,
    Band,
    Bor,
    Bxor,
    Bsh,
    Addi,
    Cmpi,
    Lui,
    Auipc,
    Lt,
    Lw,
    St,
    Sw,
    Lr,
    Sc,
    Amoswap,
    Amoadd,
    Amomin,
    Amomax,
    Amotlut,
    Cas,
    Wrs,
    Jal,
    Jalr,
    B,
    Br3,
    Sys,
    Tret,
    Csrrw,
    Csrrtmax,
    Csrrtmin,
    Fence,
    FenceI,
    FenceVm,
    Wfi,
    Brk,
    Cclean,
    Cflush,
    Cinval,
}

/// One instruction: its mnemonic, opcode, format and operand syntax.
///
/// Syntax letters, as in the Python table: d rd, s rs1, t rs2, u rs3,
/// i immediate, m imm(rs1), a (rs1), L target, N/P br3 targets, x tlut
/// table, c CSR, pos/len trit field, F fence sets.
#[derive(Debug, PartialEq, Eq)]
pub struct Op {
    pub kind: Kind,
    pub name: &'static str,
    pub opcode: i64,
    pub format: Format,
    pub syntax: &'static str,
}

impl Op {
    fn has(&self, letter: &str) -> bool {
        self.syntax.split(',').any(|s| s == letter)
    }

    pub fn atomic(&self) -> bool {
        (-27..=-20).contains(&self.opcode)
    }
}

const fn op(
    kind: Kind,
    name: &'static str,
    opcode: i64,
    format: Format,
    syntax: &'static str,
) -> Op {
    Op { kind, name, opcode, format, syntax }
}

use Format::{B as FB, I, J, R, S, T, U};

pub const OPS: &[Op] = &[
    // 7.1 integer arithmetic
    op(Kind::Add, "add", 1, R, "d,s,t"),
    op(Kind::Sub, "sub", 2, R, "d,s,t"),
    op(Kind::Mul, "mul", 3, R, "d,s,t"),
    op(Kind::Mulh, "mulh", 4, R, "d,s,t"),
    op(Kind::Div, "div", 5, R, "d,s,t"),
    op(Kind::Rem, "rem", 6, R, "d,s,t"),
    op(Kind::Divr, "divr", 7, R, "d,s,t"),
    op(Kind::Remr, "remr", 8, R, "d,s,t"),
    // 7.2 compare, select, min/max
    op(Kind::Cmp, "cmp", 9, R, "d,s,t"),
    op(Kind::Min, "min", 10, R, "d,s,t"),
    op(Kind::Max, "max", 11, R, "d,s,t"),
    op(Kind::Abs, "abs", 12, R, "d,s"),
    op(Kind::Sel, "sel", 13, R, "d,s,t,u"),
    // 7.3 shifts and trit fields
    op(Kind::Sh, "sh", 14, R, "d,s,t"),
    op(Kind::Rot, "rot", 15, R, "d,s,t"),
    op(Kind::Shi, "shi", 30, I, "d,s,i"),
    op(Kind::Roti, "roti", 31, I, "d,s,i"),
    op(Kind::Tex, "tex", 32, I, "d,s,pos,len"),
    op(Kind::Tins, "tins", 33, I, "d,s,pos,len"),
    // 7.4 trit counting
    op(Kind::Lzt, "lzt", 16, R, "d,s"),
    op(Kind::Tzt, "tzt", 17, R, "d,s"),
    op(Kind::Nzt, "nzt", 18, R, "d,s"),
    // 7.5 tritwise logic
    op(Kind::Tlut, "tlut", 19, R, "d,s,t,x"),
    // 7.6 binary operations
    op(Kind::Band, "band", 20, R, "d,s,t"),
    op(Kind::Bor, "bor", 21, R, "d,s,t"),
    op(Kind::Bxor, "bxor", 22, R, "d,s,t"),
    op(Kind::Bsh, "bsh", 23, R, "d,s,t"),
    // 7.7 immediates and pc-relative
    op(Kind::Addi, "addi", 34, I, "d,s,i"),
    op(Kind::Cmpi, "cmpi", 35, I, "d,s,i"),
    op(Kind::Lui, "lui", 36, U, "d,i"),
    op(Kind::Auipc, "auipc", 37, U, "d,i"),
    // 7.8 loads and stores
    op(Kind::Lt, "lt", -1, I, "d,m"),
    op(Kind::Lw, "lw", -2, I, "d,m"),
    op(Kind::St, "st", -3, S, "t,m"),
    op(Kind::Sw, "sw", -4, S, "t,m"),
    // 7.9 atomics
    op(Kind::Lr, "lr", -20, R, "d,a"),
    op(Kind::Sc, "sc", -21, R, "d,t,a"),
    op(Kind::Amoswap, "amoswap", -22, R, "d,t,a"),
    op(Kind::Amoadd, "amoadd", -23, R, "d,t,a"),
    op(Kind::Amomin, "amomin", -24, R, "d,t,a"),
    op(Kind::Amomax, "amomax", -25, R, "d,t,a"),
    op(Kind::Amotlut, "amotlut", -26, R, "d,t,a,x"),
    op(Kind::Cas, "cas", -27, R, "d,t,a"),
    op(Kind::Wrs, "wrs", -28, I, "s"),
    // 7.10 control transfer
    op(Kind::Jal, "jal", -10, J, "d,L"),
    op(Kind::Jalr, "jalr", -11, I, "d,m"),
    op(Kind::B, "b", -12, FB, "s,t,L"),
    op(Kind::Br3, "br3", -13, T, "s,N,P"),
    // 7.12 system
    op(Kind::Sys, "sys", -30, I, ""),
    op(Kind::Tret, "tret", -31, I, ""),
    op(Kind::Csrrw, "csrrw", -32, I, "d,c,s"),
    op(Kind::Csrrtmax, "csrrtmax", -33, I, "d,c,s"),
    op(Kind::Csrrtmin, "csrrtmin", -34, I, "d,c,s"),
    op(Kind::Fence, "fence", -35, I, "F"),
    op(Kind::FenceI, "fence.i", -36, I, ""),
    op(Kind::FenceVm, "fence.vm", -37, R, "s,t"),
    op(Kind::Wfi, "wfi", -38, I, ""),
    op(Kind::Brk, "brk", -39, I, ""),
    op(Kind::Cclean, "cclean", -40, R, "s"),
    op(Kind::Cflush, "cflush", -41, R, "s"),
    op(Kind::Cinval, "cinval", -42, R, "s"),
];

/// The opcode of a compressed bundle (6.1): recognised, not yet run.
pub const BUNDLE_OPCODE: i64 = 121;

pub fn by_opcode(opcode: i64) -> Option<&'static Op> {
    OPS.iter().find(|op| op.opcode == opcode)
}

// --- fields: (lowest trit, width) --------------------------------------------

type Field = (u32, u32);

pub const OP: Field = (0, 5);
const RD: Field = (5, 3);
const RS1: Field = (8, 3);
const RS2: Field = (11, 3);
const RS3: Field = (14, 3);
const FN: Field = (17, 10);
const IMM_I: Field = (11, 16);
const COND: Field = (5, 2);
const HINT_B: Field = (7, 1);
const OFF_B: Field = (14, 13);
const HINT_T: Field = (5, 1);
const ZERO_T: Field = (6, 2);
const OFFN: Field = (11, 8);
const OFFP: Field = (19, 8);
const IMM_U: Field = (8, 19);

fn get(word: i64, (lo, width): Field) -> i64 {
    field(word, lo, width)
}

// --- branch conditions --------------------------------------------------------

/// Whether a branch with this cond is taken when cmp(rs1, rs2) is s;
/// None for a reserved cond.
pub fn cond_taken(cond: i64, s: i64) -> Option<bool> {
    match cond {
        0 => Some(s == 0),
        3 => Some(s != 0),
        1 => Some(s == 1),
        -1 => Some(s == -1),
        2 => Some(s >= 0),
        -2 => Some(s <= 0),
        _ => None,
    }
}

// --- registers ---------------------------------------------------------------

/// A register's number from any of its names: `r-13`..`r13`, `zero`,
/// `a0`..`a5`, `t0`..`t6`, `s0`..`s8`, `fp`, `gp`, `tp`, `ra`, `sp`.
pub fn reg_number(name: &str) -> Option<i64> {
    // Exact spellings only: "r1" but not "r01" or "r+1".
    let indexed = |prefix: &str, range: std::ops::RangeInclusive<i64>| -> Option<i64> {
        let n: i64 = name.strip_prefix(prefix)?.parse().ok()?;
        (range.contains(&n) && name == format!("{prefix}{n}")).then_some(n)
    };
    match name {
        "zero" => Some(0),
        "fp" => Some(-1),
        "gp" => Some(-10),
        "tp" => Some(-11),
        "ra" => Some(-12),
        "sp" => Some(-13),
        _ => indexed("r", -13..=13)
            .or_else(|| indexed("a", 0..=5).map(|n| 1 + n))
            .or_else(|| indexed("t", 0..=6).map(|n| 7 + n))
            .or_else(|| indexed("s", 0..=8).map(|n| -1 - n)),
    }
}

/// A register's ABI name, as the disassembler prints it; None for the
/// registers that have only their number (r-1 prints as s0).
pub fn abi_name(r: i64) -> Option<String> {
    Some(match r {
        0 => "zero".into(),
        1..=6 => format!("a{}", r - 1),
        7..=13 => format!("t{}", r - 7),
        -9..=-1 => format!("s{}", -1 - r),
        -10 => "gp".into(),
        -11 => "tp".into(),
        -12 => "ra".into(),
        -13 => "sp".into(),
        _ => return None,
    })
}

/// Branch conditions by name, in the order the assembler lists them.
pub const CONDS: [(&str, i64); 6] =
    [("eq", 0), ("ne", 3), ("gt", 1), ("lt", -1), ("ge", 2), ("le", -2)];

pub fn cond_name(cond: i64) -> Option<&'static str> {
    CONDS.iter().find(|(_, c)| *c == cond).map(|(n, _)| *n)
}

// --- tritwise truth tables (7.5) ---------------------------------------------

/// A two-input trit function as the 9-trit `tlut` table.
pub fn make_table(f: impl Fn(i64, i64) -> i64) -> i64 {
    (0..9).map(|k| f(k / 3 - 1, k % 3 - 1) * crate::trits::pow3(k as u32)).sum()
}

/// The named tables, in the assembler's order.
pub fn tables() -> [(&'static str, i64); 7] {
    [
        ("tmin", make_table(i64::min)),
        ("tmax", make_table(i64::max)),
        ("tmul", make_table(|a, b| a * b)),
        ("tsum", make_table(|a, b| low(a + b, 1))),
        ("tcons", make_table(|a, b| if a == b { a } else { 0 })),
        ("tany", make_table(|a, b| (a + b).signum())),
        ("tabs", make_table(|a, _| a.abs())),
    ]
}

// --- encoding -----------------------------------------------------------------

/// The fields a format has, by name: (name, lowest trit, width).
pub fn format_fields(format: Format) -> &'static [(&'static str, u32, u32)] {
    match format {
        R => &[("rd", 5, 3), ("rs1", 8, 3), ("rs2", 11, 3), ("rs3", 14, 3), ("fn", 17, 10)],
        I => &[("rd", 5, 3), ("rs1", 8, 3), ("imm", 11, 16)],
        S => &[("rs2", 5, 3), ("rs1", 8, 3), ("imm", 11, 16)],
        FB => &[("cond", 5, 2), ("hint", 7, 1), ("rs1", 8, 3), ("rs2", 11, 3), ("off", 14, 13)],
        T => &[("hint", 5, 1), ("pad", 6, 2), ("rs1", 8, 3), ("offn", 11, 8), ("offp", 19, 8)],
        U | J => &[("rd", 5, 3), ("imm", 8, 19)],
    }
}

/// Build an instruction word from named field values, in order; the
/// first field that does not fit is the error.
pub fn encode(op: &Op, fields: &[(&str, i64)]) -> Result<i64, String> {
    let layout = format_fields(op.format);
    let mut word = op.opcode;
    for &(name, value) in fields {
        let Some(&(_, lo, width)) = layout.iter().find(|(n, _, _)| *n == name) else {
            return Err(format!("{}: format {:?} has no field {name}", op.name, op.format));
        };
        let max = crate::trits::half(width);
        if value.abs() > max {
            return Err(format!("{name} = {value} does not fit {width} trits (±{max})"));
        }
        word += value * crate::trits::pow3(lo);
    }
    Ok(word)
}

pub fn by_name(name: &str) -> Option<&'static Op> {
    OPS.iter().find(|op| op.name == name)
}

// --- trap causes (8.5) -------------------------------------------------------

pub const ILLEGAL: i64 = 1;
pub const MISALIGNED: i64 = 2;
pub const ACCESS_FAULT: i64 = 3;
pub const SYSCALL: i64 = 4;
pub const BREAKPOINT: i64 = 5;
pub const INSTR_PAGE_FAULT: i64 = 6;
pub const LOAD_PAGE_FAULT: i64 = 7;
pub const STORE_PAGE_FAULT: i64 = 8;
pub const SINGLE_STEP: i64 = 9;

// --- CSRs (8.2) --------------------------------------------------------------

/// A CSR's number: trit 8 is the lowest ring that may access it, trit 7
/// is + where it is writable.
pub const fn csr_number(ring: i64, access: i64, index: i64) -> i64 {
    6561 * ring + 2187 * access + index
}

/// Lowest ring allowed to access a CSR.
pub const fn csr_ring(number: i64) -> i64 {
    field(number, 8, 1)
}

pub const fn csr_writable(number: i64) -> bool {
    field(number, 7, 1) == 1
}

/// The trap bank each of rings +1 and 0 has, in order; the CSR names add
/// an `m` or `s` prefix.
pub const BANK: [&str; 9] =
    ["tstatus", "tvec", "tpc", "tcause", "tval", "scratch", "ie", "ip", "timecmp"];
pub const TSTATUS: usize = 0;
pub const TVEC: usize = 1;
pub const TPC: usize = 2;
pub const TCAUSE: usize = 3;
pub const TVAL: usize = 4;
pub const IE: usize = 6;
pub const IP: usize = 7;
pub const TIMECMP: usize = 8;

/// Trits of tstatus.
pub const ST_PR: u32 = 0;
pub const ST_IE: u32 = 1;
pub const ST_PIE: u32 = 2;
pub const ST_SUA: u32 = 3;
pub const ST_SS: u32 = 4;

pub const EDELEG: i64 = csr_number(1, 1, 20);
pub const IDELEG: i64 = csr_number(1, 1, 21);
pub const HARTID: i64 = csr_number(1, -1, 0);
pub const PTBR: i64 = csr_number(0, 1, 10);
pub const ENTROPY: i64 = csr_number(0, -1, 0);
pub const CYCLE: i64 = csr_number(-1, -1, 0);
pub const TIME: i64 = csr_number(-1, -1, 1);
pub const INSTRET: i64 = csr_number(-1, -1, 2);
pub const FCSR: i64 = csr_number(-1, 1, 0);

/// CSRs that are plain storage; the others are computed or live in a
/// trap bank.
pub fn stored_csrs() -> impl Iterator<Item = i64> {
    let debug = (0..9).flat_map(|i| [csr_number(0, 1, 20 + i), csr_number(0, 1, 30 + i)]);
    [EDELEG, IDELEG, PTBR, FCSR].into_iter().chain(debug)
}

/// A CSR's name from its number.
pub fn csr_name(number: i64) -> Option<String> {
    let fixed = [
        ("edeleg", EDELEG),
        ("ideleg", IDELEG),
        ("hartid", HARTID),
        ("ptbr", PTBR),
        ("entropy", ENTROPY),
        ("cycle", CYCLE),
        ("time", TIME),
        ("instret", INSTRET),
        ("fcsr", FCSR),
    ];
    if let Some((name, _)) = fixed.iter().find(|(_, n)| *n == number) {
        return Some((*name).into());
    }
    for (prefix, ring) in [('m', 1), ('s', 0)] {
        let i = number - csr_number(ring, 1, 0);
        if (0..BANK.len() as i64).contains(&i) {
            return Some(format!("{prefix}{}", BANK[i as usize]));
        }
    }
    for (prefix, base) in [("tdaddr", 20), ("tdctl", 30)] {
        let i = number - csr_number(0, 1, base);
        if (0..9).contains(&i) {
            return Some(format!("{prefix}{i}"));
        }
    }
    None
}

/// Every CSR, by name, in the assembler's order.
pub fn csrs() -> Vec<(String, i64)> {
    let mut out: Vec<(String, i64)> = Vec::new();
    for (prefix, ring) in [('m', 1), ('s', 0)] {
        for (i, b) in BANK.iter().enumerate() {
            out.push((format!("{prefix}{b}"), csr_number(ring, 1, i as i64)));
        }
    }
    for name in
        ["edeleg", "ideleg", "hartid", "ptbr", "entropy", "cycle", "time", "instret", "fcsr"]
    {
        out.push((name.into(), csr_by_name(name).unwrap()));
    }
    for i in 0..9 {
        out.push((format!("tdaddr{i}"), csr_number(0, 1, 20 + i)));
    }
    for i in 0..9 {
        out.push((format!("tdctl{i}"), csr_number(0, 1, 30 + i)));
    }
    out
}

/// A CSR's number from its name.
pub fn csr_by_name(name: &str) -> Option<i64> {
    let banked = |prefix: char, ring: i64| -> Option<i64> {
        let rest = name.strip_prefix(prefix)?;
        BANK.iter().position(|b| *b == rest).map(|i| csr_number(ring, 1, i as i64))
    };
    let debug = |prefix: &str, base: i64| -> Option<i64> {
        let i: i64 = name.strip_prefix(prefix)?.parse().ok()?;
        (0..9).contains(&i).then(|| csr_number(0, 1, base + i))
    };
    let fixed = match name {
        "edeleg" => Some(EDELEG),
        "ideleg" => Some(IDELEG),
        "hartid" => Some(HARTID),
        "ptbr" => Some(PTBR),
        "entropy" => Some(ENTROPY),
        "cycle" => Some(CYCLE),
        "time" => Some(TIME),
        "instret" => Some(INSTRET),
        "fcsr" => Some(FCSR),
        _ => None,
    };
    fixed
        .or_else(|| banked('m', 1))
        .or_else(|| banked('s', 0))
        .or_else(|| debug("tdaddr", 20))
        .or_else(|| debug("tdctl", 30))
}

/// The truth tables `csrrtmax` and `csrrtmin` combine with (7.5).
pub const TMAX: i64 = 9728;
pub const TMIN: i64 = 5792;

// --- decoding ----------------------------------------------------------------

/// One decoded instruction: the op plus every field of its format.
/// Fields the format lacks read as 0.
#[derive(Clone, Copy, Debug)]
pub struct Decoded {
    pub op: &'static Op,
    pub word: i64,
    pub rd: i64,
    pub rs1: i64,
    pub rs2: i64,
    pub rs3: i64,
    pub fn_: i64,
    pub imm: i64,
    pub cond: i64,
    pub hint: i64,
    pub pad: i64,
    pub off: i64,
    pub offn: i64,
    pub offp: i64,
}

/// Why a word is not a valid instruction.
#[derive(Debug, PartialEq, Eq)]
pub struct Illegal(pub String);

fn zero(value: i64, used: bool, name: &str) -> Result<(), Illegal> {
    if !used && value != 0 {
        return Err(Illegal(format!("{name} must be zero")));
    }
    Ok(())
}

fn fields(op: &'static Op, word: i64) -> Decoded {
    let mut d = Decoded {
        op,
        word,
        rd: 0,
        rs1: 0,
        rs2: 0,
        rs3: 0,
        fn_: 0,
        imm: 0,
        cond: 0,
        hint: 0,
        pad: 0,
        off: 0,
        offn: 0,
        offp: 0,
    };
    match op.format {
        R => {
            (d.rd, d.rs1, d.rs2) = (get(word, RD), get(word, RS1), get(word, RS2));
            (d.rs3, d.fn_) = (get(word, RS3), get(word, FN));
        }
        I => (d.rd, d.rs1, d.imm) = (get(word, RD), get(word, RS1), get(word, IMM_I)),
        S => (d.rs2, d.rs1, d.imm) = (get(word, RD), get(word, RS1), get(word, IMM_I)),
        FB => {
            (d.cond, d.hint) = (get(word, COND), get(word, HINT_B));
            (d.rs1, d.rs2, d.off) = (get(word, RS1), get(word, RS2), get(word, OFF_B));
        }
        T => {
            (d.hint, d.pad, d.rs1) = (get(word, HINT_T), get(word, ZERO_T), get(word, RS1));
            (d.offn, d.offp) = (get(word, OFFN), get(word, OFFP));
        }
        U | J => (d.rd, d.imm) = (get(word, RD), get(word, IMM_U)),
    }
    d
}

fn check_r(d: &Decoded) -> Result<(), Illegal> {
    let op = d.op;
    zero(d.rd, op.has("d"), "rd")?;
    zero(d.rs1, op.has("s") || op.has("a"), "rs1")?;
    zero(d.rs2, op.has("t"), "rs2")?;
    if op.atomic() {
        // rs3 carries the ordering: trit 0 aq, trit 1 rl, trit 2 zero.
        if field(d.rs3, 0, 1) < 0 || field(d.rs3, 1, 1) < 0 {
            return Err(Illegal("ordering trits must be 0 or +".into()));
        }
        zero(field(d.rs3, 2, 1), false, "rs3[2]")?;
    } else {
        zero(d.rs3, op.has("u"), "rs3")?;
    }
    if op.has("x") { zero(field(d.fn_, 9, 1), false, "fn[9]") } else { zero(d.fn_, false, "fn") }
}

fn check_i(d: &Decoded) -> Result<(), Illegal> {
    let op = d.op;
    zero(d.rd, op.has("d"), "rd")?;
    zero(d.rs1, op.has("s") || op.has("m"), "rs1")?;
    if op.has("c") {
        zero(d.imm - low(d.imm, 9), false, "imm above the CSR number")
    } else if op.has("pos") || op.has("F") {
        zero(d.imm - low(d.imm, 8), false, "imm above trit 7")
    } else {
        zero(d.imm, op.has("i") || op.has("m"), "imm")
    }
}

/// Decode a word, refusing any invalid encoding. Fields an instruction
/// does not use must be zero (docs/isa.md 6), so they stay free for
/// extensions.
pub fn decode(word: i64) -> Result<Decoded, Illegal> {
    let opcode = get(word, OP);
    let op = by_opcode(opcode).ok_or_else(|| Illegal(format!("opcode {opcode}")))?;
    let d = fields(op, word);
    match op.format {
        R => check_r(&d)?,
        I => check_i(&d)?,
        FB if cond_taken(d.cond, 0).is_none() => {
            return Err(Illegal(format!("reserved branch condition {}", d.cond)));
        }
        T => zero(d.pad, false, "trits 7..6")?,
        _ => {}
    }
    Ok(d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trits::pow3;

    fn r(op: i64, rd: i64, rs1: i64, rs2: i64) -> i64 {
        op + rd * pow3(5) + rs1 * pow3(8) + rs2 * pow3(11)
    }

    #[test]
    fn opcodes_are_unique() {
        for (i, a) in OPS.iter().enumerate() {
            assert!(OPS[i + 1..].iter().all(|b| b.opcode != a.opcode), "{}", a.name);
        }
    }

    #[test]
    fn decodes_register_fields() {
        let d = decode(r(1, 1, 2, -3)).unwrap();
        assert_eq!((d.op.kind, d.rd, d.rs1, d.rs2), (Kind::Add, 1, 2, -3));
    }

    #[test]
    fn unused_fields_must_be_zero() {
        assert!(decode(r(12, 1, 2, 3)).is_err()); // abs has no rs2
        assert!(decode(r(1, 1, 2, 3) + pow3(17)).is_err()); // add has no fn
        assert!(decode(-30 + pow3(5)).is_err()); // sys has no rd
    }

    #[test]
    fn atomics_check_their_ordering_trits() {
        let amoadd = r(-23, 1, 2, 3);
        assert!(decode(amoadd + pow3(14)).is_ok()); // aq
        assert!(decode(amoadd - pow3(14)).is_err());
        assert!(decode(amoadd + pow3(16)).is_err()); // trit 2
    }

    #[test]
    fn reserved_branch_conditions_are_illegal() {
        assert!(decode(-12 + 3 * pow3(5)).is_ok()); // ne
        assert!(decode(-12 + 4 * pow3(5)).is_err());
    }

    #[test]
    fn unknown_opcodes_are_illegal() {
        assert!(decode(0).is_err());
        assert!(decode(100).is_err());
    }

    #[test]
    fn tables_match_the_spec_values() {
        let t: Vec<i64> = tables().iter().map(|(_, v)| *v).collect();
        assert_eq!((t[0], t[1]), (TMIN, TMAX));
    }

    #[test]
    fn encode_checks_field_widths() {
        let add = by_name("add").unwrap();
        assert_eq!(encode(add, &[("rd", 1), ("rs1", 2), ("rs2", -3)]).unwrap(), r(1, 1, 2, -3));
        assert_eq!(encode(add, &[("rd", 14)]).unwrap_err(), "rd = 14 does not fit 3 trits (±13)");
        assert!(encode(add, &[("imm", 1)]).unwrap_err().contains("has no field imm"));
    }

    #[test]
    fn csr_names_round_trip() {
        for (name, number) in csrs() {
            assert_eq!(csr_name(number), Some(name.clone()));
            assert_eq!(csr_by_name(&name), Some(number));
        }
    }

    #[test]
    fn csr_numbers_follow_the_layout() {
        assert_eq!(csr_by_name("mtvec"), Some(6561 + 2187 + 1));
        assert_eq!(csr_by_name("stimecmp"), Some(2187 + 8));
        assert_eq!(csr_by_name("tdctl8"), Some(2187 + 38));
        assert_eq!(csr_by_name("mtnothing"), None);
        assert_eq!(csr_ring(TIME), -1);
        assert!(csr_writable(PTBR) && !csr_writable(ENTROPY));
        assert_eq!(stored_csrs().count(), 22);
    }

    #[test]
    fn register_names() {
        assert_eq!(reg_number("zero"), Some(0));
        assert_eq!(reg_number("a5"), Some(6));
        assert_eq!(reg_number("t6"), Some(13));
        assert_eq!(reg_number("s8"), Some(-9));
        assert_eq!(reg_number("r-13"), Some(-13));
        assert_eq!(reg_number("sp"), Some(-13));
        assert_eq!(reg_number("a6"), None);
        assert_eq!(reg_number("r14"), None);
        assert_eq!(reg_number("r01"), None);
        assert_eq!(reg_number("r+1"), None);
        assert_eq!(abi_name(-1).as_deref(), Some("s0"));
        assert_eq!(abi_name(13).as_deref(), Some("t6"));
    }
}
