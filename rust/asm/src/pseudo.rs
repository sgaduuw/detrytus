//! Pseudo-instructions: each expands to one or two machine
//! instructions. Ported from the original Python tools.

use detrytus_core::isa;
use detrytus_core::trits::{WORD_MAX, half, low, pow3, split_high};

use crate::expr::{ExprError, Symbols, Value, evaluate};
use crate::operands::{Fields, OperandError, Operands, fail};

/// (machine mnemonic, operand texts, fields set directly)
pub type Expansion = (&'static str, Vec<String>, Fields);

const LOGIC: [&str; 6] = ["tmin", "tmax", "tmul", "tsum", "tcons", "tany"];
const BRANCH_ZERO: [(&str, &str); 6] = [
    ("beqz", "eq"),
    ("bnez", "ne"),
    ("bltz", "lt"),
    ("bgtz", "gt"),
    ("blez", "le"),
    ("bgez", "ge"),
];
/// What one addi can load.
const SHORT_IMMEDIATE: Value = half(16) as Value;
/// lui and auipc supply everything above the low 8 trits.
const LOW_TRITS: u32 = 8;

/// (name, machine mnemonic, template, arity): operands slotted into a
/// template, "{0}" the first, "{1}" the second.
const FIXED: [(&str, &str, &[&str], usize); 11] = [
    ("nop", "addi", &["zero", "zero", "0"], 0),
    ("mv", "addi", &["{0}", "{1}", "0"], 2),
    ("neg", "sub", &["{0}", "zero", "{1}"], 2),
    ("not", "sub", &["{0}", "zero", "{1}"], 2),
    ("sgn", "cmp", &["{0}", "{1}", "zero"], 2),
    ("j", "jal", &["zero", "{0}"], 1),
    ("call", "jal", &["ra", "{0}"], 1),
    ("ret", "jalr", &["zero", "0(ra)"], 0),
    ("csrr", "csrrw", &["{0}", "{1}", "zero"], 2),
    ("csrw", "csrrw", &["zero", "{0}", "{1}"], 2),
    ("tabs", "tlut", &["{0}", "{1}", "zero", "tabs"], 2),
];

pub fn is_pseudo(name: &str) -> bool {
    FIXED.iter().any(|f| f.0 == name)
        || matches!(name, "tmap" | "li" | "la")
        || LOGIC.contains(&name)
        || name.strip_prefix('b').is_some_and(|c| isa::CONDS.iter().any(|(n, _)| *n == c))
        || BRANCH_ZERO.iter().any(|(n, _)| *n == name)
}

fn arity(name: &str, texts: &[String], n: usize) -> Result<(), OperandError> {
    if texts.len() != n {
        return fail(format!("{name} takes {n} operand(s), got {}", texts.len()));
    }
    Ok(())
}

/// value = hi·3⁸ + lo with lo balanced: no carry fix-up needed.
fn split_constant(value: Value) -> (Value, Value) {
    let m = pow3(LOW_TRITS) as Value;
    let h = (m - 1) / 2;
    let lo = (value + h).rem_euclid(m) - h;
    ((value - lo) / m, lo)
}

fn cond(name: &str) -> Fields {
    let mut f = Fields::default();
    let c = isa::CONDS.iter().find(|(n, _)| *n == name).unwrap().1;
    f.set("cond", c as Value);
    f
}

/// The machine instructions a pseudo stands for.
pub fn expand(
    ops: &Operands,
    name: &str,
    texts: &[String],
    pc: Value,
    size: i128,
) -> Result<Vec<Expansion>, OperandError> {
    if let Some(&(_, op, template, n)) = FIXED.iter().find(|f| f.0 == name) {
        arity(name, texts, n)?;
        let filled = template
            .iter()
            .map(|t| match *t {
                "{0}" => texts[0].clone(),
                "{1}" => texts[1].clone(),
                other => other.to_string(),
            })
            .collect();
        return Ok(vec![(op, filled, Fields::default())]);
    }
    if LOGIC.contains(&name) {
        arity(name, texts, 3)?;
        let mut t = texts.to_vec();
        t.push(name.to_string());
        return Ok(vec![("tlut", t, Fields::default())]);
    }
    if let Some((_, c)) = BRANCH_ZERO.iter().find(|(n, _)| *n == name) {
        arity(name, texts, 2)?;
        return Ok(vec![("b", vec![texts[0].clone(), "zero".into(), texts[1].clone()], cond(c))]);
    }
    match name {
        "li" => li(ops, texts, pc, size),
        "la" => {
            arity("la", texts, 2)?;
            let rd = &texts[0];
            let (hi, lo) = split_constant(ops.value(&texts[1], pc)? - pc);
            Ok(vec![
                ("auipc", vec![rd.clone(), hi.to_string()], Fields::default()),
                ("addi", vec![rd.clone(), rd.clone(), lo.to_string()], Fields::default()),
            ])
        }
        "tmap" => {
            arity("tmap", texts, 3)?;
            let f = ops.value(&texts[2], pc)?;
            // Only the low three trits count: reduce to them, exactly.
            let f = ((f + 13).rem_euclid(27) - 13) as i64;
            let outputs: Vec<i64> = (0..3).map(|i| low(split_high(f, i), 1)).collect();
            let table = isa::make_table(|a, _| outputs[(a + 1) as usize]);
            let t = vec![texts[0].clone(), texts[1].clone(), "zero".into(), table.to_string()];
            Ok(vec![("tlut", t, Fields::default())])
        }
        // b<cond>: the operands go straight to b, which checks them.
        _ => Ok(vec![("b", texts.to_vec(), cond(&name[1..]))]),
    }
}

fn li(
    ops: &Operands,
    texts: &[String],
    pc: Value,
    size: i128,
) -> Result<Vec<Expansion>, OperandError> {
    arity("li", texts, 2)?;
    let rd = &texts[0];
    let value = ops.value(&texts[1], pc)?;
    if value.abs() > WORD_MAX as Value {
        return fail(format!("{value} does not fit a word"));
    }
    if size == 1 {
        return Ok(vec![(
            "addi",
            vec![rd.clone(), "zero".into(), value.to_string()],
            Fields::default(),
        )]);
    }
    let (hi, lo) = split_constant(value);
    Ok(vec![
        ("lui", vec![rd.clone(), hi.to_string()], Fields::default()),
        ("addi", vec![rd.clone(), rd.clone(), lo.to_string()], Fields::default()),
    ])
}

/// Words a pseudo needs, decided in pass one and kept in pass two.
pub fn size(name: &str, texts: &[String], symbols: &Symbols) -> Result<i128, ExprError> {
    if name == "la" {
        return Ok(2);
    }
    if name == "li" && texts.len() == 2 {
        return match evaluate(&texts[1], symbols, None) {
            Err(ExprError::Undefined(_)) => Ok(2), // not known yet: the long form
            Err(e) => Err(e),
            Ok(v) => Ok(if v.abs() <= SHORT_IMMEDIATE { 1 } else { 2 }),
        };
    }
    Ok(1)
}
