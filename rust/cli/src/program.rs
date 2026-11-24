//! Files to a loadable program: assembly sources, `.dti` images, or `-`
//! for source on stdin, merged as the Python tools merged them, and
//! objects (docs/linker.md), linked together first.

use std::collections::{BTreeMap, HashSet};
use std::io::Read;

use detrytus_asm::object::Object;
use detrytus_asm::{Image, assemble, assemble_object};
use detrytus_core::image;

use crate::{BAD_INPUT, Fail, NO_INPUT};

const MAGIC: &[u8] = b"DTI\x01";

/// What a file holds: an assembled source, an image's bytes, or an
/// object.
pub enum Loaded {
    Source(Image),
    Image(Vec<u8>),
    Object(Object),
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

/// A file's text, or stdin's for `-`, with the name messages give it;
/// or an image's bytes.
enum Contents {
    Text(String, String),
    Image(Vec<u8>),
}

fn read(path: &str) -> Result<Contents, Fail> {
    if path == "-" {
        let mut text = String::new();
        std::io::stdin()
            .read_to_string(&mut text)
            .map_err(|e| Fail::new(format!("<stdin>: {e}"), NO_INPUT))?;
        return Ok(Contents::Text(text, "<stdin>".into()));
    }
    let data = std::fs::read(path)
        .map_err(|e| Fail::new(format!("{path}: {}", strerror(&e)), NO_INPUT))?;
    if data.starts_with(MAGIC) {
        image::parse(&data).map_err(|e| Fail::new(e, BAD_INPUT))?;
        return Ok(Contents::Image(data));
    }
    let text = String::from_utf8(data)
        .map_err(|_| Fail::new(format!("{path}: not UTF-8 text"), BAD_INPUT))?;
    Ok(Contents::Text(text, path.to_string()))
}

fn is_object(text: &str) -> bool {
    text.starts_with("detrytus object ")
}

pub fn load_one(path: &str) -> Result<Loaded, Fail> {
    match read(path)? {
        Contents::Image(data) => Ok(Loaded::Image(data)),
        Contents::Text(text, shown) if is_object(&text) => {
            Object::parse(&text, &shown).map(Loaded::Object).map_err(|e| Fail::new(e, BAD_INPUT))
        }
        Contents::Text(text, shown) => {
            assemble(&text, Some(&shown)).map(Loaded::Source).map_err(asm_error)
        }
    }
}

/// A file as an object: an object file read, or a source assembled (or
/// a Trice file compiled, `tc -c`) as one, with its listing.
pub fn load_object(path: &str) -> Result<(Object, Option<Image>), Fail> {
    if path.ends_with(".tc") {
        // Compiled as if to the .s beside it, so its .include paths and
        // messages read the same.
        let file = std::path::Path::new(path);
        let dir = file.parent().filter(|d| !d.as_os_str().is_empty());
        let dir = dir.unwrap_or(std::path::Path::new("."));
        let out = detrytus_tc::compile_unit(file, dir)
            .map_err(|e| Fail::new(e.to_string(), BAD_INPUT))?;
        for w in &out.warnings {
            eprintln!("{w}");
        }
        let shown = format!("{}.s", path.trim_end_matches(".tc"));
        let (o, image) = assemble_object(&out.assembly, Some(&shown)).map_err(asm_error)?;
        return Ok((o, Some(image)));
    }
    match read(path)? {
        Contents::Image(_) => Err(Fail::new(
            format!("{path}: an image cannot be linked, only objects and sources"),
            BAD_INPUT,
        )),
        Contents::Text(text, shown) if is_object(&text) => {
            let o = Object::parse(&text, &shown).map_err(|e| Fail::new(e, BAD_INPUT))?;
            Ok((o, None))
        }
        Contents::Text(text, shown) => {
            let (o, image) = assemble_object(&text, Some(&shown)).map_err(asm_error)?;
            Ok((o, Some(image)))
        }
    }
}

/// Link objects, each with the name its messages use.
pub fn link(objects: &[(String, Object)]) -> Result<detrytus_asm::link::Linked, Fail> {
    detrytus_asm::link::link(objects).map_err(|e| Fail::new(e, BAD_INPUT))
}

fn asm_error(e: detrytus_asm::AsmError) -> Fail {
    Fail::new(e.to_string(), BAD_INPUT)
}

impl Program {
    fn of(image: &Image) -> Program {
        Program {
            trytes: image.trytes.clone(),
            labels: image.label_order.iter().map(|n| (n.clone(), image.labels[n] as i64)).collect(),
            symbols: image.symbols.iter().map(|(n, v)| (n.clone(), *v)).collect(),
        }
    }
}

impl Loaded {
    /// The program this one file is on its own.
    fn program(&self) -> Result<Program, Fail> {
        match self {
            Loaded::Source(image) => Ok(Program::of(image)),
            Loaded::Object(_) => unreachable!("objects are linked before they are loaded"),
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
    let mut loaded = Vec::new();
    let mut objects = Vec::new();
    for file in files {
        match load_one(file)? {
            // The objects, linked, stand where the first of them did.
            Loaded::Object(o) => {
                if objects.is_empty() {
                    loaded.push(None);
                }
                objects.push((file.clone(), o));
            }
            other => loaded.push(Some(other.program()?)),
        }
    }
    let mut linked = (!objects.is_empty()).then(|| link(&objects)).transpose()?;
    let mut merged = Program::default();
    let mut ambiguous = HashSet::new();
    for one in loaded {
        let one = match one {
            Some(one) => one,
            None => Program::of(&linked.take().expect("linked once").image),
        };
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
