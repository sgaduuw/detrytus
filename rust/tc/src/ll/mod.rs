//! `detrytus ll`: LLVM IR, as rustc writes it for a 32-bit target, into
//! detrytus assembly through the same back end as Trice (docs/rust.md).
//!
//! The whole program is translated at once: the modules given (a
//! program's crate and `core`, say) are read together, and only what
//! `main` reaches is lowered, so core's many functions cost nothing
//! unless used. Names no module defines are left to the assembly the
//! output includes: the start code, `memcpy` and friends, system calls.

mod ir;
mod lower;

use std::path::Path;

use crate::{Error, Output};

fn read(paths: &[&Path]) -> Result<(ir::Module, String), Error> {
    let mut m = ir::Module::default();
    let mut shown = Vec::new();
    for (k, p) in paths.iter().enumerate() {
        let name = p.display().to_string();
        let text = std::fs::read_to_string(p).map_err(|e| {
            Error::new(&name, 0, format!("cannot read it: {}", e.kind().to_string().to_lowercase()))
        })?;
        ir::parse(&text, &name, k, &mut m)?;
        shown.push(name);
    }
    Ok((m, shown.join(", ")))
}

/// Translate the LLVM IR modules at `paths` into one assembly file that
/// includes `includes` first (paths as `.include` should name them).
pub fn translate(paths: &[&Path], includes: &[String]) -> Result<Output, Error> {
    let (m, shown) = read(paths)?;
    translate_module(&m, &shown, includes)
}

/// The part of the modules at `paths` that `main` reaches, as one LLVM
/// IR module: what a test needs of core, without the rest of it.
pub fn prune(paths: &[&Path]) -> Result<String, Error> {
    let (m, shown) = read(paths)?;
    if !m.funcs.contains_key("main") {
        return Err(Error::new(&shown, 0, "no function main".into()));
    }
    let low = lower::Lower::new(&m);
    let (funcs, globals, external) = low.reachable(&["main"]);
    let mut body = Vec::new();
    for g in &globals {
        body.push(m.source[&('g', g.clone())].clone());
    }
    for f in &funcs {
        body.push(String::new());
        body.push(m.source[&('f', f.clone())].clone());
    }
    let mut text = body.join("\n");
    // The aliases the text names.
    let mut aliases: Vec<&String> = m.aliases.keys().collect();
    aliases.sort();
    let mut lines = Vec::new();
    for a in aliases {
        let at = format!("@{a}");
        if text.match_indices(&at).any(|(i, _)| {
            !text[i + at.len()..].starts_with(|c: char| c.is_alphanumeric() || "._$-".contains(c))
        }) {
            lines.push(m.source[&('a', a.clone())].clone());
        }
    }
    if !lines.is_empty() {
        text = format!("{}\n{text}", lines.join("\n"));
    }
    // Declarations of what is used but not defined, intrinsics included.
    let mut decls: Vec<&String> = m
        .decls
        .iter()
        .filter(|d| {
            external.contains_key(*d)
                || (d.starts_with("llvm.") && text.contains(&format!("@{d}(")))
        })
        .collect();
    decls.sort();
    let mut tail = vec![String::new()];
    for d in decls {
        tail.push(m.source[&('d', d.clone())].clone());
    }
    // The named types the text uses, and the types they use.
    let mut names: Vec<&String> = m.types.keys().collect();
    names.sort();
    let mut types: Vec<String> = Vec::new();
    let mut scan = text.clone() + &tail.join("\n");
    loop {
        let before = types.len();
        for n in &names {
            let line = &m.source[&('t', (*n).clone())];
            let quoted = format!("%\"{n}\"");
            let bare = format!("%{n}");
            let used = scan.contains(&quoted)
                || scan.match_indices(&bare).any(|(i, _)| {
                    !scan[i + bare.len()..]
                        .starts_with(|c: char| c.is_alphanumeric() || "._$-".contains(c))
                });
            if used && !types.contains(line) {
                types.push(line.clone());
                scan.push_str(line);
            }
        }
        if types.len() == before {
            break;
        }
    }
    types.sort();
    text = format!(
        "; Pruned by detrytus ll from {shown}: what main reaches.\n{}\n{text}\n{}\n",
        types.join("\n"),
        tail.join("\n")
    );
    Ok(text)
}

/// Translate LLVM IR given as text, for tests.
pub fn translate_text(text: &str, includes: &[String]) -> Result<Output, Error> {
    let mut m = ir::Module::default();
    ir::parse(text, "input.ll", 0, &mut m)?;
    translate_module(&m, "input.ll", includes)
}

fn translate_module(m: &ir::Module, shown: &str, includes: &[String]) -> Result<Output, Error> {
    if !m.funcs.contains_key("main") {
        return Err(Error::new(shown, 0, "no function main".into()));
    }
    let mut low = lower::Lower::new(m);
    let (funcs, globals, external) = low.reachable(&["main"]);
    let muls = funcs
        .iter()
        .flat_map(|f| &m.funcs[f].blocks)
        .flat_map(|b| &b.instrs)
        .filter(|i| {
            matches!(&i.op, ir::Op::Bin { op, ty: ir::Ty::Int(64), a, b, .. }
                if op == "mul" && matches!(a, ir::Val::Local(_)) && matches!(b, ir::Val::Local(_)))
        })
        .count();
    low.mul_helper = muls > 1;
    let mut out =
        vec![format!("# Translated by detrytus ll from {shown}: docs/rust.md, docs/abi.md.")];
    for i in includes {
        out.push(format!("        .include \"{i}\""));
    }
    if !external.is_empty() {
        let names: Vec<&str> = external.keys().map(String::as_str).collect();
        out.push(format!("# From elsewhere: {}.", names.join(", ")));
    }
    out.push("        .compress".into());
    for f in &funcs {
        out.extend(low.function(&m.funcs[f])?);
    }
    out.push(String::new());
    // The 64-bit helper functions used, and those they use.
    if !low.wide_calls.is_empty() {
        let mut hm = ir::Module::default();
        ir::parse(lower::WIDE_IR, "lower/wide.ll", 0, &mut hm)?;
        let roots: Vec<&str> = low.wide_calls.iter().copied().collect();
        let mut wl = lower::Lower::new(&hm);
        wl.inline_wide = true;
        let (hfuncs, _, _) = wl.reachable(&roots);
        for f in &hfuncs {
            out.extend(wl.function(&hm.funcs[f])?);
        }
        low.helpers.extend(wl.helpers.iter().copied());
    }
    out.extend(lower::helpers(&low.helpers));
    out.push(String::new());
    for g in &globals {
        out.extend(low.global(&m.globals[g])?);
    }
    out.push("        .nocompress".into());
    let mut assembly = out.join("\n");
    assembly.push('\n');
    Ok(Output { assembly, warnings: Vec::new() })
}
