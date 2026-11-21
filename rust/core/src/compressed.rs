//! Compressed bundles (docs/isa.md 6.1): splitting a bundle into its
//! slots, expanding a slot to the full instruction it stands for, and
//! the other way, which compressed form a full instruction fits.
//!
//! Every compressed instruction expands to exactly one full one, so the
//! hart runs a slot by expanding and decoding it like any other word,
//! and a full instruction fits a form when it is that form's expansion.
//! `--mix` counts the forms to judge the table against real code.

use crate::bus::PAGE;
use crate::isa::{self, BUNDLE_OPCODE, Decoded, Kind, by_name, encode, reg_number, tables};
use crate::trits::{field, half, low, pow3};

/// A slot: 11 trits.
pub const SLOT_TRITS: u32 = 11;
/// Where slots A and B start in a bundle word.
const SLOT_A: u32 = 5;
const SLOT_B: u32 = 16;

/// A CI immediate: 5 trits.
const CI_TRITS: u32 = 5;
/// A CJ offset: 8 trits.
const CJ_TRITS: u32 = 8;
/// A two-register branch's offset, in `f`: 2 trits.
const F_TRITS: u32 = 2;

/// Slot opcodes (the table in 6.1).
const OP_REG: i64 = 1;
const OP_MEM: i64 = 2;
const OP_JUMP: i64 = 3;
const OP_ADDI: i64 = 4;
const OP_LI: i64 = 5;
const OP_SHI: i64 = 6;
const OP_CMPI: i64 = 7;
const OP_LWSP: i64 = 8;
const OP_SWSP: i64 = 9;
const OP_BEQZ: i64 = 10;
const OP_BNEZ: i64 = 11;
const OP_J: i64 = 12;
const OP_JAL: i64 = 13;
const OP_BEQ: i64 = -1;
const OP_BNE: i64 = -2;
const OP_BLT: i64 = -3;
const OP_BLE: i64 = -4;
const OP_LIP: i64 = -5;

/// `f` of op 1 for the register forms other than `c.mv` (f 0), with the
/// full instruction each is and, for `tlut`, its table.
const REG_FORMS: [(i64, &str, &str); 8] = [
    (1, "c.add", "add"),
    (-1, "c.sub", "sub"),
    (2, "c.cmp", "cmp"),
    (-2, "c.mul", "mul"),
    (3, "c.min", "min"),
    (-3, "c.max", "max"),
    (4, "c.tmin", "tmin"),
    (-4, "c.tmax", "tmax"),
];

/// `f` of op 2: the memory forms and their full instructions.
const MEM_FORMS: [(i64, &str, &str); 4] =
    [(0, "c.lw", "lw"), (1, "c.sw", "sw"), (-1, "c.lt", "lt"), (2, "c.st", "st")];

/// `f` of op 3.
const JR: i64 = 0;
const JALR: i64 = 1;
const SYS: i64 = -1;
const TRET: i64 = 2;

/// Branch `cond` values (isa.md 7.10).
const EQ: i64 = 0;
const NE: i64 = 3;
const GT: i64 = 1;
const LT: i64 = -1;
const GE: i64 = 2;
const LE: i64 = -2;

fn ci(value: i64) -> bool {
    value.abs() <= half(CI_TRITS)
}

fn table(name: &str) -> i64 {
    tables().iter().find(|(n, _)| *n == name).map(|(_, v)| *v).expect("a named table")
}

fn reg(name: &str) -> i64 {
    reg_number(name).expect("an ABI register name")
}

// --- bundles and slots --------------------------------------------------------

/// Whether a word is a bundle.
pub fn is_bundle(word: i64) -> bool {
    low(word, 5) == BUNDLE_OPCODE
}

/// Slots A and B of a bundle word.
pub fn slots(word: i64) -> [i64; 2] {
    [field(word, SLOT_A, SLOT_TRITS), field(word, SLOT_B, SLOT_TRITS)]
}

/// The bundle holding slots A and B.
pub fn bundle(a: i64, b: i64) -> i64 {
    BUNDLE_OPCODE + a * pow3(SLOT_A) + b * pow3(SLOT_B)
}

/// A CR slot: `f`, `rs`, `rd`, op.
fn cr(op: i64, f: i64, rs: i64, rd: i64) -> i64 {
    op + rd * pow3(3) + rs * pow3(6) + f * pow3(9)
}

/// A CI slot: `imm`, `rd`, op.
fn ci_slot(op: i64, imm: i64, rd: i64) -> i64 {
    op + rd * pow3(3) + imm * pow3(6)
}

