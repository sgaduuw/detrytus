//! The two passes: lay out statements and bind labels, then encode.
//! Ported from the original Python tools.
//!
//! A statement's size must not change between the passes, so anything
//! whose size depends on a value (li) decides its size in pass one,
//! taking the long form when the value is not yet known.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use detrytus_core::trits::{TRYTE_MAX, TRYTE_TRITS, low, split_high};
use detrytus_core::{compressed, isa};

use crate::error::AsmError;
use crate::expr::{ExprError, Linear, Relocatable, Symbols, Value, evaluate_linear};
use crate::object::{Object, Reloc, RelocKind, Section, Symbol};
use crate::operands::{Fields, OperandError, Operands, Pending};
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
    /// Instructions under `.compress` that may go in a slot: those whose
    /// full instruction did not fail to fit one in an earlier layout.
    slotted: BTreeSet<usize>,
    /// Assembling an object (docs/linker.md): what the linker finishes.
    object: Option<Objects>,
    /// Relaxing: where a previous link put each base this object names.
    hints: Option<HashMap<String, Value>>,
}

/// What assembling an object keeps besides the image.
#[derive(Default, Clone)]
struct Objects {
    reloc: Relocatable,
    /// The sections the linker places, each with its base's name.
    floating: HashMap<String, String>,
    /// Every section, in the order they first appear.
    order: Vec<String>,
    align: HashMap<String, u32>,
    /// Each section's trytes, by run address.
    trytes: HashMap<String, BTreeMap<i64, i64>>,
    relocs: HashMap<String, Vec<Reloc>>,
    /// Symbols in the order they were defined.
    defined: Vec<String>,
    /// The `.global` names, with the statement that declared each.
    globals: Vec<(String, usize)>,
    /// Each section's part being assembled, after a `.part`: `NAME:K`.
    parts: HashMap<String, String>,
}

impl Objects {
    fn new(defined: HashSet<String>) -> Self {
        let reloc = Relocatable { defined, ..Default::default() };
        let mut o = Objects { reloc, ..Default::default() };
        o.float("text");
        o
    }

    fn float(&mut self, section: &str) {
        self.floating.insert(section.to_string(), format!("[{section}]"));
        self.align.insert(section.to_string(), 1);
        self.order.push(section.to_string());
    }
}

impl Assembler {
    fn new(object: Option<Objects>) -> Self {
        Assembler { loc: -1, section: "text".into(), object, ..Default::default() }
    }

