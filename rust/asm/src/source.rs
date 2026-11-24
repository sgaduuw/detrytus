//! Source text to statements: comments, labels, includes, operands.
//! Ported from the original Python tools.
//!
//! Two transformations happen here, before layout sees anything:
//! .include splices another file's statements in place, and numeric
//! local labels (1:, referenced as 1b or 1f) are renamed to unique names.

use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

use crate::error::AsmError;
use crate::text::{is_space, split_first_word, split_lines, strip};

const TEXT_DIRECTIVES: [&str; 3] = [".string", ".ascii", ".include"];
const MAX_INCLUDE_DEPTH: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Word,
    Tryte,
    /// Slot B of a bundle: emitted with the statement in slot A.
    SlotB,
    None,
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub line: usize,
    pub source: String,
    pub labels: Vec<String>,
    /// Lower case; empty for a line with only labels.
    pub mnemonic: String,
    pub operands: Vec<String>,
    pub file: Option<String>,
    // Filled in by layout:
    pub addr: i128,
    /// Words, or trytes for tryte data.
    pub size: i128,
    pub kind: Kind,
    /// Load address minus run address, from its section.
    pub shift: i128,
    /// For a `c.` instruction in slot A, the statement in slot B.
    pub partner: Option<usize>,
    /// Set when `.gp` made this an offset from gp (gp.rs).
    pub gp: Option<crate::gp::Reach>,
    /// The section it was laid out in.
    pub section: String,
}

impl Statement {
    fn new(
        line: usize,
        source: &str,
        labels: Vec<String>,
        mnemonic: String,
        operands: Vec<String>,
        file: Option<&str>,
    ) -> Self {
        Statement {
            line,
            source: source.to_string(),
            labels,
            mnemonic,
            operands,
            file: file.map(str::to_string),
            addr: 0,
            size: 0,
            kind: Kind::Word,
            shift: 0,
            partner: None,
            gp: None,
            section: String::new(),
        }
    }

    pub fn error(&self, message: impl Into<String>) -> AsmError {
        AsmError::new(message, self.line, &self.source, self.file.as_deref())
    }
}

/// Drop a '#' or ';' comment, ignoring those inside quotes.
pub fn strip_comment(line: &str) -> &str {
    let mut quote: Option<char> = None;
    let mut prev = '\0';
    for (i, c) in line.char_indices() {
        match quote {
            Some(q) => {
                if c == q && prev != '\\' {
                    quote = None;
                }
            }
            None if c == '"' || c == '\'' => quote = Some(c),
            None if c == '#' || c == ';' => return &line[..i],
            None => {}
        }
        prev = c;
    }
    line
}

/// Split on commas outside parentheses and quotes.
pub fn split_operands(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    let mut prev = '\0';
    for c in text.chars() {
        let mut keep = true;
        match quote {
            Some(q) => {
                if c == q && prev != '\\' {
                    quote = None;
                }
            }
            None => match c {
                '"' | '\'' => quote = Some(c),
                '(' => depth += 1,
                ')' => depth -= 1,
                ',' if depth == 0 => {
                    out.push(strip(&cur).to_string());
                    cur.clear();
                    keep = false;
                }
                _ => {}
            },
        }
        if keep {
            cur.push(c);
        }
        prev = c;
    }
    if !strip(&cur).is_empty() {
        out.push(strip(&cur).to_string());
    }
    out
}

/// A leading label, as `\s*(\d+|[A-Za-z_.][A-Za-z0-9_.$]*)\s*:`: the
/// label and the rest of the text.
fn leading_label(text: &str) -> Option<(String, &str)> {
    let t = text.trim_start_matches(is_space);
    let first = t.chars().next()?;
    let len = if first.is_ascii_digit() {
        t.chars().take_while(char::is_ascii_digit).count()
    } else if first.is_ascii_alphabetic() || first == '_' || first == '.' {
        1 + t[1..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '$'))
            .count()
    } else {
        return None;
    };
    let rest = t[len..].trim_start_matches(is_space);
    let rest = rest.strip_prefix(':')?;
    Some((t[..len].to_string(), rest))
}

fn statement(n: usize, raw: &str, file: Option<&str>) -> Option<Statement> {
    let mut text = strip_comment(raw);
    let mut labels = Vec::new();
    while let Some((label, rest)) = leading_label(text) {
        labels.push(label);
        text = rest;
    }
    let text = strip(text);
    if text.is_empty() {
        return (!labels.is_empty())
            .then(|| Statement::new(n, raw, labels, String::new(), Vec::new(), file));
    }
    let (mnemonic, rest) = split_first_word(text);
    let operands = rest.map(split_operands).unwrap_or_default();
    Some(Statement::new(n, raw, labels, mnemonic.to_lowercase(), operands, file))
}

