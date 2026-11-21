//! Assembler expressions: integers, symbols, and + - * / % with
//! parentheses (docs/assembler.md), ported from the original Python
//! tools; comparisons, and in `.assert` `defined(NAME)`, came after.
//!
//! Number forms: decimal (42), balanced ternary (0t+0-), hexadecimal
//! (0x2a) and character literals ('a'). '.' is the current address.
//! Values are i128, so nothing an assembler meets overflows.

use std::collections::HashMap;

use crate::pyrepr::repr;
use crate::text::is_space;

pub type Value = i128;
pub type Symbols = HashMap<String, Value>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprError {
    /// A symbol the expression needs is not defined (yet).
    Undefined(String),
    Other(String),
}

impl ExprError {
    pub fn message(&self) -> &str {
        match self {
            ExprError::Undefined(m) | ExprError::Other(m) => m,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Number(Value),
    Name(String),
    Op(char),
    /// A comparison: == != < <= > >=.
    Cmp(&'static str),
}

const COMPARISONS: [&str; 6] = ["==", "!=", "<=", ">=", "<", ">"];

fn is_name_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || c == '.'
}

fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '$'
}

/// What a backslash escape in a character or string literal stands for.
/// Any other escape is refused, so a typo like `\b` is an error rather
/// than a silent `b`.
pub(crate) fn escape(c: char) -> Option<char> {
    Some(match c {
        'n' => '\n',
        't' => '\t',
        '0' => '\0',
        '\\' | '\'' | '"' => c,
        _ => return None,
    })
}

pub(crate) fn unknown_escape(c: char) -> String {
    format!("unknown escape \\{c}: the escapes are \\n \\t \\0 \\\\ \\' \\\"")
}

fn too_large(text: &str) -> ExprError {
    ExprError::Other(format!("number too large: {}", repr(text)))
}

/// One token at the start of s (whitespace already skipped): the token
/// and its length in bytes, trying the forms in the Python regex's order.
/// Every form but a character literal is ASCII, so its length in
/// characters is its length in bytes.
fn token(s: &str) -> Result<Option<(Token, usize)>, ExprError> {
    let chars: Vec<char> = s.chars().collect();
    let run = |from: usize, ok: &dyn Fn(char) -> bool| -> usize {
        chars[from..].iter().take_while(|c| ok(**c)).count()
    };
    let bytes = |n: usize| -> usize { chars[..n].iter().map(|c| c.len_utf8()).sum() };
    if s.starts_with("0t") {
        let n = run(2, &|c| matches!(c, '-' | '0' | '+'));
        if n > 0 {
            let mut v: Value = 0;
            for &c in &chars[2..2 + n] {
                let t = match c {
                    '-' => -1,
                    '+' => 1,
                    _ => 0,
                };
                v = v.checked_mul(3).and_then(|v| v.checked_add(t)).ok_or_else(|| too_large(s))?;
            }
            return Ok(Some((Token::Number(v), 2 + n)));
        }
    }
    if s.starts_with("0x") {
        let n = run(2, &|c| c.is_ascii_hexdigit());
        if n > 0 {
            let digits: String = chars[2..2 + n].iter().collect();
            let v = Value::from_str_radix(&digits, 16).map_err(|_| too_large(&digits))?;
            return Ok(Some((Token::Number(v), 2 + n)));
        }
    }
    let n = run(0, &|c| c.is_ascii_digit());
    if n > 0 {
        let digits: String = chars[..n].iter().collect();
        let v: Value = digits.parse().map_err(|_| too_large(&digits))?;
        return Ok(Some((Token::Number(v), n)));
    }
    if chars.first() == Some(&'\'') {
        // '(?P<char>[^'\\]|\\.)'
        let value = match (chars.get(1), chars.get(2), chars.get(3)) {
            (Some('\\'), Some(&e), Some('\'')) if e != '\n' => {
                Some((escape(e).ok_or_else(|| ExprError::Other(unknown_escape(e)))?, 4))
            }
            (Some(&c), Some('\''), _) if c != '\'' && c != '\\' => Some((c, 3)),
            _ => None,
        };
        if let Some((c, len)) = value {
            return Ok(Some((Token::Number(c as Value), bytes(len))));
        }
    }
    if chars.first().is_some_and(|c| is_name_start(*c)) {
        let n = 1 + run(1, &is_name_char);
        let name: String = chars[..n].iter().collect();
        return Ok(Some((Token::Name(name), n)));
    }
    if let Some(cmp) = COMPARISONS.iter().find(|c| s.starts_with(**c)) {
        return Ok(Some((Token::Cmp(cmp), cmp.len())));
    }
    if let Some(&c) = chars.first()
        && "-+*/%()".contains(c)
    {
        return Ok(Some((Token::Op(c), 1)));
    }
    Ok(None)
}

fn tokens(text: &str) -> Result<Vec<Token>, ExprError> {
    let text = text.trim_end_matches(is_space);
    let mut out = Vec::new();
    let mut pos = 0;
    while pos < text.len() {
        let skipped = text[pos..].len() - text[pos..].trim_start_matches(is_space).len();
        match token(&text[pos + skipped..])? {
            Some((tok, len)) => {
                out.push(tok);
                pos += skipped + len;
            }
            None => return Err(ExprError::Other(format!("cannot parse {}", repr(&text[pos..])))),
        }
    }
    Ok(out)
}

struct Parser<'a> {
    text: &'a str,
    toks: Vec<Token>,
    pos: usize,
    symbols: &'a Symbols,
    here: Option<Value>,
    /// Whether `defined(NAME)` may be used: only where every symbol is
    /// known, so its answer cannot depend on where it is asked.
    defined: bool,
}

