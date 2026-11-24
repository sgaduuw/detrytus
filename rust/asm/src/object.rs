//! Objects: what `detrytus asm -c` writes and the linker reads
//! (docs/linker.md). An object is text, a line to a fact:
//!
//! ```text
//! detrytus object 1
//! section NAME ALIGN END [at SHIFT]
//! data NAME ADDR TRYTE...
//! reloc NAME ADDR KIND ADDEND [+BASE] [-BASE] @ LINE [FILE]
//! symbol global|local label|constant NAME VALUE [in SECTION]
//! ```
//!
//! A section with `at` was placed by its source: its addresses are
//! where it runs, and SHIFT is its load address minus its run address.
//! Any other section is the linker's to place, and its addresses count
//! from its base, the address its first word is centred on; it starts
//! one tryte below, at -1, and ends before END. ALIGN is the power of 3
//! its base must be a multiple of. `data` lists trytes from ADDR up.
//!
//! A relocation says what the linker writes at ADDR: the value
//! ADDEND + BASE − BASE, a base being `[SECTION]` (where this object's
//! section was placed) or an external symbol's name. LINE and FILE
//! name the source line, for the linker's messages. A symbol `in` a
//! section the linker places has a VALUE counted from its base.

use std::collections::BTreeMap;
use std::fmt::{self, Write};

use crate::expr::Value;

pub const HEADER: &str = "detrytus object 1";

/// Where in a word the linker writes a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelocKind {
    /// The whole word, as `.word` stores it.
    Word,
    /// An instruction's `imm` field.
    Imm,
    /// An instruction's word offset field (`off`, `offn`, `offp`, or a
    /// `jal`'s `imm`): the value less the instruction's address, in words.
    Branch(&'static str),
    /// `lui` + `addi` (`li`): the value split over the two.
    Pair,
    /// `auipc` + `addi` (`la`): the value less the `auipc`'s address,
    /// split over the two.
    PcPair,
    /// The `imm` field of the full instruction in slot B (true) or A of
    /// the bundle: put there, and the slot encoded again.
    SlotImm(bool),
    /// A word offset field of the full instruction in a slot, counting
    /// from the bundle's address.
    SlotBranch(bool, &'static str),
}

const BRANCH_FIELDS: [&str; 4] = ["off", "offn", "offp", "imm"];

impl fmt::Display for RelocKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RelocKind::Word => write!(f, "word"),
            RelocKind::Imm => write!(f, "imm"),
            RelocKind::Branch(field) => write!(f, "branch.{field}"),
            RelocKind::Pair => write!(f, "pair"),
            RelocKind::PcPair => write!(f, "pcpair"),
            RelocKind::SlotImm(b) => write!(f, "slot.{}.imm", slot_name(*b)),
            RelocKind::SlotBranch(b, field) => {
                write!(f, "slot.{}.branch.{field}", slot_name(*b))
            }
        }
    }
}

fn slot_name(b: bool) -> &'static str {
    if b { "b" } else { "a" }
}

impl RelocKind {
    /// The kind for the same value in slot B (true) or A of a bundle.
    pub fn in_slot(self, b: bool) -> Option<Self> {
        match self {
            RelocKind::Imm => Some(RelocKind::SlotImm(b)),
            RelocKind::Branch(field) => Some(RelocKind::SlotBranch(b, field)),
            _ => None,
        }
    }