/// Python's Path.resolve(): absolute, symbolic links followed where the
/// path exists, `..` folded where it does not.
pub fn resolve(path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(path)
    };
    if let Ok(real) = absolute.canonicalize() {
        return real;
    }
    // Resolve the longest prefix that exists, then fold the rest.
    let parts: Vec<Component> = absolute.components().collect();
    for keep in (1..parts.len()).rev() {
        let prefix: PathBuf = parts[..keep].iter().collect();
        if let Ok(mut real) = prefix.canonicalize() {
            for part in &parts[keep..] {
                match part {
                    Component::ParentDir => {
                        real.pop();
                    }
                    Component::CurDir => {}
                    other => real.push(other.as_os_str()),
                }
            }
            return real;
        }
    }
    absolute
}

fn include_path(s: &Statement, base: &Path) -> Result<PathBuf, AsmError> {
    let ok = s.operands.len() == 1 && {
        let op = &s.operands[0];
        op.len() >= 3
            && op.starts_with('"')
            && op.ends_with('"')
            && !op[1..op.len() - 1].contains('"')
    };
    if !ok {
        return Err(s.error(".include takes one \"path\""));
    }
    let op = &s.operands[0];
    Ok(resolve(&base.join(&op[1..op.len() - 1])))
}

/// An OS error as Python's strerror gives it: without Rust's
/// "(os error N)" suffix.
pub fn strerror(e: &std::io::Error) -> String {
    let text = e.to_string();
    match text.rfind(" (os error ") {
        Some(i) => text[..i].to_string(),
        None => text,
    }
}

fn parse_into(
    out: &mut Vec<Statement>,
    text: &str,
    file: Option<&str>,
    base: &Path,
    stack: &[PathBuf],
) -> Result<(), AsmError> {
    for (i, raw) in split_lines(text).into_iter().enumerate() {
        let n = i + 1;
        let Some(s) = statement(n, raw, file) else { continue };
        if s.mnemonic != ".include" {
            out.push(s);
            continue;
        }
        if !s.labels.is_empty() {
            // Labels on an include line name what follows it.
            out.push(Statement::new(n, raw, s.labels.clone(), String::new(), Vec::new(), file));
        }
        let path = include_path(&s, base)?;
        if stack.contains(&path) {
            let name =
                path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            return Err(s.error(format!("{name} includes itself")));
        }
        if stack.len() >= MAX_INCLUDE_DEPTH {
            return Err(s.error("includes nested too deeply"));
        }
        let included = std::fs::read_to_string(&path)
            .map_err(|e| s.error(format!("cannot include {}: {}", path.display(), strerror(&e))))?;
        let mut deeper = stack.to_vec();
        deeper.push(path.clone());
        let parent = path.parent().unwrap_or(Path::new("/")).to_path_buf();
        parse_into(out, &included, Some(&path.display().to_string()), &parent, &deeper)?;
    }
    Ok(())
}

type LocalDefs = HashMap<String, Vec<(usize, String)>>;

/// Rename each numeric label uniquely; return where each one is defined.
fn number_local_labels(stmts: &mut [Statement]) -> LocalDefs {
    let mut defs: LocalDefs = HashMap::new();
    for (i, s) in stmts.iter_mut().enumerate() {
        for label in &mut s.labels {
            if !label.is_empty() && label.chars().all(|c| c.is_ascii_digit()) {
                let occurrences = defs.entry(label.clone()).or_default();
                let unique = format!(".L{label}.{}", occurrences.len());
                occurrences.push((i, unique.clone()));
                *label = unique;
            }
        }
    }
    defs
}

/// 1b is the nearest definition of 1 at or before statement i (a label
/// on the same line counts), 1f the nearest one after it.
fn resolve_local(
    defs: &LocalDefs,
    i: usize,
    s: &Statement,
    number: &str,
    back: bool,
) -> Result<String, AsmError> {
    let candidates = defs.get(number).map(Vec::as_slice).unwrap_or_default();
    let found = if back {
        candidates.iter().rev().find(|(at, _)| *at <= i)
    } else {
        candidates.iter().find(|(at, _)| *at > i)
    };
    found.map(|(_, u)| u.clone()).ok_or_else(|| {
        let where_ = if back { "before" } else { "after" };
        s.error(format!("no local label {number} {where_} this line"))
    })
}