impl Parser<'_> {
    fn peek_op(&self, ops: &str) -> Option<char> {
        match self.toks.get(self.pos) {
            Some(Token::Op(c)) if ops.contains(*c) => Some(*c),
            _ => None,
        }
    }

    fn take(&mut self) -> Result<Token, ExprError> {
        let tok = self.toks.get(self.pos).cloned();
        let tok =
            tok.ok_or_else(|| ExprError::Other(format!("unexpected end of {}", repr(self.text))))?;
        self.pos += 1;
        Ok(tok)
    }

    fn overflow(&self) -> ExprError {
        ExprError::Other(format!("value too large in {}", repr(self.text)))
    }

    fn parse(&mut self) -> Result<Value, ExprError> {
        if self.toks.is_empty() {
            return Err(ExprError::Other("empty expression".into()));
        }
        let v = self.comparison()?;
        if self.pos != self.toks.len() {
            return Err(ExprError::Other(format!("trailing input in {}", repr(self.text))));
        }
        Ok(v)
    }

    /// One comparison at most, below every arithmetic operator: 1 when
    /// it holds, 0 when not.
    fn comparison(&mut self) -> Result<Value, ExprError> {
        let v = self.sum()?;
        let Some(Token::Cmp(cmp)) = self.toks.get(self.pos).cloned() else { return Ok(v) };
        self.take()?;
        let rhs = self.sum()?;
        if let Some(Token::Cmp(_)) = self.toks.get(self.pos) {
            let message = format!("comparisons do not chain in {}", repr(self.text));
            return Err(ExprError::Other(message));
        }
        let holds = match cmp {
            "==" => v == rhs,
            "!=" => v != rhs,
            "<" => v < rhs,
            "<=" => v <= rhs,
            ">" => v > rhs,
            _ => v >= rhs,
        };
        Ok(holds as Value)
    }

    fn sum(&mut self) -> Result<Value, ExprError> {
        let mut v = self.product()?;
        while let Some(op) = self.peek_op("+-") {
            self.take()?;
            let rhs = self.product()?;
            let r = if op == '+' { v.checked_add(rhs) } else { v.checked_sub(rhs) };
            v = r.ok_or_else(|| self.overflow())?;
        }
        Ok(v)
    }

    fn product(&mut self) -> Result<Value, ExprError> {
        let mut v = self.atom()?;
        while let Some(op) = self.peek_op("*/%") {
            self.take()?;
            let rhs = self.atom()?;
            if op == '*' {
                v = v.checked_mul(rhs).ok_or_else(|| self.overflow())?;
                continue;
            }
            if rhs == 0 {
                return Err(ExprError::Other(format!("division by zero in {}", repr(self.text))));
            }
            let q = v / rhs; // Rust divides toward zero, as the Python helper does
            v = if op == '/' { q } else { v - q * rhs };
        }
        Ok(v)
    }

    fn atom(&mut self) -> Result<Value, ExprError> {
        match self.take()? {
            Token::Number(v) => Ok(v),
            Token::Name(name) if name == "defined" && self.peek_op("(").is_some() => {
                self.defined_call()
            }
            Token::Name(name) => self.symbol(&name),
            Token::Op('(') => {
                let v = self.comparison()?;
                if self.peek_op(")").is_none() {
                    return Err(ExprError::Other(format!("missing ')' in {}", repr(self.text))));
                }
                self.take()?;
                Ok(v)
            }
            Token::Op(c) if c == '+' || c == '-' => {
                let v = self.atom()?;
                Ok(if c == '-' { -v } else { v })
            }
            Token::Op(c) => Err(ExprError::Other(format!(
                "unexpected {} in {}",
                repr(&c.to_string()),
                repr(self.text)
            ))),
            Token::Cmp(c) => {
                Err(ExprError::Other(format!("unexpected {} in {}", repr(c), repr(self.text))))
            }
        }
    }

    /// `defined(NAME)`, the '(' next: 1 if NAME is a symbol, else 0.
    fn defined_call(&mut self) -> Result<Value, ExprError> {
        if !self.defined {
            return Err(ExprError::Other("defined() works only in .assert".into()));
        }
        self.take()?;
        let name = match (self.take()?, self.peek_op(")")) {
            (Token::Name(name), Some(_)) => name,
            _ => {
                let message = format!("defined() takes one name in {}", repr(self.text));
                return Err(ExprError::Other(message));
            }
        };
        self.take()?;
        Ok(self.symbols.contains_key(&name) as Value)
    }

    fn symbol(&self, name: &str) -> Result<Value, ExprError> {
        if name == "." {
            return self.here.ok_or_else(|| ExprError::Undefined("'.' has no address here".into()));
        }
        self.symbols
            .get(name)
            .copied()
            .ok_or_else(|| ExprError::Undefined(format!("undefined symbol {}", repr(name))))
    }
}

