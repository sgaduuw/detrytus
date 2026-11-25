//! Operand text to instruction fields, and the encoding of one
//! instruction. Ported from the original Python tools.

use std::cell::RefCell;

use detrytus_core::isa::{self, Format, Op};
use detrytus_core::trits::{WORD_TRITS, half, pow3};

use crate::expr::{ExprError, Linear, Relocatable, Symbols, Value, evaluate_linear};
use crate::object::RelocKind;
use crate::pyrepr::repr;
use crate::text::strip;

/// Why operands cannot be encoded: an expression error, or a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperandError {
    Expr(ExprError),
    Message(String),
}

impl OperandError {
    pub fn message(&self) -> &str {
        match self {
            OperandError::Expr(e) => e.message(),
            OperandError::Message(m) => m,
        }
    }
}

impl From<ExprError> for OperandError {
    fn from(e: ExprError) -> Self {
        OperandError::Expr(e)
    }
}

pub fn fail<T>(message: impl Into<String>) -> Result<T, OperandError> {
    Err(OperandError::Message(message.into()))
}

const FENCE_SETS: &str = "rwio";
const FENCE_ALL: Value = 40; // all four set trits: 1 + 3 + 9 + 27
const FIELD_SPLIT: Value = 81; // pos and len, pred and succ: two 4-trit halves

/// Instruction fields by name, in the order they were first set, as a
/// Python dict keeps them: encoding reports the first that does not fit.
#[derive(Debug, Default, Clone)]
pub struct Fields(Vec<(&'static str, Value)>);

impl Fields {
    pub fn set(&mut self, name: &'static str, value: Value) {
        match self.0.iter_mut().find(|(n, _)| *n == name) {
            Some(slot) => slot.1 = value,
            None => self.0.push((name, value)),
        }
    }

    fn take(&mut self, name: &str) -> Option<Value> {
        let i = self.0.iter().position(|(n, _)| *n == name)?;
        Some(self.0.remove(i).1)
    }
}

/// A value the linker must finish, noted while encoding: the address
/// of the word it goes in, where in that word, and the value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pending {
    pub at: Value,
    pub kind: RelocKind,
    pub value: Linear,
}

/// Reads operand text against the symbol table.
pub struct Operands<'a> {
    pub symbols: &'a Symbols,
    /// Assembling an object: what the linker will place.
    pub reloc: Option<&'a Relocatable>,
    /// The base of the section being assembled, when the linker places it.
    pub base: Option<&'a str>,
    pub pending: RefCell<Vec<Pending>>,
    /// Where the linker put things last time, when relaxing
    /// (docs/linker.md): an immediate or branch offset left for the
    /// linker is filled in from these, so it can be tried in a slot.
    pub hints: Option<&'a std::collections::HashMap<String, Value>>,
}

impl<'a> Operands<'a> {
    pub fn new(symbols: &'a Symbols) -> Self {
        Operands {
            symbols,
            reloc: None,
            base: None,
            pending: RefCell::new(Vec::new()),
            hints: None,
        }
    }

    /// A value that must be known now, such as a CSR number or a count.
    pub fn value(&self, text: &str, here: Value) -> Result<Value, OperandError> {
        let v = self.linear(text, here)?;
        v.as_constant().ok_or_else(|| {
            OperandError::Message(format!(
                "{} is not known until the program is linked",
                repr(text)
            ))
        })
    }

    /// A value the linker may finish.
    pub fn linear(&self, text: &str, here: Value) -> Result<Linear, OperandError> {
        Ok(evaluate_linear(text, self.symbols, self.reloc, Some(self.here(here)))?)
    }

    /// here as an address in the current section.
    pub fn here(&self, here: Value) -> Linear {
        Linear::based(here, self.base)
    }

    /// Leave value for the linker to put at `at`. 0 goes in the word
    /// meanwhile.
    pub fn defer(
        &self,
        text: &str,
        at: Value,
        kind: RelocKind,
        value: Linear,
    ) -> Result<Value, OperandError> {
        if value.split().is_none() {
            return fail(format!(
                "{} is more than the linker can finish: it adds one address and subtracts one",
                repr(text)
            ));
        }
        self.pending.borrow_mut().push(Pending { at, kind, value: value.clone() });
        Ok(self.hinted(text, &value)?.unwrap_or(0))
    }

