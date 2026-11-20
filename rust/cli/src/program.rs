//! Files to a loadable program: assembly sources, `.dti` images, or `-`
//! for source on stdin, merged as the Python tools merged them.

use std::collections::{BTreeMap, HashSet};
use std::io::Read;

use detrytus_asm::{Image, assemble};
use detrytus_core::image;

use crate::{BAD_INPUT, Fail, NO_INPUT};

const MAGIC: &[u8] = b"DTI\x01";

/// What a file holds: an assembled source, or an image's bytes.
pub enum Loaded {
    Source(Image),
    Image(Vec<u8>),
}

/// A program ready to load and to name addresses in.
#[derive(Default)]
pub struct Program {
    pub trytes: BTreeMap<i64, i64>,
    /// In the order the Python image's dict keeps them: a debugger naming
    /// an address takes the first of several labels there.
    pub labels: Vec<(String, i64)>,
    pub symbols: Vec<(String, i128)>,
}

pub fn strerror(e: &std::io::Error) -> String {
    let text = e.to_string();
    text.rfind(" (os error ").map_or(text.clone(), |i| text[..i].to_string())
}

pub fn load_one(path: &str) -> Result<Loaded, Fail> {
    if path == "-" {
        let mut text = String::new();
        std::io::stdin()
            .read_to_string(&mut text)
            .map_err(|e| Fail::new(format!("<stdin>: {e}"), NO_INPUT))?;
        return assemble(&text, Some("<stdin>")).map(Loaded::Source).map_err(asm_error);
    }
    let data = std::fs::read(path)
        .map_err(|e| Fail::new(format!("{path}: {}", strerror(&e)), NO_INPUT))?;
    if data.starts_with(MAGIC) {
        image::parse(&data).map_err(|e| Fail::new(e, BAD_INPUT))?;
        return Ok(Loaded::Image(data));
    }
    let text = String::from_utf8(data)
        .map_err(|_| Fail::new(format!("{path}: not UTF-8 text"), BAD_INPUT))?;
    assemble(&text, Some(path)).map(Loaded::Source).map_err(asm_error)
}

fn asm_error(e: detrytus_asm::AsmError) -> Fail {
    Fail::new(e.to_string(), BAD_INPUT)
}

impl Loaded {
    /// The program this one file is on its own.
    fn program(&self) -> Result<Program, Fail> {
        match self {
            Loaded::Source(image) => Ok(Program {
                trytes: image.trytes.clone(),
                labels: image
                    .label_order
                    .iter()
                    .map(|n| (n.clone(), image.labels[n] as i64))
                    .collect(),
                symbols: image.symbols.iter().map(|(n, v)| (n.clone(), *v)).collect(),
            }),
            Loaded::Image(data) => {
                let segments = image::parse(data).map_err(|e| Fail::new(e, BAD_INPUT))?;
                let (labels, symbols) =
                    image::parse_meta(data).map_err(|e| Fail::new(e, BAD_INPUT))?;
                let trytes = segments
                    .iter()
                    .flat_map(|s| (s.start..).zip(s.trytes.iter().copied()))
                    .collect();
                let symbols = symbols.into_iter().map(|(n, v)| (n, v as i128)).collect();
                Ok(Program { trytes, labels, symbols })
            }
        }
    }
}

/// Set name to value in an ordered table, keeping its first position; a
/// name already there with another value is ambiguous.
fn put<V: Copy + PartialEq>(
    table: &mut Vec<(String, V)>,
    name: &str,
    value: V,
    ambiguous: &mut HashSet<String>,
) {
    match table.iter_mut().find(|(n, _)| n == name) {
        Some(slot) => {
            if slot.1 != value {
                ambiguous.insert(name.to_string());
            }
            slot.1 = value;
        }
        None => table.push((name.to_string(), value)),
    }
}

/// Several files as one program. Overlapping trytes are an error; a name
/// defined with different values in two files is dropped, so nothing
/// picks the wrong one.
pub fn load(files: &[String]) -> Result<Program, Fail> {
    let mut merged = Program::default();
    let mut ambiguous = HashSet::new();
    for file in files {
        let one = load_one(file)?.program()?;
        if let Some(addr) = one.trytes.keys().find(|a| merged.trytes.contains_key(a)) {
            return Err(Fail::new(format!("images overlap at address {addr}"), BAD_INPUT));
        }
        merged.trytes.extend(one.trytes);
        for (name, value) in &one.symbols {
            put(&mut merged.symbols, name, *value, &mut ambiguous);
        }
        for (name, value) in &one.labels {
            put(&mut merged.labels, name, *value, &mut ambiguous);
        }
    }
    merged.symbols.retain(|(n, _)| !ambiguous.contains(n));
    merged.labels.retain(|(n, _)| !ambiguous.contains(n));
    Ok(merged)
}
