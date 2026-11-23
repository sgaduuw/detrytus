//! The Trice compiler (docs/trice.md): a whole program of Trice source,
//! with the files it imports, into one detrytus assembly file that
//! follows docs/abi.md.
//!
//! The compiler knows the ISA and the ABI but no operating system:
//! start code and system-call numbers arrive through imported assembly.

pub mod ast;
mod codegen;
pub mod lex;
pub mod parse;

use std::collections::HashSet;
use std::fmt;
use std::path::{Component, Path, PathBuf};

use ast::Item;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub file: String,
    pub line: usize,
    pub message: String,
}

impl Error {
    pub fn new(file: &str, line: usize, message: String) -> Self {
        Error { file: file.to_string(), line, message }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.line == 0 {
            write!(f, "{}: {}", self.file, self.message)
        } else {
            write!(f, "{}:{}: {}", self.file, self.line, self.message)
        }
    }
}

/// A compiled program.
#[derive(Debug)]
pub struct Output {
    pub assembly: String,
    /// `file:line: warning: ...`, in source order.
    pub warnings: Vec<String>,
}

/// One piece of the program, in the order the source gives it.
pub(crate) enum Unit {
    Item {
        file: String,
        item: Item,
    },
    /// An imported assembly file: its path as the output names it in
    /// `.include`, and the constants its `.equ`s define.
    Include {
        path: String,
        consts: Vec<(String, i128)>,
        /// The operands of its instructions, which name the functions
        /// it may call.
        operands: String,
        file: String,
        line: usize,
    },
}

/// Compile the program whose main file is `path`. `out_dir` is the
/// directory the assembly will be assembled from, which `.include`
/// paths are made relative to.
pub fn compile(path: &Path, out_dir: &Path) -> Result<Output, Error> {
    let mut loader = Loader { seen: HashSet::new(), units: Vec::new(), out_dir: absolute(out_dir) };
    loader.load(path, &path.display().to_string(), None)?;
    codegen::generate(&loader.units, &path.display().to_string())
}

struct Loader {
    seen: HashSet<PathBuf>,
    units: Vec<Unit>,
    out_dir: PathBuf,
}

impl Loader {
    fn read(path: &Path, shown: &str, from: Option<(&str, usize)>) -> Result<String, Error> {
        std::fs::read_to_string(path).map_err(|e| {
            let msg = format!("cannot read {shown}: {}", e.kind().to_string().to_lowercase());
            match from {
                Some((file, line)) => Error::new(file, line, msg),
                None => Error::new(shown, 0, msg),
            }
        })
    }

    fn load(&mut self, path: &Path, shown: &str, from: Option<(&str, usize)>) -> Result<(), Error> {
        let src = Self::read(path, shown, from)?;
        self.seen.insert(absolute(path));
        let toks = lex::tokens(&src, shown)?;
        let items = parse::Parser::new(toks, shown).program()?;
        let dir = path.parent().unwrap_or(Path::new("")).to_path_buf();
        for item in items {
            if let Item::Import(rel, line) = &item {
                let target = dir.join(rel);
                let target_shown = target.display().to_string();
                if !self.seen.insert(absolute(&target)) {
                    continue;
                }
                if rel.ends_with(".tc") {
                    self.seen.remove(&absolute(&target));
                    self.load(&target, &target_shown, Some((shown, *line)))?;
                } else if rel.ends_with(".s") {
                    let text = Self::read(&target, &target_shown, Some((shown, *line)))?;
                    let (consts, operands) =
                        equates(&text, &target_shown).map_err(|m| Error::new(shown, *line, m))?;
                    let path = relative(&self.out_dir, &absolute(&target));
                    self.units.push(Unit::Include {
                        path,
                        consts,
                        operands,
                        file: shown.into(),
                        line: *line,
                    });
                } else {
                    return Err(Error::new(shown, *line, "import a .tc or a .s file".into()));
                }
                continue;
            }
            self.units.push(Unit::Item { file: shown.to_string(), item });
        }
        Ok(())
    }
}

/// The constants an assembly file's `.equ` and `.set` lines define, in
/// order, with what its includes define, and the operands of its
/// instructions. A constant whose value needs a label is left out: only
/// the assembler knows it.
#[allow(clippy::type_complexity)]
fn equates(text: &str, shown: &str) -> Result<(Vec<(String, i128)>, String), String> {
    let statements = detrytus_asm::source::parse(text, Some(shown)).map_err(|e| e.to_string())?;
    let mut symbols = detrytus_asm::expr::Symbols::new();
    let mut out = Vec::new();
    let mut operands = String::new();
    for s in statements {
        if !s.mnemonic.starts_with('.') {
            operands.push_str(&s.operands.join(" "));
            operands.push('\n');
        }
        if (s.mnemonic == ".equ" || s.mnemonic == ".set")
            && s.operands.len() == 2
            && let Ok(v) = detrytus_asm::expr::evaluate(&s.operands[1], &symbols, None)
        {
            symbols.insert(s.operands[0].clone(), v);
            out.push((s.operands[0].clone(), v));
        }
    }
    Ok((out, operands))
}

fn absolute(p: &Path) -> PathBuf {
    let joined = if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(p)
    };
    std::fs::canonicalize(&joined).unwrap_or_else(|_| normalise(&joined))
}

fn normalise(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// `to` as a path relative to the directory `from`, both absolute.
fn relative(from: &Path, to: &Path) -> String {
    let a: Vec<_> = from.components().collect();
    let b: Vec<_> = to.components().collect();
    let common = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    if common == 0 {
        return to.display().to_string();
    }
    let mut out = PathBuf::new();
    for _ in common..a.len() {
        out.push("..");
    }
    for c in &b[common..] {
        out.push(c.as_os_str());
    }
    out.display().to_string()
}
