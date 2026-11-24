//! The linker (docs/linker.md): objects to one image.
//!
//! Sections of the same name from every object become one section.
//! The object whose source gave that section an address puts its part
//! there; the others' parts follow it, in the order the objects are
//! given. A section no object places follows the section before it,
//! and the first one starts so that its first word is centred on 0, as
//! the assembler starts. Then every global symbol has its address, and
//! each relocation is written into the word it names.
//!
//! A part made by `.part` is left out when nothing kept uses it: every
//! other part is kept, and so is any part a kept part's relocations
//! name, directly or through a global symbol.

use std::collections::{BTreeMap, HashMap, HashSet};

use detrytus_core::isa;
use detrytus_core::trits::{TRYTE_TRITS, WORD_TRITS, field, half, low, pow3, split_high};

use crate::assembler::Image;
use crate::expr::Value;
use crate::object::{Object, Reloc, RelocKind};

/// Where one object's part of a section went.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    pub object: String,
    /// The first name defined in it, if any.
    pub label: Option<String>,
    /// Its base: where its addresses count from (0 for the part its
    /// source placed, whose addresses are already where it runs).
    pub base: Value,
    /// Its first and last-plus-one trytes, as run addresses.
    pub start: Value,
    pub end: Value,
}

/// A section of the linked program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placed {
    pub name: String,
    /// Load address minus run address.
    pub shift: Value,
    pub parts: Vec<Part>,
}

/// A linked program: its image, and where everything went.
#[derive(Debug, Clone, Default)]
pub struct Linked {
    pub image: Image,
    pub sections: Vec<Placed>,
    /// The global symbols, by name, with the object defining each.
    pub globals: BTreeMap<String, (Value, String)>,
    /// The parts left out because nothing uses them: (object, the first
    /// name defined in the part, or the part's own name).
    pub dropped: Vec<(String, String)>,
}

/// The section a part belongs to: `text:3` is part of `text`.
pub fn section_of(part: &str) -> &str {
    part.split(':').next().unwrap_or(part)
}

/// The parts to link, by (object, index in its sections): every part
/// not made by `.part`, and whatever kept parts use.
fn used_parts(objects: &[(String, Object)]) -> HashSet<(usize, usize)> {
    let mut defined: HashMap<&str, (usize, usize)> = HashMap::new();
    for (k, (_, o)) in objects.iter().enumerate() {
        for sym in o.symbols.iter().filter(|s| s.global) {
            let part =
                sym.section.as_deref().and_then(|p| o.sections.iter().position(|s| s.name == p));
            if let Some(j) = part {
                defined.entry(&sym.name).or_insert((k, j));
            }
        }
    }
    let mut work: Vec<(usize, usize)> = Vec::new();
    for (k, (_, o)) in objects.iter().enumerate() {
        for (j, s) in o.sections.iter().enumerate() {
            if !s.name.contains(':') {
                work.push((k, j));
            }
        }
    }
    let mut kept = HashSet::new();
    while let Some((k, j)) = work.pop() {
        if !kept.insert((k, j)) {
            continue;
        }
        let o = &objects[k].1;
        for r in &o.sections[j].relocs {
            for base in [&r.plus, &r.minus].into_iter().flatten() {
                let to = match base.strip_prefix('[').and_then(|b| b.strip_suffix(']')) {
                    Some(sec) => o.sections.iter().position(|s| s.name == sec).map(|j| (k, j)),
                    None => defined.get(base.as_str()).copied(),
                };
                work.extend(to);
            }
        }
    }
    kept
}

/// The first multiple of unit at which a word can be centred without
/// covering anything before loc, the next free tryte.
fn next_centre(loc: Value, unit: Value) -> Value {
    -((-(loc + 1)).div_euclid(unit)) * unit
}

