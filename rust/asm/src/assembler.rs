//! The two passes: lay out statements and bind labels, then encode.
//! Ported from the original Python tools.
//!
//! A statement's size must not change between the passes, so anything
//! whose size depends on a value (li) decides its size in pass one,
//! taking the long form when the value is not yet known.

use std::collections::{BTreeMap, HashMap};

use detrytus_core::trits::{TRYTE_MAX, TRYTE_TRITS, low, split_high};
use detrytus_core::{compressed, isa};

use crate::error::AsmError;
use crate::expr::{ExprError, Symbols, Value, evaluate};
use crate::operands::{Fields, OperandError, Operands};
use crate::source::{Kind, Statement, parse};
use crate::{pseudo, slots};

const TRYTE_DIRECTIVES: [&str; 4] = [".tryte", ".string", ".ascii", ".zero"];

/// One line of the listing: an emitted word, or the start of tryte data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listing {
    pub addr: i128,
    pub word: Option<i64>,
    pub line: usize,
    pub source: String,
    /// Disassembly of the emitted word.
    pub text: String,
    pub file: Option<String>,
    /// Where it sits in the image, if not at addr.
    pub load: i128,
    /// An instruction, not data from .word or .words.
    pub code: bool,
}

/// An assembled program.
#[derive(Debug, Default, Clone)]
pub struct Image {
    pub trytes: BTreeMap<i64, i64>,
    pub symbols: BTreeMap<String, Value>,
    pub listing: Vec<Listing>,
    /// Addresses, not .equ values.
    pub labels: BTreeMap<String, Value>,
    /// The labels in the order they were bound, as the Python image's
    /// dict keeps them; a debugger naming an address takes the first of
    /// several labels there.
    pub label_order: Vec<String>,
}

impl Image {
    /// The image in `.dti` form, as the Python writer produces it.
    pub fn to_bytes(&self) -> Vec<u8> {
        detrytus_core::image::dump(&self.trytes, &self.labels, &self.symbols)
    }
}

/// The first multiple of unit at which a word can be centred without
/// covering anything before loc, the next free tryte: the first c with
/// c - 1 >= loc.
fn next_centre(loc: Value, unit: Value) -> Value {
    -((-(loc + 1)).div_euclid(unit)) * unit
}

/// (low, middle, high) trytes of a word.
fn split_word(word: i64) -> [i64; 3] {
    let lo = low(word, TRYTE_TRITS);
    let rest = split_high(word, TRYTE_TRITS);
    [lo, low(rest, TRYTE_TRITS), split_high(rest, TRYTE_TRITS)]
}

fn split_suffix(mnemonic: &str) -> (&str, &str) {
    if isa::by_name(mnemonic).is_some() || pseudo::is_pseudo(mnemonic) {
        return (mnemonic, "");
    }
    match mnemonic.rfind('.') {
        Some(i) if i > 0 => (&mnemonic[..i], &mnemonic[i + 1..]),
        _ => (mnemonic, ""),
    }
}

/// An error raised inside a pass, before it is located at a statement.
enum PassError {
    Located(AsmError),
    Operand(OperandError),
}

impl From<AsmError> for PassError {
    fn from(e: AsmError) -> Self {
        PassError::Located(e)
    }
}

impl From<OperandError> for PassError {
    fn from(e: OperandError) -> Self {
        PassError::Operand(e)
    }
}

impl From<ExprError> for PassError {
    fn from(e: ExprError) -> Self {
        PassError::Operand(OperandError::Expr(e))
    }
}

type PassResult<T> = Result<T, PassError>;

#[derive(Default)]
struct Assembler {
    symbols: Symbols,
    image: Image,
    /// The next free tryte, so the first word centres on 0.
    loc: Value,
    pending: Vec<usize>,
    /// The current section, and its load address minus run address.
    section: String,
    shift: Value,
    /// (next free tryte, shift) of each section but the current one.
    sections: HashMap<String, (Value, Value)>,
    /// A `c.` instruction in slot A whose slot B is still free.
    open_slot: Option<usize>,
}

impl Assembler {
    fn new() -> Self {
        Assembler { loc: -1, section: "text".into(), ..Default::default() }
    }

    // --- pass one: layout ----------------------------------------------------

