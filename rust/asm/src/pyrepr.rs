//! Python's `repr()` of a string, for error messages that must read
//! exactly as the Python assembler's do.

fn printable(c: char) -> bool {
    // Python's str.isprintable, closely enough for source text: no
    // control characters, no separators but the space, no common format
    // characters.
    !(c.is_control()
        || (c.is_whitespace() && c != ' ')
        || matches!(c, '\u{ad}' | '\u{200b}'..='\u{200f}' | '\u{2060}'..='\u{2064}' | '\u{feff}'))
}

pub fn repr(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') { '"' } else { '\'' };
    let mut out = String::from(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ if c == quote => {
                out.push('\\');
                out.push(c);
            }
            _ if printable(c) => out.push(c),
            _ if (c as u32) < 0x100 => out.push_str(&format!("\\x{:02x}", c as u32)),
            _ if (c as u32) < 0x10000 => out.push_str(&format!("\\u{:04x}", c as u32)),
            _ => out.push_str(&format!("\\U{:08x}", c as u32)),
        }
    }
    out.push(quote);
    out
}

#[cfg(test)]
mod tests {
    use super::repr;

    #[test]
    fn quotes_and_escapes_as_python_does() {
        assert_eq!(repr("abc"), "'abc'");
        assert_eq!(repr("it's"), "\"it's\"");
        assert_eq!(repr("'\""), "'\\'\"'");
        assert_eq!(repr("a\\b\n\x01"), "'a\\\\b\\n\\x01'");
        assert_eq!(repr("é"), "'é'");
    }
}
