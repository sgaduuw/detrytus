//! `detrytus link`: objects and sources (assembled as objects) into one
//! image (docs/linker.md), or a map of where everything went.

use std::io::Write;

use crate::program::{link, load_object, strerror};
use crate::{Fail, NO_INPUT, usage};

pub(crate) fn cmd_link(args: impl Iterator<Item = String>) -> Result<u8, Fail> {
    let (mut files, mut output, mut map) = (Vec::new(), None, false);
    let mut args = args;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" | "--output" => {
                output = Some(args.next().ok_or_else(|| usage("-o needs a file"))?)
            }
            "--map" => map = true,
            flag if flag.starts_with('-') && flag != "-" => {
                return Err(usage(&format!("unknown option {flag}")));
            }
            _ => files.push(arg),
        }
    }
    if files.is_empty() {
        return Err(usage("link needs a file"));
    }
    let mut objects = Vec::new();
    for file in &files {
        objects.push((file.clone(), load_object(file)?.0));
    }
    let linked = link(&objects)?;
    if let Some(out) = &output {
        std::fs::write(out, linked.image.to_bytes())
            .map_err(|e| Fail::new(format!("{out}: {}", strerror(&e)), NO_INPUT))?;
        if !map {
            return Ok(0);
        }
    }
    let mut stdout = std::io::stdout().lock();
    for s in &linked.sections {
        let _ = write!(stdout, "section {}", s.name);
        if s.shift != 0 {
            let _ = write!(stdout, ", loaded {:+}", s.shift);
        }
        let _ = writeln!(stdout);
        for p in &s.parts {
            let label = p.label.as_deref().map(|l| format!(" {l}")).unwrap_or_default();
            let _ = writeln!(stdout, "  {:>12} .. {:<12}  {}{label}", p.start, p.end, p.object);
        }
    }
    if !linked.dropped.is_empty() {
        let _ = writeln!(stdout, "left out, unused");
        for (object, name) in &linked.dropped {
            let _ = writeln!(stdout, "  {object} {name}");
        }
    }
    let _ = writeln!(stdout, "globals");
    for (name, (value, object)) in &linked.globals {
        let _ = writeln!(stdout, "  {value:>12}  {name}  ({object})");
    }
    Ok(0)
}
