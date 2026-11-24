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

use detrytus_core::trits::{TRYTE_TRITS, WORD_TRITS, field, half, low, pow3, split_high};
use detrytus_core::{compressed, isa};

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
    /// Where each object's kept sections went, by index: `[NAME]` -> base.
    pub bases: Vec<HashMap<String, Value>>,
    /// Instructions in slots that no longer fit one with the value the
    /// linker put in: (object index, its relocation). link() refuses
    /// them; relaxing assembles again without them.
    pub unfit: Vec<(usize, Reloc)>,
}

/// Something to link: an object, or a source the linker assembles
/// itself, so that it can relax it.
#[derive(Debug, Clone)]
pub enum Input {
    Object(Object),
    Source { text: String, file: String },
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
    let linked = link_unchecked(objects)?;
    if let Some((_, r)) = linked.unfit.first() {
        return Err(format!("{}: the instruction in slot no longer fits one", r.place()));
    }
    Ok(linked)
}

/// Link objects and sources, relaxing the sources (docs/linker.md):
/// each is assembled as an object and linked; then assembled again with
/// where that link put things, so automatic pairing can put in a slot an
/// instruction the linker finishes, and linked again. An instruction
/// whose slot the link finds it no longer fits is kept out of slots and
/// the sources assembled again, until every slot fits. Objects given
/// ready-made are linked as they are.
pub fn link_inputs(inputs: &[(String, Input)]) -> Result<Linked, String> {
    use crate::assembler::{Relax, assemble_object, assemble_object_relaxed};
    let build = |relax: Option<&[Relax]>| -> Result<Vec<(String, Object)>, String> {
        inputs
            .iter()
            .enumerate()
            .map(|(k, (shown, input))| {
                let o = match (input, relax) {
                    (Input::Object(o), _) => o.clone(),
                    (Input::Source { text, file }, None) => {
                        assemble_object(text, Some(file)).map_err(|e| e.to_string())?.0
                    }
                    (Input::Source { text, file }, Some(r)) => {
                        assemble_object_relaxed(text, Some(file), &r[k])
                            .map_err(|e| e.to_string())?
                            .0
                    }
                };
                Ok((shown.clone(), o))
            })
            .collect()
    };
    let mut linked = link(&build(None)?)?;
    if !inputs.iter().any(|(_, i)| matches!(i, Input::Source { .. })) {
        return Ok(linked);
    }
    let mut relax: Vec<Relax> = vec![Relax::default(); inputs.len()];
    // Each round either succeeds or keeps one more instruction out of
    // slots, so this ends; a success with new addresses tries again, in
    // case they let more in, while the program keeps shrinking.
    loop {
        for (k, r) in relax.iter_mut().enumerate() {
            r.hints = linked.bases[k].clone();
            r.hints.extend(linked.globals.iter().map(|(n, (v, _))| (n.clone(), *v)));
        }
        let again = link_unchecked(&build(Some(&relax))?)?;
        if again.unfit.is_empty() {
            if again.image.trytes.len() >= linked.image.trytes.len() {
                return Ok(linked);
            }
            linked = again;
            continue;
        }
        for (k, r) in again.unfit {
            relax[k].excluded.insert((r.file, r.line));
        }
    }
}