    fn layout(&mut self, stmts: &mut [Statement]) -> Result<(), AsmError> {
        for i in 0..stmts.len() {
            if !stmts[i].labels.is_empty() {
                self.pending.push(i);
            }
            // Two c. instructions share a bundle only when nothing else
            // comes between them.
            if !stmts[i].mnemonic.is_empty() && !slots::is_slot(&stmts[i].mnemonic) {
                self.open_slot = None;
            }
            if let Err(e) = self.lay(stmts, i) {
                // A check before the failing line that already fails is
                // the likelier cause, such as a file assembled without the
                // symbols it asserts it needs.
                if let Some(e) = self.failed_check(&stmts[..i]) {
                    return Err(e);
                }
                return Err(match e {
                    PassError::Located(e) => e,
                    PassError::Operand(e) => stmts[i].error(e.message()),
                });
            }
        }
        self.bind(stmts, self.loc)
    }

    fn lay(&mut self, stmts: &mut [Statement], i: usize) -> PassResult<()> {
        let mnemonic = stmts[i].mnemonic.clone();
        match mnemonic.as_str() {
            "" => stmts[i].kind = Kind::None,
            ".equ" | ".set" => {
                stmts[i].kind = Kind::None;
                let s = &stmts[i];
                let [name, value] = expect::<2>(s)?;
                let v = self.constant(&value)?;
                self.define(&name, v, s)?;
            }
            ".org" => {
                stmts[i].kind = Kind::None;
                let s = &stmts[i];
                let [target] = expect::<1>(s)?;
                let target = self.constant(&target)?;
                if low3(target) != 0 {
                    return Err(s
                        .error(".org needs a word-aligned address (a multiple of 3)")
                        .into());
                }
                self.loc = target - 1;
            }
            ".align" => {
                stmts[i].kind = Kind::None;
                let s = &stmts[i];
                let [power] = expect::<1>(s)?;
                let power = self.constant(&power)?;
                if !(0..=27).contains(&power) {
                    // 3**27 already spans every address.
                    let message = format!(".align takes a power of 3 from 0 to 27, not {power}");
                    return Err(s.error(message).into());
                }
                self.loc = next_centre(self.loc, 3i128.pow(power as u32)) - 1;
            }
            ".section" => {
                stmts[i].kind = Kind::None;
                self.lay_section(stmts, i)?;
            }
            // Checked in pass two, once every symbol is known.
            ".assert" => stmts[i].kind = Kind::None,
            m if TRYTE_DIRECTIVES.contains(&m) => self.lay_trytes(stmts, i)?,
            _ => self.lay_words(stmts, i)?,
        }
        Ok(())
    }

    fn bind(&mut self, stmts: &[Statement], addr: Value) -> Result<(), AsmError> {
        for i in std::mem::take(&mut self.pending) {
            let s = &stmts[i];
            for label in &s.labels {
                self.define(label, addr, s)?;
                self.image.labels.insert(label.clone(), addr);
                self.image.label_order.push(label.clone());
            }
        }
        Ok(())
    }

    fn define(&mut self, name: &str, value: Value, s: &Statement) -> Result<(), AsmError> {
        if self.symbols.contains_key(name) {
            return Err(s.error(format!("symbol {} defined twice", crate::pyrepr::repr(name))));
        }
        self.symbols.insert(name.to_string(), value);
        Ok(())
    }

    fn constant(&self, text: &str) -> Result<Value, ExprError> {
        evaluate(text, &self.symbols, None)
    }

    /// Switch sections. A section is placed once, on first use, and
    /// resumes where it left off after that.
    fn lay_section(&mut self, stmts: &[Statement], i: usize) -> PassResult<()> {
        let s = &stmts[i];
        if !(1..=3).contains(&s.operands.len()) {
            return Err(s.error(".section takes NAME, or NAME, LOAD[, RUN]").into());
        }
        let name = crate::text::strip(&s.operands[0]).to_string();
        if name == self.section && s.operands.len() == 1 {
            return Ok(());
        }
        let known = name == self.section || self.sections.contains_key(&name);
        let (start, shift) = if s.operands.len() > 1 {
            if known {
                let q = crate::pyrepr::repr(&name);
                return Err(s.error(format!("section {q} is already placed")).into());
            }
            self.place(s)?
        } else if !known {
            let q = crate::pyrepr::repr(&name);
            return Err(s
                .error(format!(
                    "section {q} needs an address the first time: .section {name}, ADDR"
                ))
                .into());
        } else {
            self.sections[&name]
        };
        // Labels written before the switch name the end of the old section.
        self.bind(stmts, self.loc)?;
        self.sections.insert(self.section.clone(), (self.loc, self.shift));
        self.sections.remove(&name);
        (self.section, self.loc, self.shift) = (name, start, shift);
        Ok(())
    }