/// A CJ slot: `off`, op.
fn cj(op: i64, off: i64) -> i64 {
    op + off * pow3(3)
}

/// A slot's fields, read every way: op, rd, rs, f, the CI immediate and
/// the CJ offset.
struct Fields {
    op: i64,
    rd: i64,
    rs: i64,
    f: i64,
    imm: i64,
    off: i64,
}

fn fields(slot: i64) -> Fields {
    Fields {
        op: low(slot, 3),
        rd: field(slot, 3, 3),
        rs: field(slot, 6, 3),
        f: field(slot, 9, F_TRITS),
        imm: field(slot, 6, CI_TRITS),
        off: field(slot, 3, CJ_TRITS),
    }
}

fn full(name: &str, fields: &[(&str, i64)]) -> i64 {
    encode(by_name(name).expect("a full instruction"), fields).expect("an expansion fits")
}

/// The full instruction a slot expands to, or None for an illegal slot:
/// op 0, a reserved op or `f`, or a register field that must be r0 and
/// is not.
pub fn expand(slot: i64) -> Option<i64> {
    let Fields { op, rd, rs, f, imm, off } = fields(slot);
    let sp = reg("sp");
    let branch = |cond, rs1, rs2, off| {
        full("b", &[("cond", cond), ("rs1", rs1), ("rs2", rs2), ("off", off)])
    };
    Some(match op {
        OP_REG if f == 0 => full("addi", &[("rd", rd), ("rs1", rs)]),
        OP_REG => {
            let &(_, _, name) = REG_FORMS.iter().find(|(v, _, _)| *v == f)?;
            match name {
                "tmin" | "tmax" => {
                    full("tlut", &[("rd", rd), ("rs1", rd), ("rs2", rs), ("fn", table(name))])
                }
                _ => full(name, &[("rd", rd), ("rs1", rd), ("rs2", rs)]),
            }
        }
        OP_MEM => {
            let &(_, _, name) = MEM_FORMS.iter().find(|(v, _, _)| *v == f)?;
            match name {
                "sw" | "st" => full(name, &[("rs2", rd), ("rs1", rs)]),
                _ => full(name, &[("rd", rd), ("rs1", rs)]),
            }
        }
        OP_JUMP if rd != 0 => return None,
        OP_JUMP => match f {
            JR => full("jalr", &[("rs1", rs)]),
            JALR => full("jalr", &[("rd", reg("ra")), ("rs1", rs)]),
            SYS if rs == 0 => full("sys", &[]),
            TRET if rs == 0 => full("tret", &[]),
            _ => return None,
        },
        OP_ADDI => full("addi", &[("rd", rd), ("rs1", rd), ("imm", imm)]),
        OP_LI => full("addi", &[("rd", rd), ("imm", imm)]),
        OP_SHI => full("shi", &[("rd", rd), ("rs1", rd), ("imm", imm)]),
        OP_CMPI => full("cmpi", &[("rd", rd), ("rs1", rd), ("imm", imm)]),
        OP_LWSP => full("lw", &[("rd", rd), ("rs1", sp), ("imm", 3 * imm)]),
        OP_SWSP => full("sw", &[("rs2", rd), ("rs1", sp), ("imm", 3 * imm)]),
        OP_BEQZ => branch(EQ, rd, 0, imm),
        OP_BNEZ => branch(NE, rd, 0, imm),
        OP_J => full("jal", &[("imm", off)]),
        OP_JAL => full("jal", &[("rd", reg("ra")), ("imm", off)]),
        OP_BEQ => branch(EQ, rd, rs, f),
        OP_BNE => branch(NE, rd, rs, f),
        OP_BLT => branch(LT, rd, rs, f),
        OP_BLE => branch(LE, rd, rs, f),
        OP_LIP => full("addi", &[("rd", rd), ("imm", imm * PAGE)]),
        _ => return None,
    })
}

