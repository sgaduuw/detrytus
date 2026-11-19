//! Disassembler: one instruction word back to assembly text. Mirrors
//! `detrytus/disasm.py`.

use detrytus_core::isa::{self, Decoded, Format};
use detrytus_core::trits::{field, low};

const FENCE: &str = "rwio";
const FENCE_ALL: i64 = 40;

pub fn reg(r: i64) -> String {
    isa::abi_name(r).unwrap_or_else(|| format!("r{r}"))
}

fn target(pc: Option<i64>, off: i64) -> String {
    match pc {
        Some(pc) => (pc + 3 * off).to_string(),
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

/// Render a word; pc, when given, turns branch offsets into addresses.
pub fn disassemble(word: i64, pc: Option<i64>) -> String {
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