    /// (first free tryte, shift) for a new section's LOAD and RUN.
    fn place(&self, s: &Statement) -> PassResult<(Value, Value)> {
        let load = self.constant(&s.operands[1])?;
        let run = if s.operands.len() == 3 { self.constant(&s.operands[2])? } else { load };
        if low3(run) != 0 {
            return Err(s
                .error("a section starts at a word-aligned address (a multiple of 3)")
                .into());
        }
        if (load - run).rem_euclid(3) != 0 {
            return Err(s
                .error("a section's load and run addresses must differ by a multiple of 3")
                .into());
        }
        Ok((run - 1, load - run))
    }

    fn lay_trytes(&mut self, stmts: &mut [Statement], i: usize) -> PassResult<()> {
        let size = self.tryte_count(&stmts[i])?;
        let s = &mut stmts[i];
        s.kind = Kind::Tryte;
        s.shift = self.shift;
        s.addr = self.loc;
        s.size = size;
        let addr = s.addr;
        self.bind(stmts, addr)?;
        self.loc += size;
        Ok(())
    }

    fn lay_words(&mut self, stmts: &mut [Statement], i: usize) -> PassResult<()> {
        if slots::is_slot(&stmts[i].mnemonic) {
            // A label starts a new word: a branch cannot enter slot B.
            if let Some(a) = self.open_slot.take()
                && self.pending.is_empty()
            {
                let (addr, shift) = (stmts[a].addr + 1, self.shift);
                stmts[a].partner = Some(i);
                let s = &mut stmts[i];
                (s.kind, s.shift, s.size, s.addr) = (Kind::SlotB, shift, 0, addr);
                return Ok(());
            }
            self.open_slot = Some(i);
        }
        let s = &stmts[i];
        let size = match s.mnemonic.as_str() {
            ".word" => s.operands.len() as i128,
            ".words" => {
                let [count] = expect::<1>(s)?;
                let n = self.constant(&count)?;
                if n < 0 {
                    return Err(s.error(".words needs a count of 0 or more").into());
                }
                n
            }
            m if slots::is_slot(m) => 1,
            m => pseudo::size(split_suffix(m).0, &s.operands, &self.symbols)?,
        };
        let addr = next_centre(self.loc, 3);
        let s = &mut stmts[i];
        (s.kind, s.shift, s.size, s.addr) = (Kind::Word, self.shift, size, addr);
        self.bind(stmts, addr)?;
        self.loc = addr + 3 * size - 1;
        Ok(())
    }

    fn tryte_count(&self, s: &Statement) -> PassResult<i128> {
        Ok(match s.mnemonic.as_str() {
            ".zero" => {
                let [count] = expect::<1>(s)?;
                self.constant(&count)?
            }
            ".tryte" => s.operands.len() as i128,
            _ => string(s)?.chars().count() as i128 + i128::from(s.mnemonic == ".string"),
        })
    }

    // --- pass two: emit ------------------------------------------------------

    fn emit(&mut self, stmts: &[Statement]) -> Result<(), AsmError> {
        for (i, s) in stmts.iter().enumerate() {
            let result = match s.kind {
                Kind::Tryte => self.emit_trytes(s),
                Kind::Word if slots::is_slot(&s.mnemonic) => self.emit_bundle(stmts, i),
                Kind::Word => self.emit_words(s),
                Kind::SlotB => Ok(()),
                Kind::None if s.mnemonic == ".assert" => self.check(s),
                Kind::None => Ok(()),
            };
            if let Err(e) = result {
                return Err(match e {
                    PassError::Located(e) => e,
                    PassError::Operand(e) => s.error(e.message()),
                });
            }
        }
        Ok(())
    }

    /// Place a tryte at its load address: addr (a run address) + shift.
    fn put(&mut self, addr: Value, tryte: i64, s: &Statement) -> PassResult<()> {
        let addr = addr + s.shift;
        let key =
            i64::try_from(addr).map_err(|_| s.error(format!("address {addr} is out of range")))?;
        if self.image.trytes.contains_key(&key) {
            return Err(s.error(format!("address {addr} assembled twice")).into());
        }
        self.image.trytes.insert(key, tryte);
        Ok(())
    }

