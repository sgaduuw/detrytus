//! Compressed bundles (docs/isa.md 6.1), from the full side: which
//! compressed form, if any, a full instruction could be written as.
//!
//! Every compressed instruction expands to exactly one full one, so a
//! full instruction fits a form when it is that form's expansion. This
//! is what `--mix` counts to judge the table against real code; the
//! machine does not run bundles yet.

use crate::isa::{Decoded, Kind, reg_number, tables};
use crate::trits::half;

/// A CI immediate: 5 trits.
const CI_TRITS: u32 = 5;
/// A CJ offset: 8 trits.
const CJ_TRITS: u32 = 8;

/// The `cond` values of `beq` and `bne`.
const EQ: i64 = 0;
const NE: i64 = 3;

fn ci(value: i64) -> bool {
    value.abs() <= half(CI_TRITS)
}

fn table(name: &str) -> i64 {
    tables().iter().find(|(n, _)| *n == name).map(|(_, v)| *v).expect("a named table")
}

fn reg(name: &str) -> i64 {
    reg_number(name).expect("an ABI register name")
}

/// The compressed form `d` is the expansion of, by its mnemonic.
pub fn form(d: &Decoded) -> Option<&'static str> {
    let two = d.rd == d.rs1;
    let (sp, ra) = (reg("sp"), reg("ra"));
    Some(match d.op.kind {
        Kind::Addi if d.rd == 0 && d.rs1 == 0 && d.imm == 0 => "c.nop",
        Kind::Addi if d.rs1 == 0 && ci(d.imm) => "c.li",
        Kind::Addi if d.imm == 0 => "c.mv",
        Kind::Addi if two && ci(d.imm) => "c.addi",
        Kind::Add if two => "c.add",
        Kind::Sub if two => "c.sub",
        Kind::Cmp if two => "c.cmp",
        Kind::Mul if two => "c.mul",
        Kind::Min if two => "c.min",
        Kind::Max if two => "c.max",
        Kind::Tlut if two && d.fn_ == table("tmin") => "c.tmin",
        Kind::Tlut if two && d.fn_ == table("tmax") => "c.tmax",
        Kind::Lw if d.rs1 == sp && d.imm % 3 == 0 && ci(d.imm / 3) => "c.lwsp",
        Kind::Sw if d.rs1 == sp && d.imm % 3 == 0 && ci(d.imm / 3) => "c.swsp",
        Kind::Lw if d.imm == 0 => "c.lw",
        Kind::Sw if d.imm == 0 => "c.sw",
        Kind::Lt if d.imm == 0 => "c.lt",
        Kind::St if d.imm == 0 => "c.st",
        Kind::Jalr if d.rd == 0 && d.imm == 0 => "c.jr",
        Kind::Jalr if d.rd == ra && d.imm == 0 => "c.jalr",
        Kind::Shi if two && ci(d.imm) => "c.shi",
        Kind::Cmpi if two && ci(d.imm) => "c.cmpi",
        // Comparing with zero is symmetric, so either side may be r0.
        Kind::B if d.hint == 0 && (d.rs1 == 0 || d.rs2 == 0) && ci(d.off) => match d.cond {
            EQ => "c.beqz",
            NE => "c.bnez",
            _ => return None,
        },
        Kind::Jal if d.rd == 0 && d.imm.abs() <= half(CJ_TRITS) => "c.j",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::isa::{by_name, decode, encode};

    fn word(name: &str, fields: &[(&str, i64)]) -> Option<&'static str> {
        form(&decode(encode(by_name(name).unwrap(), fields).unwrap()).unwrap())
    }

    #[test]
    fn immediates_fit_up_to_five_trits() {
        assert_eq!(word("addi", &[("rd", 1), ("rs1", 1), ("imm", 121)]), Some("c.addi"));
        assert_eq!(word("addi", &[("rd", 1), ("rs1", 1), ("imm", -122)]), None);
        assert_eq!(word("addi", &[("rd", 1), ("rs1", 2), ("imm", 1)]), None, "two registers");
        assert_eq!(word("addi", &[("rd", 1), ("imm", -121)]), Some("c.li"));
        assert_eq!(word("addi", &[("rd", 1), ("rs1", 2)]), Some("c.mv"));
        assert_eq!(word("addi", &[]), Some("c.nop"));
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
        assert_eq!(word("jal", &[("rd", ra), ("imm", 5)]), None, "no c.jal");
        assert_eq!(word("b", &[("cond", NE), ("rs1", 4), ("off", 121)]), Some("c.bnez"));
        assert_eq!(word("b", &[("cond", EQ), ("rs2", 4), ("off", -1)]), Some("c.beqz"));
        assert_eq!(word("b", &[("cond", EQ), ("rs1", 4), ("off", 122)]), None);
        assert_eq!(word("b", &[("cond", 1), ("rs1", 4)]), None, "no c.bgtz");
        assert_eq!(word("b", &[("cond", EQ), ("hint", 1), ("rs1", 4)]), None, "hinted");
        assert_eq!(word("b", &[("cond", EQ), ("rs1", 4), ("rs2", 5)]), None);
    }
}