fn word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Replace 1b and 1f in an operand, as the regex
/// `(?<![\w.$'])(\d+)([bf])(?![\w.$])` does: a digit run that starts
/// after no name character or quote, followed by b or f and then by no
/// name character.
fn replace_local_refs(
    op: &str,
    mut resolve: impl FnMut(&str, bool) -> Result<String, AsmError>,
) -> Result<String, AsmError> {
    let chars: Vec<char> = op.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let starts = c.is_ascii_digit()
            && (i == 0 || !(word_char(chars[i - 1]) || matches!(chars[i - 1], '.' | '$' | '\'')));
        if !starts {
            out.push(c);
            i += 1;
            continue;
        }
        let end = i + chars[i..].iter().take_while(|c| c.is_ascii_digit()).count();
        let direction = chars.get(end).copied();
        let after = chars.get(end + 1).copied();
        let free_after = after.is_none_or(|a| !(word_char(a) || matches!(a, '.' | '$')));
        if matches!(direction, Some('b' | 'f')) && free_after {
            let number: String = chars[i..end].iter().collect();
            out.push_str(&resolve(&number, direction == Some('b'))?);
            i = end + 1;
        } else {
            out.extend(&chars[i..end]);
            i = end;
        }
    }
    Ok(out)
}

/// Give each numeric label a unique name and point 1b/1f at it.
fn rename_local_labels(stmts: &mut [Statement]) -> Result<(), AsmError> {
    let defs = number_local_labels(stmts);
    for (i, s) in stmts.iter_mut().enumerate() {
        if TEXT_DIRECTIVES.contains(&s.mnemonic.as_str()) {
            continue;
        }
        let mut renamed = Vec::with_capacity(s.operands.len());
        for op in &s.operands {
            // A quoted operand, such as an .assert message, is text.
            if op.starts_with('"') {
                renamed.push(op.clone());
                continue;
            }
            renamed.push(replace_local_refs(op, |number, back| {
                resolve_local(&defs, i, s, number, back)
            })?);
        }
        s.operands = renamed;
    }
    Ok(())
}

/// Statements of source. file names it in errors; includes are found
/// relative to its directory, or the current directory without one.
pub fn parse(source: &str, file: Option<&str>) -> Result<Vec<Statement>, AsmError> {
    let (base, stack) = match file {
        Some(f) => {
            let path = resolve(Path::new(f));
            (path.parent().unwrap_or(Path::new("/")).to_path_buf(), vec![path])
        }
        None => (std::env::current_dir().unwrap_or_default(), Vec::new()),
    };
    let mut stmts = Vec::new();
    parse_into(&mut stmts, source, file, &base, &stack)?;
    rename_local_labels(&mut stmts)?;
    Ok(stmts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comments_respect_quotes() {
        assert_eq!(strip_comment("li a0, 1 # one"), "li a0, 1 ");
        assert_eq!(strip_comment(".string \"a#b\" ; c"), ".string \"a#b\" ");
        assert_eq!(strip_comment("li a0, '#'"), "li a0, '#'");
    }

    #[test]
    fn operands_split_outside_parentheses_and_quotes() {
        assert_eq!(split_operands("a0, 3(sp)"), vec!["a0", "3(sp)"]);
        assert_eq!(split_operands("\"a, b\", ','"), vec!["\"a, b\"", "','"]);
        assert_eq!(split_operands("a, , b"), vec!["a", "", "b"]);
        assert_eq!(split_operands("a,"), vec!["a"]);
    }

    #[test]
    fn statements_take_labels_and_lower_case_mnemonics() {
        let stmts = parse("x: 1: ADD a0, a1, a2\nloop:\n  # nothing\n", None).unwrap();
        assert_eq!(stmts.len(), 2);
        assert_eq!(stmts[0].labels, vec!["x", ".L1.0"]);
        assert_eq!(stmts[0].mnemonic, "add");
        assert_eq!(stmts[1].labels, vec!["loop"]);
    }

    #[test]
    fn local_references_follow_the_regex() {
        let stmts = parse("1: j 1b\nj 1f\n1: li a0, 0x1b\nj 2f", None);
        assert_eq!(stmts.unwrap_err().message, "no local label 2 after this line");
        let stmts = parse("1: j 1b\n j 1f\n1: li a0, 0x1b\n.word '1'", None).unwrap();
        assert_eq!(stmts[0].operands, vec![".L1.0"]);
        assert_eq!(stmts[1].operands, vec![".L1.1"]);
        assert_eq!(stmts[2].operands, vec!["a0", "0x1b"]);
    }
}