/// A legal slot's mnemonic.
pub fn mnemonic(slot: i64) -> Option<&'static str> {
    expand(slot)?;
    let Fields { op, rd, f, imm, .. } = fields(slot);
    let pick = |forms: &[(i64, &'static str, &str)]| {
        forms.iter().find(|(v, _, _)| *v == f).map(|(_, m, _)| *m)
    };
    Some(match op {
        OP_REG if f == 0 => "c.mv",
        OP_REG => return pick(&REG_FORMS),
        OP_MEM => return pick(&MEM_FORMS),
        OP_JUMP => match f {
            JR => "c.jr",
            JALR => "c.jalr",
            SYS => "c.sys",
            _ => "c.tret",
        },
        OP_ADDI if rd == 0 && imm == 0 => "c.nop",
        OP_ADDI => "c.addi",
        OP_LI => "c.li",
        OP_SHI => "c.shi",
        OP_CMPI => "c.cmpi",
        OP_LWSP => "c.lwsp",
        OP_SWSP => "c.swsp",
        OP_BEQZ => "c.beqz",
        OP_BNEZ => "c.bnez",
        OP_J => "c.j",
        OP_JAL => "c.jal",
        OP_BEQ => "c.beq",
        OP_BNE => "c.bne",
        OP_BLT => "c.blt",
        OP_BLE => "c.ble",
        _ => "c.lip",
    })
}

// --- from the full side -----------------------------------------------------

/// The slot `d` can be written as, if any.
pub fn compress(d: &Decoded) -> Option<i64> {
    let two = d.rd == d.rs1;
    let (sp, ra) = (reg("sp"), reg("ra"));
    let reg_form = |name: &str| REG_FORMS.iter().find(|(_, _, n)| *n == name).map(|(f, _, _)| *f);
    let mem_form = |name: &str| MEM_FORMS.iter().find(|(_, _, n)| *n == name).map(|(f, _, _)| *f);
    Some(match d.op.kind {
        Kind::Addi if d.rd == 0 && d.rs1 == 0 && d.imm == 0 => ci_slot(OP_ADDI, 0, 0),
        Kind::Addi if d.rs1 == 0 && ci(d.imm) => ci_slot(OP_LI, d.imm, d.rd),
        Kind::Addi if d.rs1 == 0 && d.imm % PAGE == 0 && ci(d.imm / PAGE) => {
            ci_slot(OP_LIP, d.imm / PAGE, d.rd)
        }
        Kind::Addi if d.imm == 0 => cr(OP_REG, 0, d.rs1, d.rd),
        Kind::Addi if two && ci(d.imm) => ci_slot(OP_ADDI, d.imm, d.rd),
        Kind::Add | Kind::Sub | Kind::Cmp | Kind::Mul | Kind::Min | Kind::Max if two => {
            cr(OP_REG, reg_form(d.op.name)?, d.rs2, d.rd)
        }
        Kind::Tlut if two && d.fn_ == isa::TMIN => cr(OP_REG, reg_form("tmin")?, d.rs2, d.rd),
        Kind::Tlut if two && d.fn_ == isa::TMAX => cr(OP_REG, reg_form("tmax")?, d.rs2, d.rd),
        Kind::Lw if d.rs1 == sp && d.imm % 3 == 0 && ci(d.imm / 3) => {
            ci_slot(OP_LWSP, d.imm / 3, d.rd)
        }
        Kind::Sw if d.rs1 == sp && d.imm % 3 == 0 && ci(d.imm / 3) => {
            ci_slot(OP_SWSP, d.imm / 3, d.rs2)
        }
        Kind::Lw | Kind::Lt if d.imm == 0 => cr(OP_MEM, mem_form(d.op.name)?, d.rs1, d.rd),
        Kind::Sw | Kind::St if d.imm == 0 => cr(OP_MEM, mem_form(d.op.name)?, d.rs1, d.rs2),
        Kind::Jalr if d.rd == 0 && d.imm == 0 => cr(OP_JUMP, JR, d.rs1, 0),
        Kind::Jalr if d.rd == ra && d.imm == 0 => cr(OP_JUMP, JALR, d.rs1, 0),
        Kind::Sys => cr(OP_JUMP, SYS, 0, 0),
        Kind::Tret => cr(OP_JUMP, TRET, 0, 0),
        Kind::Shi if two && ci(d.imm) => ci_slot(OP_SHI, d.imm, d.rd),
        Kind::Cmpi if two && ci(d.imm) => ci_slot(OP_CMPI, d.imm, d.rd),
        // Comparing with zero is symmetric, so either side may be r0.
        Kind::B
            if d.hint == 0
                && (d.rs1 == 0 || d.rs2 == 0)
                && ci(d.off)
                && (d.cond == EQ || d.cond == NE) =>
        {
            let r = if d.rs2 == 0 { d.rs1 } else { d.rs2 };
            ci_slot(if d.cond == EQ { OP_BEQZ } else { OP_BNEZ }, d.off, r)
        }
        // bgt and bge are blt and ble with the registers swapped.
        Kind::B if d.hint == 0 && d.off.abs() <= half(F_TRITS) => match d.cond {
            EQ => cr(OP_BEQ, d.off, d.rs2, d.rs1),
            NE => cr(OP_BNE, d.off, d.rs2, d.rs1),
            LT => cr(OP_BLT, d.off, d.rs2, d.rs1),
            GT => cr(OP_BLT, d.off, d.rs1, d.rs2),
            LE => cr(OP_BLE, d.off, d.rs2, d.rs1),
            GE => cr(OP_BLE, d.off, d.rs1, d.rs2),
            _ => return None,
        },
        Kind::Jal if d.rd == 0 && d.imm.abs() <= half(CJ_TRITS) => cj(OP_J, d.imm),
        Kind::Jal if d.rd == ra && d.imm.abs() <= half(CJ_TRITS) => cj(OP_JAL, d.imm),
        _ => return None,
    })
}