/// Link objects, noting instead of refusing slots that no longer fit.
pub fn link_unchecked(objects: &[(String, Object)]) -> Result<Linked, String> {
    let mut linked = Linked { bases: vec![HashMap::new(); objects.len()], ..Default::default() };
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

    for (&(k, name), &(base, _)) in &bases {
        linked.bases[k].insert(format!("[{name}]"), base);
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
                if !apply(&mut linked.image.trytes, r, value, at, shift)? {
                    linked.unfit.push((k, r.clone()));
                }
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
    let word = with_field(word, r, at, name, value)?;
    write(trytes, at, shift, word);
    Ok(())
}

/// The instruction word with value in its named field.
fn with_field(word: i64, r: &Reloc, at: Value, name: &str, value: Value) -> Result<i64, String> {
    let place = r.place();
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
    Ok(word + (value as i64 - old) * pow3(lo))
}

/// A word offset from at to value, for a branch field.
fn word_offset(r: &Reloc, value: Value, at: Value) -> Result<Value, String> {
    let delta = value - at;
    if delta % 3 != 0 {
        return Err(format!("{}: branch target {value} is not word aligned", r.place()));
    }
    Ok(delta / 3)
}

/// Put value in the full instruction of a bundle's slot, and encode it
/// in the slot again: false when it no longer fits one.
fn set_slot(
    trytes: &mut BTreeMap<i64, i64>,
    r: &Reloc,
    at: Value,
    shift: Value,
    b: bool,
    name: &str,
    value: Value,
) -> Result<bool, String> {
    let place = r.place();
    let word = read(trytes, at, shift).ok_or_else(|| format!("{place}: no word at {at}"))?;
    let [a, bb] = compressed::slots(word);
    let slot = if b { bb } else { a };
    let full = compressed::expand(slot).ok_or_else(|| format!("{place}: no slot at {at}"))?;
    // A value beyond the full instruction's field cannot fit a slot.
    let Ok(full) = with_field(full, r, at, name, value) else { return Ok(false) };
    let d = isa::decode(full).map_err(|_| format!("{place}: no instruction at {at}"))?;
    let Some(slot) = compressed::compress(&d) else { return Ok(false) };
    let word = if b { compressed::bundle(a, slot) } else { compressed::bundle(slot, bb) };
    write(trytes, at, shift, word);
    Ok(true)
}

/// Write a relocation: false when it is in a slot it no longer fits.
fn apply(
    trytes: &mut BTreeMap<i64, i64>,
    r: &Reloc,
    value: Value,
    at: Value,
    shift: Value,
) -> Result<bool, String> {
    match r.kind {
        RelocKind::SlotImm(b) => set_slot(trytes, r, at, shift, b, "imm", value),
        RelocKind::SlotBranch(b, name) => {
            let offset = word_offset(r, value, at)?;
            set_slot(trytes, r, at, shift, b, name, offset)
        }
        _ => apply_word(trytes, r, value, at, shift).map(|()| true),
    }
}

fn apply_word(
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
            let offset = word_offset(r, value, at)?;
            set_field(trytes, r, at, shift, name, offset)
        }
        RelocKind::Pair | RelocKind::PcPair => {
            let v = if r.kind == RelocKind::PcPair { value - at } else { value };
            let (hi, lo) = crate::pseudo::split_constant(v);
            set_field(trytes, r, at, shift, "imm", hi)?;
            set_field(trytes, r, at + 3, shift, "imm", lo)
        }
        RelocKind::SlotImm(_) | RelocKind::SlotBranch(..) => unreachable!("handled by apply"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembler::{Relax, assemble_object, assemble_object_relaxed};

    const MAIN: &str = "        .compress\n        call  f\n        addi  a0, a0, 1\n";
    // f lies farther than a slot's jal reaches: 3,280 words.
    const FAR: &str = "        .zero 20000\n        .global f\nf:      ret\n";

    #[test]
    fn a_slot_that_no_longer_fits_is_found() {
        // Told that f is near, the assembler puts the call in a slot.
        let relax = Relax { hints: [("f".to_string(), 30)].into(), ..Default::default() };
        let (main, _) = assemble_object_relaxed(MAIN, Some("main.s"), &relax).unwrap();
        let kinds: Vec<_> = main.sections[0].relocs.iter().map(|r| r.kind).collect();
        assert_eq!(kinds, [RelocKind::SlotBranch(false, "imm")]);
        let (far, _) = assemble_object(FAR, Some("far.s")).unwrap();
        let objects = [("main.s".to_string(), main), ("far.s".to_string(), far)];
        let linked = link_unchecked(&objects).unwrap();
        assert_eq!(linked.unfit.len(), 1);
        assert_eq!(linked.unfit[0].1.line, 2);
        let e = link(&objects).unwrap_err();
        assert_eq!(e, "main.s:2: the instruction in slot no longer fits one");
    }

    #[test]
    fn relaxing_keeps_what_does_not_fit_out_of_slots() {
        let source = |text: &str, file: &str| Input::Source {
            text: text.to_string(),
            file: file.to_string(),
        };
        let inputs = [
            ("main.s".to_string(), source(MAIN, "main.s")),
            ("far.s".to_string(), source(FAR, "far.s")),
        ];
        let linked = link_inputs(&inputs).unwrap();
        assert!(linked.unfit.is_empty());
        // The call stays a whole word: f is at 6 + 20000 + 1.
        assert_eq!(linked.globals["f"].0, 20007);
    }
}
