//! Compressed instructions written by hand (docs/isa.md 6.1): the full
//! instruction each `c.` mnemonic stands for. A `c.` instruction takes
//! the operands of its full instruction, less the ones its form fixes:
//! `c.addi rd, imm` is `addi rd, rd, imm`, `c.lwsp rd, off` is
//! `lw rd, off(sp)`, `c.lip rd, value` is `addi rd, zero, value`.
//! Whether the result fits the form is decided from the encoded word.

use crate::operands::{OperandError, fail};
use crate::text::strip;

/// The `c.` mnemonics and the operands each takes.
const ARITY: [(&str, usize); 33] = [
    ("c.nop", 0),
    ("c.mv", 2),
    ("c.add", 2),
    ("c.sub", 2),
    ("c.cmp", 2),
    ("c.mul", 2),
    ("c.min", 2),
    ("c.max", 2),
    ("c.tmin", 2),
    ("c.tmax", 2),
    ("c.lw", 2),
    ("c.sw", 2),
    ("c.lt", 2),
    ("c.st", 2),
    ("c.jr", 1),
    ("c.jalr", 1),
    ("c.sys", 0),
    ("c.tret", 0),
    ("c.addi", 2),
    ("c.li", 2),
    ("c.shi", 2),
    ("c.cmpi", 2),
    ("c.lwsp", 2),
    ("c.swsp", 2),
    ("c.beqz", 2),
    ("c.bnez", 2),
    ("c.j", 1),
    ("c.jal", 1),
    ("c.beq", 3),
    ("c.bne", 3),
    ("c.blt", 3),
    ("c.ble", 3),
    ("c.lip", 2),
];

fn arity(name: &str) -> Option<usize> {
    ARITY.iter().find(|(n, _)| *n == name).map(|(_, n)| *n)
}

pub fn is_slot(name: &str) -> bool {
    arity(name).is_some()
}

/// The full instruction, mnemonic and operands, that `name` with these
/// operands stands for. The mnemonic may be a pseudo-instruction
/// (`beq`) that is one word.
pub fn full(name: &str, texts: &[String]) -> Result<(&'static str, Vec<String>), OperandError> {
    let n = arity(name).expect("a c. mnemonic");
    if texts.len() != n {
        return fail(format!("{name} takes {n} operand(s), got {}", texts.len()));
    }
    let t = |i: usize| strip(&texts[i]).to_string();
    let s = |x: &str| x.to_string();
    Ok(match name {
        "c.nop" => ("addi", vec![s("zero"), s("zero"), s("0")]),
        "c.mv" => ("addi", vec![t(0), t(1), s("0")]),
        "c.add" | "c.sub" | "c.cmp" | "c.mul" | "c.min" | "c.max" => {
            let op = &name[2..];
            let op = ["add", "sub", "cmp", "mul", "min", "max"].into_iter().find(|o| *o == op);
            (op.expect("listed"), vec![t(0), t(0), t(1)])
        }
        "c.tmin" => ("tlut", vec![t(0), t(0), t(1), s("tmin")]),
        "c.tmax" => ("tlut", vec![t(0), t(0), t(1), s("tmax")]),
        "c.lw" => ("lw", vec![t(0), t(1)]),
        "c.sw" => ("sw", vec![t(0), t(1)]),
        "c.lt" => ("lt", vec![t(0), t(1)]),
        "c.st" => ("st", vec![t(0), t(1)]),
        "c.jr" => ("jalr", vec![s("zero"), format!("0({})", t(0))]),
        "c.jalr" => ("jalr", vec![s("ra"), format!("0({})", t(0))]),
        "c.sys" => ("sys", vec![]),
        "c.tret" => ("tret", vec![]),
        "c.addi" => ("addi", vec![t(0), t(0), t(1)]),
        "c.li" | "c.lip" => ("addi", vec![t(0), s("zero"), t(1)]),
        "c.shi" => ("shi", vec![t(0), t(0), t(1)]),
        "c.cmpi" => ("cmpi", vec![t(0), t(0), t(1)]),
        "c.lwsp" => ("lw", vec![t(0), format!("({})(sp)", t(1))]),
        "c.swsp" => ("sw", vec![t(0), format!("({})(sp)", t(1))]),
        "c.beqz" => ("beq", vec![t(0), s("zero"), t(1)]),
        "c.bnez" => ("bne", vec![t(0), s("zero"), t(1)]),
        "c.j" => ("jal", vec![s("zero"), t(0)]),
        "c.jal" => ("jal", vec![s("ra"), t(0)]),
        "c.beq" => ("beq", vec![t(0), t(1), t(2)]),
        "c.bne" => ("bne", vec![t(0), t(1), t(2)]),
        "c.blt" => ("blt", vec![t(0), t(1), t(2)]),
        "c.ble" => ("ble", vec![t(0), t(1), t(2)]),
        _ => unreachable!("every c. mnemonic is listed"),
    })
}

#[cfg(test)]
mod tests {
    use detrytus_core::compressed::{expand, slots};
    use detrytus_core::isa::{Kind, decode};
    use detrytus_core::trits::half;

    use crate::assembler::assemble;
    use crate::disasm::disassemble_slot;

    #[test]
    fn every_slot_reassembles_from_its_disassembly() {
        let bound = half(detrytus_core::compressed::SLOT_TRITS);
        for slot in -bound..=bound {
            let Some(word) = expand(slot) else { continue };
            let text = disassemble_slot(slot, Some(0));
            let image = assemble(&text, None).unwrap_or_else(|e| panic!("{text}: {e:?}"));
            let bundle = (-1..=1).map(|i| image.trytes[&i] * 19683_i64.pow((i + 1) as u32)).sum();
            let again = expand(slots(bundle)[0]).unwrap();
            // An equality test with r0 may come back with its registers
            // swapped (c.beq a0, zero is c.beqz a0).
            let (d, e) = (decode(word).unwrap(), decode(again).unwrap());
            let swapped = d.op.kind == Kind::B
                && (d.rs1, d.rs2, d.cond, d.off) == (e.rs2, e.rs1, e.cond, e.off);
            assert!(again == word || swapped, "{text}");
        }
    }
}
