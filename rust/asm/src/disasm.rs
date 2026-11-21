//! Disassembler: one instruction word back to assembly text, ported
//! from the original Python tools.

use detrytus_core::compressed;
use detrytus_core::isa::{self, Decoded, Format};
use detrytus_core::trits::{field, low};

const FENCE: &str = "rwio";
const FENCE_ALL: i64 = 40;

pub fn reg(r: i64) -> String {
    isa::abi_name(r).unwrap_or_else(|| format!("r{r}"))
}

/// A branch target. Offsets count from the bundle's address when pc is
/// slot B of one (isa.md 6.1), as a trace of an expanded slot gives it.
fn target(pc: Option<i64>, off: i64) -> String {
    match pc {
        Some(pc) => (pc - low(pc, 1) + 3 * off).to_string(),
        None => format!(".{:+}", 3 * off),
    }
}

fn table(fn_: i64) -> String {
    isa::tables()
        .iter()
        .find(|(_, v)| *v == fn_)
        .map_or_else(|| fn_.to_string(), |(n, _)| n.to_string())
}

fn fence_set(trits: i64) -> String {
    FENCE
        .chars()
        .enumerate()
        .filter(|(i, _)| field(trits, *i as u32, 1) == 1)
        .map(|(_, c)| c)
        .collect()
}

fn render(letter: &str, d: &Decoded, pc: Option<i64>) -> Vec<String> {
    match letter {
        "d" => vec![reg(d.rd)],
        "s" => vec![reg(d.rs1)],
        "t" => vec![reg(d.rs2)],
        "u" => vec![reg(d.rs3)],
        "i" => vec![d.imm.to_string()],
        "m" => vec![format!("{}({})", d.imm, reg(d.rs1))],
        "a" => vec![format!("({})", reg(d.rs1))],
        "L" => vec![target(pc, if d.op.format == Format::B { d.off } else { d.imm })],
        "N" => vec![target(pc, d.offn)],
        "P" => vec![target(pc, d.offp)],
        "x" => vec![table(d.fn_)],
        "c" => {
            let number = low(d.imm, 9);
            vec![isa::csr_name(number).unwrap_or_else(|| number.to_string())]
        }
        "pos" => vec![low(d.imm, 4).to_string()],
        "len" => vec![field(d.imm, 4, 4).to_string()],
        "F" => {
            let (pred, succ) = (low(d.imm, 4), field(d.imm, 4, 4));
            if (pred, succ) == (FENCE_ALL, FENCE_ALL) {
                vec![]
            } else {
                vec![fence_set(pred), fence_set(succ)]
            }
        }
        other => unreachable!("operand letter {other}"),
    }
}

fn mnemonic(d: &Decoded) -> String {
    let op = d.op;
    match op.format {
        Format::B => {
            let hint = match d.hint {
                1 => ".likely",
                -1 => ".unlikely",
                _ => "",
            };
            format!("b{}{hint}", isa::cond_name(d.cond).unwrap_or("?"))
        }
        Format::T => {
            let hint = match d.hint {
                -1 => ".n",
                0 => ".z",
                _ => ".p",
            };
            format!("{}{hint}", op.name)
        }
        _ if op.atomic() => {
            let order = match d.rs3 {
                0 => String::new(),
                1 => ".aq".into(),
                3 => ".rl".into(),
                4 => ".aqrl".into(),
                other => format!(".?{other}"),
            };
            format!("{}{order}", op.name)
        }
        _ => op.name.to_string(),
    }
}

/// Render one slot of a bundle in the syntax the assembler reads
/// (slots.rs): the operands of the full instruction it expands to, less
/// the ones its form fixes. pc, when given, is the bundle's address.
pub fn disassemble_slot(slot: i64, pc: Option<i64>) -> String {
    let (Some(name), Some(word)) = (compressed::mnemonic(slot), compressed::expand(slot)) else {
        return format!("c.? {slot}  # illegal slot");
    };
    let d = isa::decode(word).expect("a legal slot expands to a legal word");
    let off = if d.op.format == Format::B { d.off } else { d.imm };
    let parts = match name {
        "c.nop" | "c.sys" | "c.tret" => vec![],
        "c.mv" => vec![reg(d.rd), reg(d.rs1)],
        "c.lw" | "c.lt" => vec![reg(d.rd), format!("({})", reg(d.rs1))],
        "c.sw" | "c.st" => vec![reg(d.rs2), format!("({})", reg(d.rs1))],
        "c.jr" | "c.jalr" => vec![reg(d.rs1)],
        "c.addi" | "c.li" | "c.lip" | "c.shi" | "c.cmpi" | "c.lwsp" => {
            vec![reg(d.rd), d.imm.to_string()]
        }
        "c.swsp" => vec![reg(d.rs2), d.imm.to_string()],
        "c.beqz" | "c.bnez" => vec![reg(d.rs1), target(pc, off)],
        "c.j" | "c.jal" => vec![target(pc, off)],
        "c.beq" | "c.bne" | "c.blt" | "c.ble" => vec![reg(d.rs1), reg(d.rs2), target(pc, off)],
        // The register forms: rd = rd op rs.
        _ => vec![reg(d.rd), reg(d.rs2)],
    };
    if parts.is_empty() { name.to_string() } else { format!("{name} {}", parts.join(", ")) }
}

/// Render a word; pc, when given, turns branch offsets into addresses.
/// A bundle shows both slots, `slot A | slot B`.
pub fn disassemble(word: i64, pc: Option<i64>) -> String {
    if compressed::is_bundle(word) {
        let [a, b] = compressed::slots(word);
        return format!("{} | {}", disassemble_slot(a, pc), disassemble_slot(b, pc));
    }
    let d = match isa::decode(word) {
        Ok(d) => d,
        Err(e) => return format!(".word {word}  # illegal: {}", e.0),
    };
    let letters: Vec<&str> =
        if d.op.syntax.is_empty() { vec![] } else { d.op.syntax.split(',').collect() };
    let parts: Vec<String> = letters.iter().flat_map(|l| render(l, &d, pc)).collect();
    let name = mnemonic(&d);
    if parts.is_empty() { name } else { format!("{name} {}", parts.join(", ")) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branch_targets_count_from_the_bundle_in_slot_b() {
        let jal = isa::encode(isa::by_name("jal").unwrap(), &[("imm", 2)]).unwrap();
        assert_eq!(disassemble(jal, Some(9)), "jal zero, 15");
        assert_eq!(disassemble(jal, Some(10)), "jal zero, 15");
    }
}