/// The compressed form `d` is the expansion of, by its mnemonic.
pub fn form(d: &Decoded) -> Option<&'static str> {
    mnemonic(compress(d)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::isa::{by_name, decode, encode};

    fn word(name: &str, fields: &[(&str, i64)]) -> Option<&'static str> {
        form(&decode(encode(by_name(name).unwrap(), fields).unwrap()).unwrap())
    }

    #[test]
    fn every_legal_slot_expands_to_a_legal_word_that_compresses_back() {
        let mut legal = 0;
        for slot in -half(SLOT_TRITS)..=half(SLOT_TRITS) {
            let Some(word) = expand(slot) else { continue };
            legal += 1;
            let d = decode(word).unwrap_or_else(|e| panic!("slot {slot}: {e:?}"));
            assert!(mnemonic(slot).is_some(), "slot {slot}");
            // Some slots share an expansion (c.li r0 and c.nop, c.mv from
            // r0 and c.li 0), so compare what they expand to; an equality
            // test with r0 may come back with its registers swapped.
            let back = compress(&d).unwrap_or_else(|| panic!("slot {slot} does not compress"));
            let again = decode(expand(back).unwrap()).unwrap();
            let swapped = d.op.kind == Kind::B
                && (d.cond == EQ || d.cond == NE)
                && (again.cond, again.off) == (d.cond, d.off)
                && (again.rs1, again.rs2) == (d.rs2, d.rs1);
            assert!(again.word == word || swapped, "slot {slot}");
        }
        assert!(legal > 100_000, "{legal}");
    }

    #[test]
    fn illegal_slots() {
        assert_eq!(expand(0), None, "op 0");
        assert_eq!(expand(-6), None, "reserved op");
        assert_eq!(expand(cr(OP_MEM, -2, 1, 1)), None, "reserved f");
        assert_eq!(expand(cr(OP_JUMP, JR, 1, 1)), None, "c.jr with rd");
        assert_eq!(expand(cr(OP_JUMP, SYS, 1, 0)), None, "c.sys with rs");
        assert_eq!(expand(cr(OP_JUMP, -2, 0, 0)), None, "reserved op 3 f");
        assert_eq!(mnemonic(0), None);
    }

    #[test]
    fn bundles_split_into_their_slots() {
        let (a, b) = (ci_slot(OP_LI, -121, 13), cj(OP_J, 3280));
        let word = bundle(a, b);
        assert!(is_bundle(word));
        assert_eq!(slots(word), [a, b]);
        assert_eq!(mnemonic(ci_slot(OP_ADDI, 0, 0)), Some("c.nop"));
        assert_eq!(
            expand(ci_slot(OP_SWSP, -2, 4)),
            Some(full("sw", &[("rs2", 4), ("rs1", reg("sp")), ("imm", -6)]))
        );
    }

    #[test]
    fn immediates_fit_up_to_five_trits() {
        assert_eq!(word("addi", &[("rd", 1), ("rs1", 1), ("imm", 121)]), Some("c.addi"));
        assert_eq!(word("addi", &[("rd", 1), ("rs1", 1), ("imm", -122)]), None);
        assert_eq!(word("addi", &[("rd", 1), ("rs1", 2), ("imm", 1)]), None, "two registers");
        assert_eq!(word("addi", &[("rd", 1), ("imm", -121)]), Some("c.li"));
        assert_eq!(word("addi", &[("rd", 1), ("rs1", 2)]), Some("c.mv"));
        assert_eq!(word("addi", &[]), Some("c.nop"));
        assert_eq!(word("addi", &[("rd", 1), ("imm", -121 * PAGE)]), Some("c.lip"));
        assert_eq!(word("addi", &[("rd", 1), ("imm", 122 * PAGE)]), None);
        assert_eq!(word("addi", &[("rd", 1), ("imm", PAGE + 3)]), None, "not a page");
        assert_eq!(word("addi", &[("rd", 1), ("rs1", 1), ("imm", PAGE)]), None);
        assert_eq!(word("shi", &[("rd", 3), ("rs1", 3), ("imm", -5)]), Some("c.shi"));
        assert_eq!(word("cmpi", &[("rd", 3), ("rs1", 4), ("imm", 0)]), None);
    }

    #[test]
    fn register_ops_need_rd_to_be_rs1() {
        assert_eq!(word("add", &[("rd", 1), ("rs1", 1), ("rs2", 5)]), Some("c.add"));
        assert_eq!(word("add", &[("rd", 1), ("rs1", 5), ("rs2", 1)]), None);
        assert_eq!(word("div", &[("rd", 1), ("rs1", 1), ("rs2", 5)]), None, "no c.div");
        let tmax = table("tmax");
        assert_eq!(word("tlut", &[("rd", 2), ("rs1", 2), ("fn", tmax)]), Some("c.tmax"));
        let tmul = table("tmul");
        assert_eq!(word("tlut", &[("rd", 2), ("rs1", 2), ("fn", tmul)]), None);
    }

    #[test]
    fn memory_forms_take_offset_zero_or_sp_words() {
        let sp = reg("sp");
        assert_eq!(word("lw", &[("rd", 1), ("rs1", 2)]), Some("c.lw"));
        assert_eq!(word("lw", &[("rd", 1), ("rs1", 2), ("imm", 3)]), None);
        assert_eq!(word("lw", &[("rd", 1), ("rs1", sp), ("imm", -363)]), Some("c.lwsp"));
        assert_eq!(word("lw", &[("rd", 1), ("rs1", sp), ("imm", 366)]), None, "too far");
        assert_eq!(word("sw", &[("rs2", 1), ("rs1", sp), ("imm", 4)]), None, "not a word");
        assert_eq!(word("st", &[("rs2", 1), ("rs1", 2)]), Some("c.st"));
        assert_eq!(word("lt", &[("rd", 1), ("rs1", sp), ("imm", 3)]), None, "no c.ltsp");
    }

    #[test]
    fn control_transfers() {
        let ra = reg("ra");
        assert_eq!(word("jalr", &[("rs1", ra)]), Some("c.jr"));
        assert_eq!(word("jalr", &[("rd", ra), ("rs1", 5)]), Some("c.jalr"));
        assert_eq!(word("jalr", &[("rd", 1), ("rs1", 5)]), None);
        assert_eq!(word("jal", &[("imm", -3280)]), Some("c.j"));
        assert_eq!(word("jal", &[("imm", 3281)]), None);
        assert_eq!(word("jal", &[("rd", ra), ("imm", 3280)]), Some("c.jal"));
        assert_eq!(word("jal", &[("rd", ra), ("imm", -3281)]), None);
        assert_eq!(word("jal", &[("rd", 1), ("imm", 5)]), None, "only ra links");
        assert_eq!(word("sys", &[]), Some("c.sys"));
        assert_eq!(word("tret", &[]), Some("c.tret"));
        assert_eq!(word("b", &[("cond", NE), ("rs1", 4), ("off", 121)]), Some("c.bnez"));
        assert_eq!(word("b", &[("cond", EQ), ("rs2", 4), ("off", -1)]), Some("c.beqz"));
        assert_eq!(word("b", &[("cond", EQ), ("rs1", 4), ("off", 122)]), None);
        assert_eq!(word("b", &[("cond", EQ), ("hint", 1), ("rs1", 4)]), None, "hinted");
        let two = |cond, off| word("b", &[("cond", cond), ("rs1", 4), ("rs2", 5), ("off", off)]);
        assert_eq!(two(EQ, -4), Some("c.beq"));
        assert_eq!(two(NE, 4), Some("c.bne"));
        assert_eq!(two(LT, 1), Some("c.blt"));
        assert_eq!(two(GT, -2), Some("c.blt"), "swapped");
        assert_eq!(two(LE, 0), Some("c.ble"));
        assert_eq!(two(GE, 3), Some("c.ble"), "swapped");
        assert_eq!(two(EQ, 5), None, "beyond f");
        assert_eq!(word("b", &[("cond", 1), ("rs1", 4), ("off", 5)]), None, "no c.bgtz");
        let hinted = word("b", &[("cond", LT), ("hint", -1), ("rs1", 4), ("rs2", 5)]);
        assert_eq!(hinted, None);
    }
}