/// Link objects, each with the name its messages use.
pub fn link(objects: &[(String, Object)]) -> Result<Linked, String> {
    let mut linked = Linked::default();
    let kept = used_parts(objects);
    // The first name defined in each part, for the map.
    let label = |k: usize, part: &str| -> Option<String> {
        let o = &objects[k].1;
        o.symbols
            .iter()
            .filter(|s| s.section.as_deref() == Some(part))
            .min_by_key(|s| s.value)
            .map(|s| s.name.clone())
    };
    for (k, (shown, o)) in objects.iter().enumerate() {
        for (j, s) in o.sections.iter().enumerate() {
            if !kept.contains(&(k, j)) {
                let name = label(k, &s.name).unwrap_or_else(|| s.name.clone());
                linked.dropped.push((shown.clone(), name));
            }
        }
    }
    let is_kept = |k: usize, part: &str| {
        objects[k]
            .1
            .sections
            .iter()
            .position(|s| s.name == part)
            .is_some_and(|j| kept.contains(&(k, j)))
    };
    // Where each object's parts went: (object, part) -> (base, shift).
    let mut bases: HashMap<(usize, &str), (Value, Value)> = HashMap::new();
    let mut order: Vec<&str> = Vec::new();
    for (k, (_, o)) in objects.iter().enumerate() {
        for s in o.sections.iter().filter(|s| is_kept(k, &s.name)) {
            if !order.contains(&section_of(&s.name)) {
                order.push(section_of(&s.name));
            }
        }
    }
    let mut cursor: Value = -1;
    for name in order {
        let parts: Vec<(usize, &crate::object::Section)> = objects
            .iter()
            .enumerate()
            .flat_map(|(k, (_, o))| {
                o.sections
                    .iter()
                    .filter(move |s| section_of(&s.name) == name && is_kept(k, &s.name))
                    .map(move |s| (k, s))
            })
            .collect();
        let placed: Vec<_> = parts.iter().filter(|(_, s)| s.placed.is_some()).collect();
        if let [a, b, ..] = placed[..] {
            return Err(format!(
                "section {name} is placed by both {} and {}",
                objects[a.0].0, objects[b.0].0
            ));
        }
        let mut out = Placed { name: name.to_string(), shift: 0, parts: Vec::new() };
        if let Some(&&(k, s)) = placed.first() {
            out.shift = s.placed.unwrap_or(0);
            let start = s.trytes.keys().next().map_or(s.end, |&a| a as Value);
            bases.insert((k, &s.name), (0, out.shift));
            let label = label(k, &s.name);
            let object = objects[k].0.clone();
            out.parts.push(Part { object, label, base: 0, start, end: s.end });
            cursor = s.end;
        }
        for &(k, s) in parts.iter().filter(|(_, s)| s.placed.is_none()) {
            let base = next_centre(cursor, pow3(s.align) as Value);
            bases.insert((k, &s.name), (base, out.shift));
            let part = Part {
                object: objects[k].0.clone(),
                label: label(k, &s.name),
                base,
                start: base - 1,
                end: base + s.end,
            };
            cursor = part.end;
            out.parts.push(part);
        }
        linked.sections.push(out);
    }

    // Global symbols.
    for (k, (shown, o)) in objects.iter().enumerate() {
        for sym in o.symbols.iter().filter(|s| s.global) {
            if sym.section.as_deref().is_some_and(|p| !is_kept(k, p)) {
                continue;
            }
            let value = sym.value + section_base(&bases, k, sym.section.as_deref(), shown)?;
            if let Some((_, other)) = linked.globals.get(&sym.name) {
                return Err(format!("symbol {} is defined in both {other} and {shown}", sym.name));
            }
            linked.globals.insert(sym.name.clone(), (value, shown.clone()));
        }
    }

    // Trytes, at their load addresses.
    let image = &mut linked.image;
    for (k, (shown, o)) in objects.iter().enumerate() {
        for s in o.sections.iter().filter(|s| is_kept(k, &s.name)) {
            let (base, shift) = bases[&(k, s.name.as_str())];
            for (&a, &t) in &s.trytes {
                let at = base + a as Value + shift;
                let key = i64::try_from(at).map_err(|_| format!("address {at} is out of range"))?;
                if image.trytes.insert(key, t).is_some() {
                    return Err(format!(
                        "{shown}: section {} lands on address {at}, which is already taken",
                        s.name
                    ));
                }
            }
        }
    }

    // Relocations.
    for (k, (shown, o)) in objects.iter().enumerate() {
        for s in o.sections.iter().filter(|s| is_kept(k, &s.name)) {
            let (base, shift) = bases[&(k, s.name.as_str())];
            for r in &s.relocs {
                let resolve = |b: &Option<String>| -> Result<Value, String> {
                    let Some(b) = b else { return Ok(0) };
                    if let Some(sec) = b.strip_prefix('[').and_then(|b| b.strip_suffix(']')) {
                        return section_base(&bases, k, Some(sec), shown);
                    }
                    linked.globals.get(b).map(|(v, _)| *v).ok_or_else(|| {
                        format!("{}: undefined symbol {}", r.place(), crate::pyrepr::repr(b))
                    })
                };
                let value = r.addend + resolve(&r.plus)? - resolve(&r.minus)?;
                let at = base + r.addr;
                apply(&mut linked.image.trytes, r, value, at, shift)?;
            }
        }
    }

    // Names for the debugger: every global, and each local whose name no
    // global has and no other object uses for another value.
    let mut order: Vec<(&str, bool)> = Vec::new();
    let mut locals: HashMap<&str, Value> = HashMap::new();
    let mut ambiguous: HashSet<&str> = HashSet::new();
    for (k, (shown, o)) in objects.iter().enumerate() {
        for sym in &o.symbols {
            if sym.section.as_deref().is_some_and(|p| !is_kept(k, p)) {
                continue;
            }
            if !order.iter().any(|(n, _)| *n == sym.name) {
                order.push((&sym.name, sym.label));
            }
            if sym.global {
                continue;
            }
            let v = sym.value + section_base(&bases, k, sym.section.as_deref(), shown)?;
            if *locals.entry(&sym.name).or_insert(v) != v {
                ambiguous.insert(&sym.name);
            }
        }
    }
    let image = &mut linked.image;
    for (name, label) in order {
        let value = match linked.globals.get(name) {
            Some(&(v, _)) => v,
            None if ambiguous.contains(name) => continue,
            None => locals[name],
        };
        image.symbols.insert(name.to_string(), value);
        if label {
            image.labels.insert(name.to_string(), value);
            image.label_order.push(name.to_string());
        }
    }
    Ok(linked)
}