    /// The first `.assert` among stmts that fails with the symbols known
    /// so far. One that cannot be evaluated yet, for a symbol not defined
    /// yet, is passed over.
    fn failed_check(&self, stmts: &[Statement]) -> Option<AsmError> {
        stmts.iter().filter(|s| s.mnemonic == ".assert").find_map(|s| match self.check(s) {
            Err(PassError::Located(e)) => Some(e),
            _ => None,
        })
    }

    /// `.assert EXPR, "MESSAGE"`: the message is the error when EXPR is 0.
    fn check(&self, s: &Statement) -> PassResult<()> {
        let [condition, message] =
            expect::<2>(s).map_err(|_| s.error(".assert takes a condition and a \"message\""))?;
        let message = quoted(s, &message)?;
        if crate::expr::evaluate_check(&condition, &self.symbols)? == 0 {
            return Err(s.error(message).into());
        }
        Ok(())
    }

    fn emit_words(&mut self, s: &Statement) -> PassResult<()> {
        let words = self.words(s)?;
        if words.len() as i128 != s.size {
            let message = format!(
                "internal error: {} sized {} in pass one but {} in pass two",
                s.mnemonic,
                s.size,
                words.len()
            );
            return Err(AsmError::new(message, s.line, &s.source, None).into());
        }
        for (i, word) in words.into_iter().enumerate() {
            let addr = s.addr + 3 * i as i128;
            for (offset, tryte) in [-1, 0, 1].into_iter().zip(split_word(word)) {
                self.put(addr + offset, tryte, s)?;
            }
            self.image.listing.push(Listing {
                addr,
                word: Some(word),
                line: s.line,
                source: s.source.clone(),
                text: String::new(),
                file: s.file.clone(),
                load: addr + s.shift,
                code: !s.mnemonic.starts_with('.'),
            });
        }
        Ok(())
    }

    /// A bundle: the `c.` instruction at i in slot A, and its partner in
    /// slot B or `c.nop`. Each slot has its line in the listing.
    fn emit_bundle(&mut self, stmts: &[Statement], i: usize) -> PassResult<()> {
        let a = &stmts[i];
        let b = a.partner.map(|j| &stmts[j]);
        let slot_a = self.slot(a)?;
        let slot_b = match b {
            Some(b) => self.slot(b).map_err(|e| match e {
                PassError::Operand(e) => PassError::Located(b.error(e.message())),
                located => located,
            })?,
            None => compressed::NOP,
        };
        let word = compressed::bundle(slot_a, slot_b);
        for (offset, tryte) in [-1, 0, 1].into_iter().zip(split_word(word)) {
            self.put(a.addr + offset, tryte, a)?;
        }
        let pc = Some(a.addr as i64);
        for (s, slot, w) in
            [(a, slot_a, Some(word))].into_iter().chain(b.map(|b| (b, slot_b, None)))
        {
            self.image.listing.push(Listing {
                addr: s.addr,
                word: w,
                line: s.line,
                source: s.source.clone(),
                text: crate::disasm::disassemble_slot(slot, pc),
                file: s.file.clone(),
                load: s.addr + s.shift,
                code: true,
            });
        }
        Ok(())
    }

    /// The slot a `c.` instruction encodes to: its full instruction,
    /// encoded with the bundle's address as pc, must fit the form.
    fn slot(&self, s: &Statement) -> PassResult<i64> {
        let base = s.addr - low3(s.addr);
        let ops = Operands { symbols: &self.symbols };
        let (name, texts) = slots::full(&s.mnemonic, &s.operands)?;
        let mut expansions = if pseudo::is_pseudo(name) {
            pseudo::expand(&ops, name, &texts, base, 1)?
        } else {
            vec![(name, texts, Fields::default())]
        };
        let (op, texts, extra) = expansions.pop().expect("one word");
        let op = isa::by_name(op).expect("a c. form names a real instruction");
        let word = ops.encode_instruction(op, "", &texts, base, extra)?;
        let d = isa::decode(word).expect("an encoded instruction decodes");
        compressed::compress(&d).ok_or_else(|| {
            let full = crate::disasm::disassemble(word, Some(base as i64));
            OperandError::Message(format!("{full} does not fit {}", s.mnemonic)).into()
        })
    }