/// The value of text, with '.' standing for here when it is known.
pub fn evaluate(text: &str, symbols: &Symbols, here: Option<Value>) -> Result<Value, ExprError> {
    let toks = tokens(text)?;
    Parser { text, toks, pos: 0, symbols, here, defined: false }.parse()
}

/// The value of an `.assert` condition: as `evaluate`, with every
/// symbol known, so `defined(NAME)` may be asked. A check has no
/// address of its own, so '.' is not defined in one.
pub fn evaluate_check(text: &str, symbols: &Symbols) -> Result<Value, ExprError> {
    let toks = tokens(text)?;
    Parser { text, toks, pos: 0, symbols, here: None, defined: true }.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(text: &str) -> Result<Value, ExprError> {
        let symbols: Symbols = [("x".to_string(), 10)].into_iter().collect();
        evaluate(text, &symbols, Some(100))
    }

    #[test]
    fn numbers_in_every_form() {
        assert_eq!(eval("42"), Ok(42));
        assert_eq!(eval("0t+0-"), Ok(8));
        assert_eq!(eval("0x2a"), Ok(42));
        assert_eq!(eval("'a'"), Ok(97));
        assert_eq!(eval("'\\n'"), Ok(10));
        assert_eq!(eval("'é'"), Ok(233));
    }

    #[test]
    fn precedence_and_division() {
        assert_eq!(eval("1 + 2 * 3"), Ok(7));
        assert_eq!(eval("(1 + 2) * 3"), Ok(9));
        assert_eq!(eval("-7 / 2"), Ok(-3));
        assert_eq!(eval("-7 % 2"), Ok(-1));
        assert_eq!(eval("x * 2 + ."), Ok(120));
        assert_eq!(eval("- -3"), Ok(3));
    }

    #[test]
    fn errors_read_as_in_python() {
        let msg = |t: &str| eval(t).unwrap_err().message().to_string();
        assert_eq!(msg("y"), "undefined symbol 'y'");
        assert!(matches!(eval("y"), Err(ExprError::Undefined(_))));
        assert_eq!(msg("1 @ 2"), "cannot parse ' @ 2'");
        assert_eq!(msg("1 / 0"), "division by zero in '1 / 0'");
        assert_eq!(msg("(1"), "missing ')' in '(1'");
        assert_eq!(msg("1 +"), "unexpected end of '1 +'");
        assert_eq!(msg("1 2"), "trailing input in '1 2'");
        assert_eq!(msg(")"), "unexpected ')' in ')'");
        assert_eq!(msg(""), "empty expression");
        let undefined = evaluate(".", &Symbols::new(), None).unwrap_err();
        assert_eq!(undefined, ExprError::Undefined("'.' has no address here".into()));
    }

    #[test]
    fn comparisons_give_one_or_zero_below_arithmetic() {
        assert_eq!(eval("x == 10"), Ok(1));
        assert_eq!(eval("x != 10"), Ok(0));
        assert_eq!(eval("x * 2 > 19"), Ok(1));
        assert_eq!(eval("x <= 9"), Ok(0));
        assert_eq!(eval("x <= 10"), Ok(1));
        assert_eq!(eval("x < 10"), Ok(0));
        assert_eq!(eval("x > 10"), Ok(0));
        assert_eq!(eval("x >= 11"), Ok(0));
        assert_eq!(eval("x >= 10"), Ok(1));
        assert_eq!(eval("1 < 2"), Ok(1));
        assert_eq!(eval("(1 < 2) + (3 < 2)"), Ok(1));
        let msg = |t: &str| eval(t).unwrap_err().message().to_string();
        assert_eq!(msg("1 < 2 < 3"), "comparisons do not chain in '1 < 2 < 3'");
        assert_eq!(msg("== 1"), "unexpected '==' in '== 1'");
    }

    #[test]
    fn defined_asks_whether_a_symbol_exists_but_only_in_a_check() {
        let symbols: Symbols = [("x".to_string(), 10)].into_iter().collect();
        let check = |t: &str| evaluate_check(t, &symbols);
        assert_eq!(check("defined(x)"), Ok(1));
        assert_eq!(check("defined(y) == 0"), Ok(1));
        assert_eq!(check("defined(x) * x"), Ok(10));
        let msg = check("defined(1)").unwrap_err().message().to_string();
        assert_eq!(msg, "defined() takes one name in 'defined(1)'");
        let msg = eval("defined(x)").unwrap_err().message().to_string();
        assert_eq!(msg, "defined() works only in .assert");
        // Without the parenthesis it is an ordinary name.
        assert_eq!(
            check("defined"),
            Err(ExprError::Undefined("undefined symbol 'defined'".into()))
        );
    }

    #[test]
    fn fallbacks_follow_the_regex_order() {
        // "0x" without digits is 0 then the name "x"; "0t" likewise.
        assert_eq!(eval("0x"), Err(ExprError::Other("trailing input in '0x'".into())));
        assert_eq!(eval("0t"), Err(ExprError::Other("trailing input in '0t'".into())));
    }
}