/// Where an object's section went (0 for no section: a constant).
fn section_base(
    bases: &HashMap<(usize, &str), (Value, Value)>,
    k: usize,
    section: Option<&str>,
    shown: &str,
) -> Result<Value, String> {
    let Some(sec) = section else { return Ok(0) };
    bases.get(&(k, sec)).map(|b| b.0).ok_or_else(|| format!("{shown}: no section {sec}"))
}

/// The word whose centre is at (run address), with shift to its load
/// address.
fn read(trytes: &BTreeMap<i64, i64>, at: Value, shift: Value) -> Option<i64> {
    let t = |d: Value| trytes.get(&i64::try_from(at + shift + d).ok()?).copied();
    let (lo, mid, hi) = (t(-1)?, t(0)?, t(1)?);
    Some(lo + mid * pow3(TRYTE_TRITS) + hi * pow3(2 * TRYTE_TRITS))
}

fn write(trytes: &mut BTreeMap<i64, i64>, at: Value, shift: Value, word: i64) {
    let lo = low(word, TRYTE_TRITS);
    let rest = split_high(word, TRYTE_TRITS);
    for (d, t) in [(-1, lo), (0, low(rest, TRYTE_TRITS)), (1, split_high(rest, TRYTE_TRITS))] {
        trytes.insert((at + shift + d) as i64, t);
    }
}

/// Put value into the named field of the instruction at `at`.
fn set_field(
    trytes: &mut BTreeMap<i64, i64>,
    r: &Reloc,
    at: Value,
    shift: Value,
    name: &str,
    value: Value,
) -> Result<(), String> {
    let place = r.place();
    let word = read(trytes, at, shift).ok_or_else(|| format!("{place}: no word at {at}"))?;
    let op = isa::decode(word).map_err(|_| format!("{place}: no instruction at {at}"))?.op;
    let &(_, lo, width) = isa::format_fields(op.format)
        .iter()
        .find(|(n, _, _)| *n == name)
        .ok_or_else(|| format!("{place}: {} has no field {name}", op.name))?;
    let max = half(width) as Value;
    if value.abs() > max {
        return Err(format!("{place}: {value} does not fit {}'s {width}-trit {name}", op.name));
    }
    let old = field(word, lo, width);
    write(trytes, at, shift, word + (value as i64 - old) * pow3(lo));
    Ok(())
}

fn apply(
    trytes: &mut BTreeMap<i64, i64>,
    r: &Reloc,
    value: Value,
    at: Value,
    shift: Value,
) -> Result<(), String> {
    match r.kind {
        RelocKind::Word => {
            // As .word stores it: the low 27 trits.
            let m = pow3(WORD_TRITS) as Value;
            let word = (value + (m - 1) / 2).rem_euclid(m) - (m - 1) / 2;
            write(trytes, at, shift, word as i64);
            Ok(())
        }
        RelocKind::Imm => set_field(trytes, r, at, shift, "imm", value),
        RelocKind::Branch(name) => {
            let delta = value - at;
            if delta % 3 != 0 {
                return Err(format!("{}: branch target {value} is not word aligned", r.place()));
            }
            set_field(trytes, r, at, shift, name, delta / 3)
        }
        RelocKind::Pair | RelocKind::PcPair => {
            let v = if r.kind == RelocKind::PcPair { value - at } else { value };
            let (hi, lo) = crate::pseudo::split_constant(v);
            set_field(trytes, r, at, shift, "imm", hi)?;
            set_field(trytes, r, at + 3, shift, "imm", lo)
        }
    }
}
