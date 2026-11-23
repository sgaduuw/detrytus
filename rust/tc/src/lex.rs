//! Trice tokens (docs/trice.md section 2).

use crate::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tok {
    Ident(String),
    /// An integer or character literal.
    Int(i128),
    /// A string literal's code points, without the terminating 0.
    Str(Vec<i128>),
    Punct(&'static str),
    /// The text between the braces of `asm { ... }`.
    Asm(String),
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub tok: Tok,
    pub line: usize,
}

const PUNCTS: [&str; 46] = [
    "<<=", ">>=", "<=>", "<<", ">>", "<=", ">=", "==", "!=", "&&", "||", "++", "--", "+=", "-=",
    "*=", "/=", "%=", "&=", "|=", "->", "+", "-", "*", "/", "%", "<", ">", "=", "!", "&", "|", "?",
    ":", ";", ",", ".", "(", ")", "{", "}", "[", "]", "~", "^", "#",
];

/// The largest value a word holds, H27.
pub const WORD_MAX: i128 = (7_625_597_484_987 - 1) / 2;
/// The largest code point a tryte holds, H9.
pub const TRYTE_MAX: i128 = (19_683 - 1) / 2;

pub fn tokens(src: &str, file: &str) -> Result<Vec<Token>, Error> {
    let chars: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    let mut line = 1;
    let err = |line: usize, msg: String| Error::new(file, line, msg);
    while i < chars.len() {
        let c = chars[i];
        if c == '\n' {
            line += 1;
            i += 1;
        } else if c.is_whitespace() {
            i += 1;
        } else if c == '/' && chars.get(i + 1) == Some(&'/') {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
        } else if c == '/' && chars.get(i + 1) == Some(&'*') {
            let start = line;
            i += 2;
            loop {
                match chars.get(i) {
                    None => return Err(err(start, "comment not closed".into())),
                    Some('*') if chars.get(i + 1) == Some(&'/') => {
                        i += 2;
                        break;
                    }
                    Some('\n') => line += 1,
                    _ => {}
                }
                i += 1;
            }
        } else if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            if word == "asm" {
                let mut j = i;
                while j < chars.len() && chars[j].is_whitespace() && chars[j] != '\n' {
                    j += 1;
                }
                if chars.get(j) == Some(&'{') {
                    let begin = line;
                    let mut k = j + 1;
                    while k < chars.len() && chars[k] != '}' {
                        k += 1;
                    }
                    if k == chars.len() {
                        return Err(err(begin, "asm block not closed".into()));
                    }
                    let text: String = chars[j + 1..k].iter().collect();
                    out.push(Token { tok: Tok::Asm(text.clone()), line: begin });
                    line += text.matches('\n').count();
                    i = k + 1;
                    continue;
                }
            }
            out.push(Token { tok: Tok::Ident(word), line });
        } else if c.is_ascii_digit() {
            let (value, next) = number(&chars, i).map_err(|m| err(line, m))?;
            if value.abs() > WORD_MAX {
                return Err(err(line, "literal does not fit a word".into()));
            }
            out.push(Token { tok: Tok::Int(value), line });
            i = next;
        } else if c == '\'' {
            let (cp, next) = escaped(&chars, i + 1).map_err(|m| err(line, m))?;
            if chars.get(next) != Some(&'\'') {
                return Err(err(line, "character literal not closed".into()));
            }
            check_code_point(cp).map_err(|m| err(line, m))?;
            out.push(Token { tok: Tok::Int(cp), line });
            i = next + 1;
        } else if c == '"' {
            let mut text = Vec::new();
            i += 1;
            loop {
                match chars.get(i) {
                    None | Some('\n') => return Err(err(line, "string not closed".into())),
                    Some('"') => break,
                    _ => {
                        let (cp, next) = escaped(&chars, i).map_err(|m| err(line, m))?;
                        check_code_point(cp).map_err(|m| err(line, m))?;
                        text.push(cp);
                        i = next;
                    }
                }
            }
            out.push(Token { tok: Tok::Str(text), line });
            i += 1;
        } else {
            let rest: String = chars[i..chars.len().min(i + 3)].iter().collect();
            let Some(p) = PUNCTS.iter().find(|p| rest.starts_with(**p)) else {
                return Err(err(line, format!("unexpected character {c:?}")));
            };
            out.push(Token { tok: Tok::Punct(p), line });
            i += p.chars().count();
        }
    }
    out.push(Token { tok: Tok::Eof, line });
    Ok(out)
}

fn check_code_point(cp: i128) -> Result<(), String> {
    if cp > TRYTE_MAX {
        return Err(format!("code point {cp} does not fit a tryte"));
    }
    Ok(())
}

fn number(chars: &[char], start: usize) -> Result<(i128, usize), String> {
    let mut i = start;
    let prefix = |c: char| chars.get(start + 1).is_some_and(|x| x.eq_ignore_ascii_case(&c));
    let value: i128 = if chars[start] == '0' && prefix('t') {
        i += 2;
        let mut v: i128 = 0;
        let digits = i;
        while let Some(d) = chars.get(i).and_then(|c| match c {
            '+' => Some(1),
            '0' => Some(0),
            '-' => Some(-1),
            _ => None,
        }) {
            v = v.checked_mul(3).and_then(|v| v.checked_add(d)).ok_or("literal too large")?;
            i += 1;
        }
        if i == digits {
            return Err("0t needs trits".into());
        }
        v
    } else if chars[start] == '0' && prefix('x') {
        i += 2;
        let digits = i;
        while chars.get(i).is_some_and(|c| c.is_ascii_hexdigit()) {
            i += 1;
        }
        let text: String = chars[digits..i].iter().collect();
        i128::from_str_radix(&text, 16).map_err(|_| "bad hexadecimal literal")?
    } else {
        while chars.get(i).is_some_and(|c| c.is_ascii_digit()) {
            i += 1;
        }
        let text: String = chars[start..i].iter().collect();
        text.parse().map_err(|_| "literal too large")?
    };
    if chars.get(i).is_some_and(|c| c.is_ascii_alphanumeric() || *c == '_') {
        return Err("bad number".into());
    }
    Ok((value, i))
}

/// One character of a literal at `i`, with the assembler's escapes.
fn escaped(chars: &[char], i: usize) -> Result<(i128, usize), String> {
    match chars.get(i) {
        None => Err("literal not closed".into()),
        Some('\\') => {
            let v = match chars.get(i + 1) {
                Some('n') => '\n',
                Some('t') => '\t',
                Some('0') => '\0',
                Some('\\') => '\\',
                Some('\'') => '\'',
                Some('"') => '"',
                Some(other) => return Err(format!("unknown escape \\{other}")),
                None => return Err("literal not closed".into()),
            };
            Ok((v as i128, i + 2))
        }
        Some(c) => Ok((*c as i128, i + 1)),
    }
}