    /// value as the hints have it, when relaxing.
    fn hinted(&self, text: &str, value: &Linear) -> Result<Option<Value>, OperandError> {
        let Some(hints) = self.hints else { return Ok(None) };
        let (plus, minus) = value.split().expect("checked by defer");
        let at = |base: Option<&str>| -> Result<Value, OperandError> {
            let Some(base) = base else { return Ok(0) };
            match hints.get(base) {
                Some(&v) => Ok(v),
                None => fail(format!("{} is not placed yet", repr(text))),
            }
        };
        Ok(Some(value.constant + at(plus)? - at(minus)?))
    }

    /// An immediate field: known now, or left for the linker.
    fn immediate(&self, text: &str, pc: Value) -> Result<Value, OperandError> {
        let v = self.linear(text, pc)?;
        match v.as_constant() {
            Some(c) => Ok(c),
            None => self.defer(text, pc, RelocKind::Imm, v),
        }
    }

    pub fn reg(text: &str) -> Result<Value, OperandError> {
        let name = strip(text).to_lowercase();
        match isa::reg_number(&name) {
            Some(r) => Ok(r as Value),
            None => fail(format!("not a register: {}", repr(text))),
        }
    }

    /// 'imm(reg)' or '(reg)' as (imm, reg).
    pub fn mem(&self, text: &str, here: Value) -> Result<(Value, Value), OperandError> {
        let text = strip(text);
        if !text.ends_with(')') || !text.contains('(') {
            return fail(format!("expected imm(reg), got {}", repr(text)));
        }
        let open = text.rfind('(').unwrap();
        let imm_text = strip(&text[..open]);
        let imm = if imm_text.is_empty() { 0 } else { self.immediate(imm_text, here)? };
        Ok((imm, Self::reg(&text[open + 1..text.len() - 1])?))
    }

    /// A branch target as a word offset from pc, in `field`. A target in
    /// another section, or outside the object, is the linker's.
    pub fn offset(
        &self,
        text: &str,
        pc: Value,
        field: &'static str,
    ) -> Result<Value, OperandError> {
        let target = self.linear(text, pc)?;
        let delta = target.add_scaled(&self.here(pc), -1).and_then(|d| d.as_constant());
        let Some(delta) = delta else {
            let v = self.defer(text, pc, RelocKind::Branch(field), target)?;
            if self.hints.is_none() {
                return Ok(v);
            }
            if (v - pc) % 3 != 0 {
                return fail(format!("branch target {v} is not word aligned"));
            }
            return Ok((v - pc) / 3);
        };
        let target = target.constant;
        if delta % 3 != 0 {
            return fail(format!("branch target {target} is not word aligned"));
        }
        Ok(delta / 3)
    }

    /// A name from a table (a tlut table, a CSR), or any expression.
    fn named(
        &self,
        table: &[(String, Value)],
        text: &str,
        here: Value,
    ) -> Result<Value, OperandError> {
        let name = strip(text).to_lowercase();
        match table.iter().find(|(n, _)| *n == name) {
            Some((_, v)) => Ok(*v),
            None => self.value(text, here),
        }
    }

    fn fence(texts: &[String]) -> Result<Value, OperandError> {
        if texts.is_empty() {
            return Ok(FENCE_ALL + FIELD_SPLIT * FENCE_ALL);
        }
        if texts.len() != 2 {
            return fail("fence takes no operands, or pred, succ");
        }
        let trits = |text: &str| -> Result<Value, OperandError> {
            let letters = strip(text).to_lowercase();
            if letters.is_empty() || letters.chars().any(|c| !FENCE_SETS.contains(c)) {
                return fail(format!("fence set {}: use letters r, w, i, o", repr(text)));
            }
            let mut seen = String::new();
            let mut v = 0;
            for c in letters.chars().filter(|c| {
                let new = !seen.contains(*c);
                seen.push(*c);
                new
            }) {
                v += pow3(FENCE_SETS.find(c).unwrap() as u32) as Value;
            }
            Ok(v)
        };
        Ok(trits(&texts[0])? + FIELD_SPLIT * trits(&texts[1])?)
    }

