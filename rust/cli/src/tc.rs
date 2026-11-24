//! `detrytus tc`: compile a Trice program (docs/trice.md) to assembly,
//! or straight to an image; with `-c`, one file of it, to assembly or
//! an object for the linker (docs/linker.md).

use std::path::Path;

use crate::{BAD_INPUT, Fail, NO_INPUT, usage};

pub(crate) fn cmd_tc(args: impl Iterator<Item = String>) -> Result<u8, Fail> {
    let (mut file, mut output, mut unit) = (None, None, false);
    let mut args = args;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" | "--output" => {
                output = Some(args.next().ok_or_else(|| usage("-o needs a file"))?)
            }
            "-c" => unit = true,
            flag if flag.starts_with('-') => {
                return Err(usage(&format!("unknown option {flag}")));
            }
            _ if file.is_some() => return Err(usage("tc takes one file")),
            _ => file = Some(arg),
        }
    }
    let file = file.ok_or_else(|| usage("tc needs a file"))?;
    let out_dir = match output.as_deref().map(Path::new).and_then(Path::parent) {
        Some(dir) if !dir.as_os_str().is_empty() => dir.to_path_buf(),
        _ => Path::new(".").to_path_buf(),
    };
    let compile = if unit { detrytus_tc::compile_unit } else { detrytus_tc::compile };
    let compiled =
        compile(Path::new(&file), &out_dir).map_err(|e| Fail::new(e.to_string(), BAD_INPUT))?;
    for w in &compiled.warnings {
        eprintln!("{w}");
    }
    let write = |path: &str, bytes: &[u8]| {
        std::fs::write(path, bytes)
            .map_err(|e| Fail::new(format!("{path}: {}", crate::program::strerror(&e)), NO_INPUT))
    };
    match output {
        None => print!("{}", compiled.assembly),
        Some(out) if unit && out.ends_with(".dto") => {
            let shown = format!("{}.s", out.trim_end_matches(".dto"));
            let (object, _) = detrytus_asm::assemble_object(&compiled.assembly, Some(&shown))
                .map_err(|e| Fail::new(e.to_string(), BAD_INPUT))?;
            write(&out, object.to_text().as_bytes())?;
        }
        Some(out) if unit && out.ends_with(".dti") => {
            return Err(usage("tc -c makes an object (.dto) or assembly, not an image"));
        }
        Some(out) if out.ends_with(".dti") => {
            // Assembled as if it were the .s beside the image, so its
            // .include paths and messages read the same.
            let shown = format!("{}.s", out.trim_end_matches(".dti"));
            let image = detrytus_asm::assemble(&compiled.assembly, Some(&shown))
                .map_err(|e| Fail::new(e.to_string(), BAD_INPUT))?;
            write(&out, &image.to_bytes())?;
        }
        Some(out) => write(&out, compiled.assembly.as_bytes())?,
    }
    Ok(0)
}
