//! The global pointer (docs/assembler.md): under `.gp E`, `la rd, symbol`
//! is one `addi rd, gp, symbol - E`, and a load or store may name a
//! symbol instead of `imm(reg)`. Each section keeps its own `.gp`.
//!
//! This runs before layout and rewrites each such statement into the
//! machine instruction it stands for, so layout sees one word that
//! automatic bundling may put in a slot (`c.lagp`, `c.lwgp`, `c.swgp`).

use std::collections::HashMap;

use detrytus_core::isa::reg_number;
use detrytus_core::trits::half;

use crate::error::AsmError;
use crate::expr::Value;
use crate::operands::{OperandError, Operands, fail};
use crate::source::Statement;
use crate::text::strip;

/// The loads and stores that take a symbol under `.gp`.
const MEMORY: [&str; 4] = ["lw", "lt", "sw", "st"];

/// What a rewritten statement reaches: its symbol and gp's value, as
/// written, so a distance out of reach is reported in their terms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reach {
    pub symbol: String,
    pub gp: String,
}

impl Reach {
    /// Refuse a symbol farther from gp than an I-format offset reaches.
    pub fn check(&self, ops: &Operands, here: Value) -> Result<(), OperandError> {
        let distance = ops.value(&self.symbol, here)? - ops.value(&self.gp, here)?;
        if distance.abs() > half(16) as Value {
            return fail(format!(
                "{} is {distance} trytes from gp ({}), beyond the 16 trits an offset holds",
                self.symbol, self.gp
            ));
        }
        Ok(())
    }
}

/// Rewrite every `la` and symbolic load or store under `.gp`.
pub fn rewrite(stmts: &mut [Statement]) -> Result<(), AsmError> {
    let gp_reg = reg_number("gp").expect("gp is an ABI name") as Value;
    let mut section = "text".to_string();
    // The `.gp` in force in each section that has one.
    let mut pointers: HashMap<String, String> = HashMap::new();
    for s in stmts.iter_mut() {
        match s.mnemonic.as_str() {
            ".section" => {
                if let Some(name) = s.operands.first() {
                    section = strip(name).to_string();
                }
            }
            ".gp" => {
                let [e] = <[String; 1]>::try_from(s.operands.clone())
                    .map_err(|_| s.error(".gp takes 1 operand(s)"))?;
                pointers.insert(section.clone(), strip(&e).to_string());
            }
            ".nogp" => {
                if !s.operands.is_empty() {
                    return Err(s.error(".nogp takes 0 operand(s)"));
                }
                pointers.remove(&section);
            }
            _ => {
                if let Some(e) = pointers.get(&section) {
                    rewrite_one(s, e, gp_reg);
                }
            }
        }
    }
    Ok(())
}

fn rewrite_one(s: &mut Statement, e: &str, gp_reg: Value) {
    if s.operands.len() != 2 {
        return; // the instruction reports its own arity
    }
    let symbol = strip(&s.operands[1]).to_string();
    let offset = format!("({symbol}) - ({e})");
    if s.mnemonic == "la" {
        // la gp stays pc-relative: it is what gives gp its value.
        if Operands::reg(&s.operands[0]).ok() == Some(gp_reg) {
            return;
        }
        s.mnemonic = "addi".into();
        s.operands = vec![s.operands[0].clone(), "gp".into(), offset];
    } else if MEMORY.contains(&s.mnemonic.as_str()) && !based(&symbol) {
        s.operands[1] = format!("({offset})(gp)");
    } else {
        return;
    }
    s.gp = Some(Reach { symbol, gp: e.to_string() });
}

/// Whether a memory operand already names its base, as `imm(reg)`.
fn based(text: &str) -> bool {
    text.ends_with(')')
        && text
            .rfind('(')
            .is_some_and(|open| Operands::reg(&text[open + 1..text.len() - 1]).is_ok())
}