    /// Store the field(s) one operand letter sets.
    fn set(
        &self,
        letter: &str,
        op: &Op,
        text: &str,
        pc: Value,
        f: &mut Fields,
    ) -> Result<(), OperandError> {
        match letter {
            "d" => f.set("rd", Self::reg(text)?),
            "s" => f.set("rs1", Self::reg(text)?),
            "t" => f.set("rs2", Self::reg(text)?),
            "u" => f.set("rs3", Self::reg(text)?),
            "i" => f.set("imm", self.immediate(text, pc)?),
            "m" => {
                let (imm, rs1) = self.mem(text, pc)?;
                f.set("imm", imm);
                f.set("rs1", rs1);
            }
            "a" => {
                let (imm, rs1) = self.mem(text, pc)?;
                f.set("rs1", rs1);
                if imm != 0 {
                    return fail("atomics take no offset: write (reg)");
                }
            }
            "L" => {
                let field = if op.format == Format::B { "off" } else { "imm" };
                f.set(field, self.offset(text, pc, field)?)
            }
            "N" => f.set("offn", self.offset(text, pc, "offn")?),
            "P" => f.set("offp", self.offset(text, pc, "offp")?),
            "x" => {
                let tables: Vec<(String, Value)> =
                    isa::tables().iter().map(|(n, v)| (n.to_string(), *v as Value)).collect();
                f.set("fn", self.named(&tables, text, pc)?);
            }
            "c" => {
                let csrs: Vec<(String, Value)> =
                    isa::csrs().into_iter().map(|(n, v)| (n, v as Value)).collect();
                f.set("imm", self.named(&csrs, text, pc)?);
            }
            "pos" => f.set("pos", self.value(text, pc)?),
            "len" => f.set("len", self.value(text, pc)?),
            other => unreachable!("operand letter {other}"),
        }
        Ok(())
    }

    /// Encode one machine instruction from its operand texts.
    pub fn encode_instruction(
        &self,
        op: &Op,
        suffix: &str,
        texts: &[String],
        pc: Value,
        extra: Fields,
    ) -> Result<i64, OperandError> {
        let mut fields = extra;
        let mut syntax: Vec<&str> =
            if op.syntax.is_empty() { vec![] } else { op.syntax.split(',').collect() };
        let mut texts = texts;
        if syntax == ["F"] {
            fields.set("imm", Self::fence(texts)?);
            syntax.clear();
            texts = &[];
        }
        if texts.len() != syntax.len() {
            return fail(format!(
                "{} takes {} operand(s), got {}",
                op.name,
                syntax.len(),
                texts.len()
            ));
        }
        for (letter, text) in syntax.iter().zip(texts) {
            self.set(letter, op, text, pc, &mut fields)?;
        }
        combine_trit_field(&mut fields)?;
        apply_suffix(op, suffix, &mut fields)?;
        encode(op, &fields)
    }
}

fn combine_trit_field(fields: &mut Fields) -> Result<(), OperandError> {
    let Some(pos) = fields.take("pos") else { return Ok(()) };
    let len = fields.take("len").unwrap_or(0);
    if pos < 0 || len < 1 || pos + len > WORD_TRITS as Value {
        return fail(format!("trit field {pos}+{len} is outside the word"));
    }
    fields.set("imm", pos + FIELD_SPLIT * len);
    Ok(())
}

fn apply_suffix(op: &Op, suffix: &str, fields: &mut Fields) -> Result<(), OperandError> {
    if suffix.is_empty() {
        return Ok(());
    }
    let ordering = match suffix {
        "aq" => Some(1),
        "rl" => Some(3),
        "aqrl" => Some(4),
        _ => None,
    };
    let hint_t = match suffix {
        "n" => Some(-1),
        "z" => Some(0),
        "p" => Some(1),
        _ => None,
    };
    let hint_b = match suffix {
        "likely" => Some(1),
        "unlikely" => Some(-1),
        _ => None,
    };
    let rounding = isa::ROUNDING_SUFFIXES.iter().find(|(s, _)| *s == suffix).map(|(_, v)| *v);
    if op.rounds() {
        let Some(mode) = rounding else {
            return fail(format!("{} takes no suffix .{suffix}", op.name));
        };
        fields.set("fn", mode as Value);
        return Ok(());
    }
    match (op.atomic(), op.format, ordering, hint_t, hint_b) {
        (true, _, Some(o), _, _) => fields.set("rs3", o),
        (_, Format::T, _, Some(h), _) => fields.set("hint", h),
        (_, Format::B, _, _, Some(h)) => fields.set("hint", h),
        _ => return fail(format!("{} takes no suffix .{suffix}", op.name)),
    }
    Ok(())
}

/// isa.encode: the fields in order, the first that does not fit an error.
fn encode(op: &Op, fields: &Fields) -> Result<i64, OperandError> {
    let layout = isa::format_fields(op.format);
    let mut word = op.opcode as Value;
    for &(name, value) in &fields.0 {
        let Some(&(_, lo, width)) = layout.iter().find(|(n, _, _)| *n == name) else {
            return fail(format!("{}: format {:?} has no field {name}", op.name, op.format));
        };
        let max = half(width) as Value;
        if value.abs() > max {
            return fail(format!("{name} = {value} does not fit {width} trits (±{max})"));
        }
        word += value * pow3(lo) as Value;
    }
    Ok(word as i64)
}