    fn emit_trytes(&mut self, s: &Statement) -> PassResult<()> {
        let values: Vec<Value> = match s.mnemonic.as_str() {
            ".zero" => vec![0; s.size.max(0) as usize],
            ".tryte" => {
                let ops = Operands { symbols: &self.symbols };
                s.operands.iter().map(|op| ops.value(op, s.addr)).collect::<Result<_, _>>()?
            }
            _ => {
                let mut v: Vec<Value> = string(s)?.chars().map(|c| c as Value).collect();
                if s.mnemonic == ".string" {
                    v.push(0);
                }
                v
            }
        };
        for (i, v) in values.into_iter().enumerate() {
            if v.abs() > TRYTE_MAX as Value {
                return Err(s.error(format!("{v} does not fit a tryte")).into());
            }
            self.put(s.addr + i as i128, v as i64, s)?;
        }
        self.image.listing.push(Listing {
            addr: s.addr,
            word: None,
            line: s.line,
            source: s.source.clone(),
            text: String::new(),
            file: s.file.clone(),
            load: s.addr + s.shift,
            code: false,
        });
        Ok(())
    }

    fn words(&self, s: &Statement) -> PassResult<Vec<i64>> {
        let pc = s.addr;
        let ops = Operands { symbols: &self.symbols };
        if s.mnemonic == ".word" {
            return s
                .operands
                .iter()
                .map(|op| {
                    // A word is stored as given; the trytes take its low 27 trits.
                    let v = ops.value(op, pc)?;
                    Ok(((v + HALF_WORD_MOD).rem_euclid(WORD_MOD) - HALF_WORD_MOD) as i64)
                })
                .collect();
        }
        if s.mnemonic == ".words" {
            return Ok(vec![0; s.size as usize]);
        }
        let (name, suffix) = split_suffix(&s.mnemonic);
        let expansions = if pseudo::is_pseudo(name) {
            pseudo::expand(&ops, name, &s.operands, pc, s.size)?
        } else if let Some(op) = isa::by_name(name) {
            vec![(op.name, s.operands.clone(), Fields::default())]
        } else {
            let q = crate::pyrepr::repr(&s.mnemonic);
            return Err(OperandError::Message(format!("unknown instruction {q}")).into());
        };
        let mut words = Vec::with_capacity(expansions.len());
        for (i, (op, texts, extra)) in expansions.into_iter().enumerate() {
            let op = isa::by_name(op).expect("pseudo-instructions expand to real ones");
            words.push(ops.encode_instruction(op, suffix, &texts, pc + 3 * i as i128, extra)?);
        }
        Ok(words)
    }
}

const WORD_MOD: Value = 7_625_597_484_987; // 3**27
const HALF_WORD_MOD: Value = (WORD_MOD - 1) / 2;

fn low3(x: Value) -> Value {
    (x + 1).rem_euclid(3) - 1
}

fn expect<const N: usize>(s: &Statement) -> Result<[String; N], AsmError> {
    s.operands
        .clone()
        .try_into()
        .map_err(|_| s.error(format!("{} takes {N} operand(s)", s.mnemonic)))
}

/// A .string or .ascii operand, with its escapes applied.
fn string(s: &Statement) -> Result<String, AsmError> {
    let [lit] = expect::<1>(s)?;
    quoted(s, &lit)
}

/// A double-quoted literal's text, with its escapes applied.
fn quoted(s: &Statement, lit: &str) -> Result<String, AsmError> {
    let chars: Vec<char> = lit.chars().collect();
    if chars.len() < 2 || chars[0] != '"' || chars[chars.len() - 1] != '"' {
        return Err(s.error("expected a double-quoted string"));
    }
    let body = &chars[1..chars.len() - 1];
    let mut out = String::new();
    let mut i = 0;
    while i < body.len() {
        if body[i] == '\\' && i + 1 < body.len() && body[i + 1] != '\n' {
            let c = body[i + 1];
            out.push(
                crate::expr::escape(c).ok_or_else(|| s.error(crate::expr::unknown_escape(c)))?,
            );
            i += 2;
        } else {
            out.push(body[i]);
            i += 1;
        }
    }
    Ok(out)
}

/// Assemble source text. file, if given, names it in error messages and
/// is where .include paths start from.
pub fn assemble(source: &str, file: Option<&str>) -> Result<Image, AsmError> {
    let mut asm = Assembler::new();
    let mut stmts = parse(source, file)?;
    asm.layout(&mut stmts)?;
    asm.emit(&stmts)?;
    let mut image = asm.image;
    image.symbols = asm.symbols.into_iter().collect();
    for entry in image.listing.iter_mut().filter(|e| e.text.is_empty()) {
        if let Some(word) = entry.word {
            entry.text = crate::disasm::disassemble(word, Some(entry.addr as i64));
        }
    }
    Ok(image)
}