    fn parse(text: &str) -> Option<Self> {
        if let Some(rest) = text.strip_prefix("slot.") {
            let (which, inner) = rest.split_once('.')?;
            let b = match which {
                "a" => false,
                "b" => true,
                _ => return None,
            };
            return RelocKind::parse(inner)?.in_slot(b);
        }
        Some(match text {
            "word" => RelocKind::Word,
            "imm" => RelocKind::Imm,
            "pair" => RelocKind::Pair,
            "pcpair" => RelocKind::PcPair,
            _ => {
                let field = text.strip_prefix("branch.")?;
                RelocKind::Branch(BRANCH_FIELDS.into_iter().find(|f| *f == field)?)
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reloc {
    pub addr: Value,
    pub kind: RelocKind,
    pub addend: Value,
    pub plus: Option<String>,
    pub minus: Option<String>,
    pub line: usize,
    pub file: Option<String>,
}

impl Reloc {
    /// `file:line` (or `line N`), for messages.
    pub fn place(&self) -> String {
        match &self.file {
            Some(f) => format!("{f}:{}", self.line),
            None => format!("line {}", self.line),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub name: String,
    /// Load minus run address, for a section its source placed.
    pub placed: Option<Value>,
    /// Its base is a multiple of 3^align.
    pub align: u32,
    /// The first free tryte after it.
    pub end: Value,
    pub trytes: BTreeMap<i64, i64>,
    pub relocs: Vec<Reloc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub global: bool,
    /// An address, not an `.equ` value.
    pub label: bool,
    pub value: Value,
    /// The section the linker places that value counts from.
    pub section: Option<String>,
}

/// A relocatable object.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Object {
    pub sections: Vec<Section>,
    /// In the order they were defined.
    pub symbols: Vec<Symbol>,
}

/// Trytes per `data` line.
const DATA_LINE: usize = 27;

impl Object {
    pub fn to_text(&self) -> String {
        let mut out = format!("{HEADER}\n");
        for s in &self.sections {
            let _ = write!(out, "section {} {} {}", s.name, s.align, s.end);
            if let Some(shift) = s.placed {
                let _ = write!(out, " at {shift}");
            }
            out.push('\n');
            // Runs of consecutive trytes, a line at most DATA_LINE long.
            let mut run: Vec<(i64, i64)> = Vec::new();
            let flush = |run: &mut Vec<(i64, i64)>, out: &mut String| {
                if let Some(&(start, _)) = run.first() {
                    let _ = write!(out, "data {} {start}", s.name);
                    for (_, t) in run.iter() {
                        let _ = write!(out, " {t}");
                    }
                    out.push('\n');
                }
                run.clear();
            };
            for (&a, &t) in &s.trytes {
                if run.last().is_some_and(|&(last, _)| last + 1 != a) || run.len() == DATA_LINE {
                    flush(&mut run, &mut out);
                }
                run.push((a, t));
            }
            flush(&mut run, &mut out);
            for r in &s.relocs {
                let _ = write!(out, "reloc {} {} {} {}", s.name, r.addr, r.kind, r.addend);
                if let Some(p) = &r.plus {
                    let _ = write!(out, " +{p}");
                }
                if let Some(m) = &r.minus {
                    let _ = write!(out, " -{m}");
                }
                let _ = write!(out, " @ {}", r.line);
                if let Some(f) = &r.file {
                    let _ = write!(out, " {f}");
                }
                out.push('\n');
            }
        }
        for sym in &self.symbols {
            let scope = if sym.global { "global" } else { "local" };
            let what = if sym.label { "label" } else { "constant" };
            let _ = write!(out, "symbol {scope} {what} {} {}", sym.name, sym.value);
            if let Some(sec) = &sym.section {
                let _ = write!(out, " in {sec}");
            }
            out.push('\n');
        }
        out
    }

    /// Read an object's text. `shown` names it in errors.
    pub fn parse(text: &str, shown: &str) -> Result<Object, String> {
        let mut lines = text.lines().enumerate();
        match lines.next() {
            Some((_, HEADER)) => {}
            Some((_, l)) if l.starts_with("detrytus object ") => {
                return Err(format!("{shown}: object format {} is not 1", &l[16..]));
            }
            _ => return Err(format!("{shown}: not a detrytus object")),
        }
        let mut object = Object::default();
        for (i, row) in lines {
            let bad = |what: &str| format!("{shown}:{}: {what}", i + 1);
            let words: Vec<&str> = row.split_whitespace().collect();
            let num = |w: Option<&&str>| -> Result<Value, String> {
                w.and_then(|w| w.parse().ok()).ok_or_else(|| bad("expected a number"))
            };
            let section = |object: &mut Object, name: Option<&&str>| -> Result<usize, String> {
                let name = name.ok_or_else(|| bad("expected a section"))?;
                object
                    .sections
                    .iter()
                    .position(|s| s.name == *name)
                    .ok_or_else(|| bad(&format!("no section {name}")))
            };
            match words.first().copied() {
                None => {}
                Some("section") => {
                    let name = words.get(1).ok_or_else(|| bad("expected a section name"))?;
                    let align = num(words.get(2))?;
                    let end = num(words.get(3))?;
                    let placed = match (words.get(4), words.len()) {
                        (None, 4) => None,
                        (Some(&"at"), 6) => Some(num(words.get(5))?),
                        _ => return Err(bad("expected section NAME ALIGN END [at SHIFT]")),
                    };
                    if object.sections.iter().any(|s| s.name == *name) {
                        return Err(bad(&format!("section {name} given twice")));
                    }
                    object.sections.push(Section {
                        name: name.to_string(),
                        placed,
                        align: u32::try_from(align).map_err(|_| bad("bad alignment"))?,
                        end,
                        trytes: BTreeMap::new(),
                        relocs: Vec::new(),
                    });
                }
                Some("data") => {
                    let k = section(&mut object, words.get(1))?;
                    let start = num(words.get(2))?;
                    for (j, w) in words.iter().skip(3).enumerate() {
                        let t = num(Some(w))?;
                        let addr = i64::try_from(start + j as Value).map_err(|_| bad("address"))?;
                        object.sections[k].trytes.insert(addr, t as i64);
                    }
                }
                Some("reloc") => {
                    let k = section(&mut object, words.get(1))?;
                    let addr = num(words.get(2))?;
                    let kind = words
                        .get(3)
                        .and_then(|w| RelocKind::parse(w))
                        .ok_or_else(|| bad("expected a relocation kind"))?;
                    let addend = num(words.get(4))?;
                    let (mut plus, mut minus, mut j) = (None, None, 5);
                    while let Some(w) = words.get(j).filter(|w| **w != "@") {
                        match (w.strip_prefix('+'), w.strip_prefix('-')) {
                            (Some(p), _) if plus.is_none() => plus = Some(p.to_string()),
                            (_, Some(m)) if minus.is_none() => minus = Some(m.to_string()),
                            _ => return Err(bad("expected +BASE, -BASE, then @")),
                        }
                        j += 1;
                    }
                    if words.get(j) != Some(&"@") {
                        return Err(bad("expected @ LINE"));
                    }
                    let line = usize::try_from(num(words.get(j + 1))?).map_err(|_| bad("line"))?;
                    // The file is the rest of the line, spaces and all.
                    let file = line_rest(row, j + 2);
                    object.sections[k].relocs.push(Reloc {
                        addr,
                        kind,
                        addend,
                        plus,
                        minus,
                        line,
                        file,
                    });
                }
                Some("symbol") => {
                    let global = match words.get(1) {
                        Some(&"global") => true,
                        Some(&"local") => false,
                        _ => return Err(bad("expected global or local")),
                    };
                    let label = match words.get(2) {
                        Some(&"label") => true,
                        Some(&"constant") => false,
                        _ => return Err(bad("expected label or constant")),
                    };
                    let name = words.get(3).ok_or_else(|| bad("expected a name"))?;
                    let value = num(words.get(4))?;
                    let section = match (words.get(5), words.len()) {
                        (None, 5) => None,
                        (Some(&"in"), 7) => {
                            let k = section(&mut object, words.get(6))?;
                            Some(object.sections[k].name.clone())
                        }
                        _ => return Err(bad("expected symbol ... VALUE [in SECTION]")),
                    };
                    object.symbols.push(Symbol {
                        name: name.to_string(),
                        global,
                        label,
                        value,
                        section,
                    });
                }
                Some(w) => return Err(bad(&format!("unknown line {w}"))),
            }
        }
        Ok(object)
    }
}

/// What follows the first n whitespace-separated words of line, if
/// anything.
fn line_rest(line: &str, n: usize) -> Option<String> {
    let mut rest = line;
    for _ in 0..n {
        rest = rest.trim_start();
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        rest = &rest[end..];
    }
    let rest = rest.trim();
    (!rest.is_empty()).then(|| rest.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_reads_back_as_written() {
        let object = Object {
            sections: vec![
                Section {
                    name: "text".into(),
                    placed: None,
                    align: 1,
                    end: 5,
                    trytes: [(-1, 3), (0, -4), (1, 0), (3, 9)].into_iter().collect(),
                    relocs: vec![Reloc {
                        addr: 0,
                        kind: RelocKind::Branch("imm"),
                        addend: 6,
                        plus: Some("[data]".into()),
                        minus: None,
                        line: 4,
                        file: Some("dir with space/a.s".into()),
                    }],
                },
                Section {
                    name: "data".into(),
                    placed: Some(-59049),
                    align: 2,
                    end: 19700,
                    trytes: BTreeMap::new(),
                    relocs: vec![
                        Reloc {
                            addr: 19683,
                            kind: RelocKind::Word,
                            addend: -2,
                            plus: Some("ext".into()),
                            minus: Some("gp".into()),
                            line: 9,
                            file: None,
                        },
                        Reloc {
                            addr: 19686,
                            kind: RelocKind::SlotBranch(true, "imm"),
                            addend: 0,
                            plus: Some("f".into()),
                            minus: None,
                            line: 10,
                            file: None,
                        },
                    ],
                },
            ],
            symbols: vec![
                Symbol {
                    name: "start".into(),
                    global: true,
                    label: true,
                    value: 0,
                    section: Some("text".into()),
                },
                Symbol { name: "N".into(), global: false, label: false, value: 3, section: None },
            ],
        };
        let text = object.to_text();
        assert!(text.contains("data text -1 3 -4 0\ndata text 3 9\n"), "{text}");
        assert_eq!(Object::parse(&text, "a.dto"), Ok(object));
    }

    #[test]
    fn errors_name_the_line() {
        let e = Object::parse("detrytus object 1\nsection text 1\n", "a.dto").unwrap_err();
        assert_eq!(e, "a.dto:2: expected a number");
        let e = Object::parse("detrytus object 2\n", "a.dto").unwrap_err();
        assert_eq!(e, "a.dto: object format 2 is not 1");
        let e = Object::parse("hello\n", "a.dto").unwrap_err();
        assert_eq!(e, "a.dto: not a detrytus object");
    }
}
