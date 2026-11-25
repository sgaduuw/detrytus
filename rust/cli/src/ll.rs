//! `detrytus ll`: translate LLVM IR, as rustc writes it, into assembly
//! or straight to an image (docs/rust.md).

use std::path::Path;

use crate::{BAD_INPUT, Fail, NO_INPUT, usage};

pub(crate) fn cmd_ll(args: impl Iterator<Item = String>) -> Result<u8, Fail> {
    let (mut files, mut output, mut includes, mut prune) = (Vec::new(), None, Vec::new(), false);
    let mut roots = Vec::new();
    let mut args = args;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" | "--output" => {
                output = Some(args.next().ok_or_else(|| usage("-o needs a file"))?)
            }
            "--include" => {
                includes.push(args.next().ok_or_else(|| usage("--include needs a file"))?)
            }
            "--prune" => prune = true,
            "--root" => roots.push(args.next().ok_or_else(|| usage("--root needs a name"))?),
            flag if flag.starts_with('-') => {
                return Err(usage(&format!("unknown option {flag}")));
            }
            _ => files.push(arg),
        }
    }
    if files.is_empty() {
        return Err(usage("ll needs a file"));
    }
    let paths: Vec<&Path> = files.iter().map(Path::new).collect();
    let write = |path: &str, bytes: &[u8]| {
        std::fs::write(path, bytes)
            .map_err(|e| Fail::new(format!("{path}: {}", crate::program::strerror(&e)), NO_INPUT))
    };
    if prune {
        if roots.is_empty() {
            roots.push("main".to_string());
        }
        let roots: Vec<&str> = roots.iter().map(String::as_str).collect();
        let text = detrytus_tc::ll::prune(&paths, &roots)
            .map_err(|e| Fail::new(e.to_string(), BAD_INPUT))?;
        match output {
            None => print!("{text}"),
            Some(o) => write(&o, text.as_bytes())?,
        }
        return Ok(0);
    }
    let out = detrytus_tc::ll::translate(&paths, &includes)
        .map_err(|e| Fail::new(e.to_string(), BAD_INPUT))?;
    match output {
        None => print!("{}", out.assembly),
        Some(o) if o.ends_with(".dti") => {
            let shown = format!("{}.s", o.trim_end_matches(".dti"));
            let image = detrytus_asm::assemble(&out.assembly, Some(&shown))
                .map_err(|e| Fail::new(e.to_string(), BAD_INPUT))?;
            write(&o, &image.to_bytes())?;
        }
        Some(o) => write(&o, out.assembly.as_bytes())?,
    }
    Ok(0)
}