    /// Operands for a statement in section.
    fn ops(&self, section: &str) -> Operands<'_> {
        let mut ops = Operands::new(&self.symbols);
        if let Some(o) = &self.object {
            ops.reloc = Some(&o.reloc);
            ops.base = o.floating.get(section).map(String::as_str);
        }
        ops
    }

    /// Whether the linker places the current section.
    fn floating(&self) -> bool {
        self.object.as_ref().is_some_and(|o| o.floating.contains_key(&self.section))
    }

    // --- pass one: layout ----------------------------------------------------

    fn layout(&mut self, stmts: &mut [Statement]) -> Result<(), AsmError> {
        for i in 0..stmts.len() {
            if !stmts[i].labels.is_empty() {
                self.pending.push(i);
            }
            // Two c. instructions share a bundle only when nothing else
            // comes between them.
            if !stmts[i].mnemonic.is_empty()
                && !slots::is_slot(&stmts[i].mnemonic)
                && !self.slotted.contains(&i)
            {
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
                let v = self.relocatable(&value)?;
                self.define(&name, v.constant, s)?;
                if let Some(o) = &mut self.object
                    && !v.terms.is_empty()
                {
                    // A name for an address the linker places, or for an
                    // external symbol plus a constant.
                    match v.split() {
                        Some((Some(base), None)) => {
                            o.reloc.bases.insert(name, base.to_string());
                        }
                        _ => {
                            let q = crate::pyrepr::repr(&value);
                            return Err(s
                                .error(format!("{q} is not one address plus a constant"))
                                .into());
                        }
                    }
                }
            }
            ".global" => {
                stmts[i].kind = Kind::None;
                let s = &stmts[i];
                if s.operands.is_empty() {
                    return Err(s.error(".global takes one or more names").into());
                }
                if let Some(o) = &mut self.object {
                    for name in &s.operands {
                        o.globals.push((crate::text::strip(name).to_string(), i));
                    }
                }
            }
            ".part" => {
                stmts[i].kind = Kind::None;
                expect::<0>(&stmts[i])?;
                if self.object.is_some() {
                    self.new_part(stmts)?;
                }
            }
            ".org" => {
                stmts[i].kind = Kind::None;
                let s = &stmts[i];
                if self.floating() {
                    return Err(s
                        .error(".org in a section the linker places: give the section an address")
                        .into());
                }
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
                let section = self.section.clone();
                if let Some(o) = &mut self.object {
                    let align = o.align.entry(section).or_insert(0);
                    *align = (*align).max(power as u32);
                }
            }
            ".section" => {
                stmts[i].kind = Kind::None;
                self.lay_section(stmts, i)?;
            }
            // Checked in pass two, once every symbol is known.
            ".assert" => stmts[i].kind = Kind::None,
            // Read by compressible() before layout.
            // Read by gp::rewrite() before layout.
            ".gp" | ".nogp" => stmts[i].kind = Kind::None,
            ".compress" | ".nocompress" => {
                stmts[i].kind = Kind::None;
                expect::<0>(&stmts[i])?;
            }
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
                if let Some(o) = &mut self.object
                    && let Some(base) = o.floating.get(&self.section)
                {
                    o.reloc.bases.insert(label.clone(), base.clone());
                }
            }
        }
        Ok(())
    }

    fn define(&mut self, name: &str, value: Value, s: &Statement) -> Result<(), AsmError> {
        if self.symbols.contains_key(name) {
            return Err(s.error(format!("symbol {} defined twice", crate::pyrepr::repr(name))));
        }
        self.symbols.insert(name.to_string(), value);
        if let Some(o) = &mut self.object {
            o.defined.push(name.to_string());
        }
        Ok(())
    }

    fn relocatable(&self, text: &str) -> Result<Linear, ExprError> {
        evaluate_linear(text, &self.symbols, self.object.as_ref().map(|o| &o.reloc), None)
    }

    fn constant(&self, text: &str) -> Result<Value, ExprError> {
        let v = self.relocatable(text)?;
        v.as_constant().ok_or_else(|| {
            let q = crate::pyrepr::repr(text);
            ExprError::Other(format!("{q} is not known until the program is linked"))
        })
    }

    /// Switch sections. A section is placed once, on first use, and
    /// resumes where it left off after that.
    /// `.part` in an object: the current section goes on in a new part,
    /// which the linker places on its own and leaves out if nothing uses
    /// it (docs/linker.md).
    fn new_part(&mut self, stmts: &[Statement]) -> PassResult<()> {
        let o = self.object.as_mut().expect("assembling an object");
        let section = self.section.split(':').next().unwrap_or_default().to_string();
        let count = o.order.iter().filter(|p| p.split(':').next() == Some(&section)).count();
        let part = format!("{section}:{}", count + 1);
        o.float(&part);
        o.parts.insert(section, part.clone());
        // Labels written before it name the end of the part left.
        self.bind(stmts, self.loc)?;
        self.sections.insert(self.section.clone(), (self.loc, self.shift));
        (self.section, self.loc, self.shift) = (part, -1, 0);
        Ok(())
    }

    fn lay_section(&mut self, stmts: &[Statement], i: usize) -> PassResult<()> {
        let s = &stmts[i];
        if !(1..=3).contains(&s.operands.len()) {
            return Err(s.error(".section takes NAME, or NAME, LOAD[, RUN]").into());
        }
        let mut name = crate::text::strip(&s.operands[0]).to_string();
        // A section resumes in its latest part.
        if let Some(part) = self.object.as_ref().and_then(|o| o.parts.get(&name))
            && s.operands.len() == 1
        {
            name = part.clone();
        }
        if name == self.section && s.operands.len() == 1 {
            return Ok(());
        }
        let known = name == self.section || self.sections.contains_key(&name);
        let (start, shift) = if s.operands.len() > 1 {
            if known {
                let q = crate::pyrepr::repr(&name);
                return Err(s.error(format!("section {q} is already placed")).into());
            }
            let placed = self.place(s)?;
            if let Some(o) = &mut self.object {
                o.order.push(name.clone());
            }
            placed
        } else if !known && self.object.is_some() {
            // In an object, the linker places it.
            self.object.as_mut().unwrap().float(&name);
            (-1, 0)
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
        s.section = self.section.clone();
        s.shift = self.shift;
        s.addr = self.loc;
        s.size = size;
        let addr = s.addr;
        self.bind(stmts, addr)?;
        self.loc += size;
        Ok(())
    }

    fn lay_words(&mut self, stmts: &mut [Statement], i: usize) -> PassResult<()> {
        let size = self.word_count(&stmts[i])?;
        let manual = slots::is_slot(&stmts[i].mnemonic);
        if manual || (self.slotted.contains(&i) && size == 1) {
            // A label starts a new word: a branch cannot enter slot B.
            if let Some(a) = self.open_slot.take()
                && self.pending.is_empty()
            {
                let (addr, shift) = (stmts[a].addr + 1, self.shift);
                stmts[a].partner = Some(i);
                let s = &mut stmts[i];
                s.section = self.section.clone();
                (s.kind, s.shift, s.size, s.addr) = (Kind::SlotB, shift, 0, addr);
                return Ok(());
            }
            // Slot A, or a full word if nothing pairs with it: the same
            // size either way.
            self.open_slot = Some(i);
        } else {
            self.open_slot = None;
        }
        let addr = next_centre(self.loc, 3);
        let s = &mut stmts[i];
        s.section = self.section.clone();
        (s.kind, s.shift, s.size, s.addr) = (Kind::Word, self.shift, size, addr);
        self.bind(stmts, addr)?;
        self.loc = addr + 3 * size - 1;
        Ok(())
    }

    fn word_count(&self, s: &Statement) -> PassResult<i128> {
        Ok(match s.mnemonic.as_str() {
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
            m => pseudo::size(split_suffix(m).0, &s.operands, &self.ops(&self.section))?,
        })
    }

    /// The paired instructions written under `.compress` whose full
    /// instruction does not fit a slot where layout put them.
    fn unfit(&self, stmts: &[Statement]) -> Vec<usize> {
        let paired = |i: usize| stmts[i].kind == Kind::SlotB || stmts[i].partner.is_some();
        self.slotted
            .iter()
            .copied()
            .filter(|&i| paired(i) && self.slot(&stmts[i]).is_err())
            .collect()
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
                Kind::Word if slots::is_slot(&s.mnemonic) || s.partner.is_some() => {
                    self.emit_bundle(stmts, i)
                }
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
        if let Some(o) = &mut self.object {
            // An object keeps each section apart, by run address: the
            // linker checks where they land.
            let key = i64::try_from(addr)
                .map_err(|_| s.error(format!("address {addr} is out of range")))?;
            let trytes = o.trytes.entry(s.section.clone()).or_default();
            if trytes.insert(key, tryte).is_some() {
                return Err(s.error(format!("address {addr} assembled twice")).into());
            }
            return Ok(());
        }
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
        let reloc = self.object.as_ref().map(|o| &o.reloc);
        if crate::expr::evaluate_check(&condition, &self.symbols, reloc)? == 0 {
            return Err(s.error(message).into());
        }
        Ok(())
    }

    /// Keep what the linker must finish in s's words.
    fn keep(&mut self, s: &Statement, pending: Vec<Pending>) {
        let Some(o) = &mut self.object else { return };
        for p in pending {
            let (plus, minus) = p.value.split().expect("deferred values split");
            o.relocs.entry(s.section.clone()).or_default().push(Reloc {
                addr: p.at,
                kind: p.kind,
                addend: p.value.constant,
                plus: plus.map(str::to_string),
                minus: minus.map(str::to_string),
                line: s.line,
                file: s.file.clone(),
            });
        }
    }

    fn emit_words(&mut self, s: &Statement) -> PassResult<()> {
        let ops = self.ops(&s.section);
        let words = self.words_at(&ops, s, s.addr);
        let pending = ops.pending.into_inner();
        let words = words?;
        self.keep(s, pending);
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
        let (slot_a, pending_a) = self.slot(a)?;
        let (slot_b, pending_b) = match b {
            Some(b) => self.slot(b).map_err(|e| match e {
                PassError::Operand(e) => PassError::Located(b.error(e.message())),
                located => located,
            })?,
            None => (compressed::NOP, Vec::new()),
        };
        // What the linker finishes in a slot, counting from the bundle.
        let in_slot = |pending: Vec<Pending>, b: bool| -> Vec<Pending> {
            pending
                .into_iter()
                .map(|p| Pending { kind: p.kind.in_slot(b).expect("checked by slot"), ..p })
                .collect()
        };
        self.keep(a, in_slot(pending_a, false));
        if let Some(b) = b {
            self.keep(b, in_slot(pending_b, true));
        }
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

    /// The slot an instruction encodes to: its full instruction, encoded
    /// with the bundle's address as pc, must fit a form, and for a `c.`
    /// instruction the one it names.
    /// What the linker must finish in it comes back too: when relaxing,
    /// an immediate or a branch offset, tried with the hints' values.
    fn slot(&self, s: &Statement) -> PassResult<(i64, Vec<Pending>)> {
        let base = s.addr - low3(s.addr);
        if !slots::is_slot(&s.mnemonic) {
            let mut ops = self.ops(&s.section);
            ops.hints = self.hints.as_ref();
            let word = self.words_at(&ops, s, base)?[0];
            let pending = ops.pending.take();
            if ops.hints.is_none() || pending.iter().any(|p| p.kind.in_slot(false).is_none()) {
                ops.pending.replace(pending.clone());
                unlinked(&ops, s)?;
            }
            let d = isa::decode(word).expect("an encoded instruction decodes");
            let slot = compressed::compress(&d).ok_or_else(|| {
                OperandError::Message(format!("{} does not fit a slot", s.mnemonic))
            })?;
            return Ok((slot, pending));
        }
        let ops = self.ops(&s.section);
        let (name, texts) = slots::full(&s.mnemonic, &s.operands)?;
        let mut expansions = if pseudo::is_pseudo(name) {
            pseudo::expand(&ops, name, &texts, base, 1)?
        } else {
            vec![(name, texts, Fields::default())]
        };
        let (op, texts, extra) = expansions.pop().expect("one word");
        let op = isa::by_name(op).expect("a c. form names a real instruction");
        let word = ops.encode_instruction(op, "", &texts, base, extra)?;
        unlinked(&ops, s)?;
        let d = isa::decode(word).expect("an encoded instruction decodes");
        let slot = compressed::compress(&d).ok_or_else(|| {
            let full = crate::disasm::disassemble(word, Some(base as i64));
            OperandError::Message(format!("{full} does not fit {}", s.mnemonic))
        })?;
        Ok((slot, Vec::new()))
    }

    fn emit_trytes(&mut self, s: &Statement) -> PassResult<()> {
        let values: Vec<Value> = match s.mnemonic.as_str() {
            ".zero" => vec![0; s.size.max(0) as usize],
            ".tryte" => {
                let ops = self.ops(&s.section);
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

    /// The words of a statement assembled at pc, which differs from its
    /// address only in slot B, where offsets count from the bundle.
    /// What the linker must finish is left in ops.
    fn words_at(&self, ops: &Operands, s: &Statement, pc: Value) -> PassResult<Vec<i64>> {
        if s.mnemonic == ".word" {
            return s
                .operands
                .iter()
                .enumerate()
                .map(|(i, op)| {
                    // A word is stored as given; the trytes take its low 27 trits.
                    let v = ops.linear(op, pc)?;
                    let v = match v.as_constant() {
                        Some(v) => v,
                        None => ops.defer(op, pc + 3 * i as i128, RelocKind::Word, v)?,
                    };
                    Ok(((v + HALF_WORD_MOD).rem_euclid(WORD_MOD) - HALF_WORD_MOD) as i64)
                })
                .collect();
        }
        if s.mnemonic == ".words" {
            return Ok(vec![0; s.size as usize]);
        }
        if let Some(reach) = &s.gp {
            reach.check(ops, pc)?;
        }
        let (name, suffix) = split_suffix(&s.mnemonic);
        let expansions = if pseudo::is_pseudo(name) {
            pseudo::expand(ops, name, &s.operands, pc, s.size.max(1))?
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

/// The instructions between `.compress` and `.nocompress` (or the end):
/// the ones automatic pairing may put in a slot.
fn compressible(stmts: &[Statement]) -> BTreeSet<usize> {
    let mut on = false;
    let mut found = BTreeSet::new();
    for (i, s) in stmts.iter().enumerate() {
        match s.mnemonic.as_str() {
            ".compress" => on = true,
            ".nocompress" => on = false,
            m if on && !m.is_empty() && !m.starts_with('.') && !slots::is_slot(m) => {
                found.insert(i);
            }
            _ => {}
        }
    }
    found
}

/// Assemble source text. file, if given, names it in error messages and
/// is where .include paths start from.
///
/// Instructions under `.compress` start out allowed into slots. Whether
/// one fits depends on where layout puts it (a short branch's reach), so
/// layout repeats, each time without the ones that did not fit, until
/// every pair fits. The set only shrinks, so this ends.
pub fn assemble(source: &str, file: Option<&str>) -> Result<Image, AsmError> {
    let (asm, _) = run(source, file, false, None)?;
    let mut image = asm.image;
    image.symbols = asm.symbols.into_iter().collect();
    Ok(image)
}

/// Assemble source text into an object for the linker (docs/linker.md),
/// with its listing. A section without an address is the linker's to
/// place, and a name the source does not define is external.
pub fn assemble_object(source: &str, file: Option<&str>) -> Result<(Object, Image), AsmError> {
    object_from(source, file, None)
}

/// What relaxing (docs/linker.md) tells the assembler: where a previous
/// link put each base the object names (its own sections, `[NAME]`, and
/// other objects' globals), and the instructions, by (file, line), not
/// to put in a slot after all because the link found they did not fit.
#[derive(Debug, Clone, Default)]
pub struct Relax {
    pub hints: HashMap<String, Value>,
    pub excluded: HashSet<(Option<String>, usize)>,
}

/// Assemble an object as assemble_object does, but let automatic pairing
/// put an instruction the linker finishes in a slot when, with the
/// hints' values, it fits one. The linker checks it still fits.
pub fn assemble_object_relaxed(
    source: &str,
    file: Option<&str>,
    relax: &Relax,
) -> Result<(Object, Image), AsmError> {
    object_from(source, file, Some(relax))
}

fn object_from(
    source: &str,
    file: Option<&str>,
    relax: Option<&Relax>,
) -> Result<(Object, Image), AsmError> {
    let (mut asm, stmts) = run(source, file, true, relax)?;
    let mut o = asm.object.take().expect("assembled as an object");
    // Each section ends where layout left it.
    asm.sections.insert(asm.section.clone(), (asm.loc, asm.shift));
    let section_of =
        |name: &str| o.reloc.bases.get(name).and_then(|b| b.strip_prefix('[')?.strip_suffix(']'));
    let mut globals = HashSet::new();
    for (name, i) in &o.globals {
        let s = &stmts[*i];
        if !asm.symbols.contains_key(name) {
            let q = crate::pyrepr::repr(name);
            return Err(s.error(format!("{q} is declared .global but never defined")));
        }
        if o.reloc.bases.contains_key(name) && section_of(name).is_none() {
            let q = crate::pyrepr::repr(name);
            return Err(s.error(format!("{q} names an external symbol and cannot be global")));
        }
        globals.insert(name.clone());
    }
    let mut symbols = Vec::new();
    for name in &o.defined {
        // A name for an external symbol plus a constant stays local to
        // the source: the linker resolves its uses.
        if o.reloc.bases.contains_key(name) && section_of(name).is_none() {
            continue;
        }
        symbols.push(Symbol {
            name: name.clone(),
            global: globals.contains(name),
            label: asm.image.labels.contains_key(name),
            value: asm.symbols[name],
            section: section_of(name).map(str::to_string),
        });
    }
    // A section with nothing in it, and nothing that names it, is left out.
    let named: HashSet<String> = symbols
        .iter()
        .filter_map(|s| s.section.clone())
        .chain(
            o.relocs
                .values()
                .flatten()
                .flat_map(|r| [r.plus.clone(), r.minus.clone()])
                .flatten()
                .filter_map(|b| Some(b.strip_prefix('[')?.strip_suffix(']')?.to_string())),
        )
        .collect();
    let mut sections = Vec::new();
    for name in &o.order {
        let trytes = o.trytes.remove(name).unwrap_or_default();
        let relocs = o.relocs.remove(name).unwrap_or_default();
        if trytes.is_empty() && relocs.is_empty() && !named.contains(name) {
            continue;
        }
        let (end, shift) = asm.sections[name];
        sections.push(Section {
            name: name.clone(),
            placed: (!o.floating.contains_key(name)).then_some(shift),
            align: o.align.get(name).copied().unwrap_or(1),
            end,
            trytes,
            relocs,
        });
    }
    let mut image = asm.image;
    image.symbols = asm.symbols.into_iter().collect();
    Ok((Object { sections, symbols }, image))
}

/// Instructions under `.compress` start out allowed into slots. Whether
/// one fits depends on where layout put it (a short branch's reach), so
/// layout repeats, each time without the ones that did not fit, until
/// every pair fits. The set only shrinks, so this ends.
fn run(
    source: &str,
    file: Option<&str>,
    object: bool,
    relax: Option<&Relax>,
) -> Result<(Assembler, Vec<Statement>), AsmError> {
    let mut parsed = parse(source, file)?;
    crate::gp::rewrite(&mut parsed)?;
    let objects = object.then(|| Objects::new(defined_names(&parsed)));
    let mut slotted = compressible(&parsed);
    if let Some(relax) = relax {
        slotted.retain(|&i| !relax.excluded.contains(&(parsed[i].file.clone(), parsed[i].line)));
    }
    let hints = relax.map(|r| r.hints.clone());
    let (mut asm, stmts) = loop {
        let mut asm = Assembler {
            slotted: slotted.clone(),
            hints: hints.clone(),
            ..Assembler::new(objects.clone())
        };
        let mut stmts = parsed.clone();
        asm.layout(&mut stmts)?;
        let unfit = asm.unfit(&stmts);
        if unfit.is_empty() {
            break (asm, stmts);
        }
        for i in unfit {
            slotted.remove(&i);
        }
    };
    asm.emit(&stmts)?;
    for entry in asm.image.listing.iter_mut().filter(|e| e.text.is_empty()) {
        if let Some(word) = entry.word {
            entry.text = crate::disasm::disassemble(word, Some(entry.addr as i64));
        }
    }
    Ok((asm, stmts))
}

/// Every name the source defines, as a label or a constant: any other
/// name is external.
fn defined_names(stmts: &[Statement]) -> HashSet<String> {
    let mut names = HashSet::new();
    for s in stmts {
        names.extend(s.labels.iter().cloned());
        if matches!(s.mnemonic.as_str(), ".equ" | ".set")
            && let Some(name) = s.operands.first()
        {
            names.insert(crate::text::strip(name).to_string());
        }
    }
    names
}

/// Refuse a slot to an instruction the linker has to finish: its reach
/// is not known until then.
fn unlinked(ops: &Operands, s: &Statement) -> PassResult<()> {
    if ops.pending.borrow().is_empty() {
        return Ok(());
    }
    let message = format!("{} is finished by the linker, so it cannot go in a slot", s.mnemonic);
    Err(OperandError::Message(message).into())
}
