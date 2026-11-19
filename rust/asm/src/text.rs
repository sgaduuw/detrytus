//! Python's string handling where the assembler relies on it, so lines,
//! words and whitespace split the same way in both assemblers.

/// Python's str.isspace, which also counts the ASCII separators
/// U+001C to U+001F.
pub fn is_space(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

/// Python's str.strip().
pub fn strip(s: &str) -> &str {
    s.trim_matches(is_space)
}

/// Python's str.splitlines(): lines without their endings.
pub fn split_lines(text: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        let ends = matches!(
            c,
            '\n' | '\r'
                | '\u{b}'
                | '\u{c}'
                | '\u{1c}'
                | '\u{1d}'
                | '\u{1e}'
                | '\u{85}'
                | '\u{2028}'
                | '\u{2029}'
        );
        if ends {
            lines.push(&text[start..i]);
            let mut next = i + c.len_utf8();
            if c == '\r' && chars.peek().is_some_and(|&(_, n)| n == '\n') {
                chars.next();
                next += 1;
            }
            start = next;
        }
    }
    if start < text.len() {
        lines.push(&text[start..]);
    }
    lines
}

/// Python's text.split(None, 1): the first word and the rest, stripped
/// on the left.
pub fn split_first_word(text: &str) -> (&str, Option<&str>) {
    let text = text.trim_start_matches(is_space);
    match text.find(is_space) {
        None => (text, None),
        Some(i) => {
            let rest = text[i..].trim_start_matches(is_space);
            (&text[..i], (!rest.is_empty()).then_some(rest))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_split_as_python_splitlines() {
        assert_eq!(split_lines("a\nb\r\nc\rd"), vec!["a", "b", "c", "d"]);
        assert_eq!(split_lines("a\n"), vec!["a"]);
        assert_eq!(split_lines("a\n\nb"), vec!["a", "", "b"]);
        assert_eq!(split_lines(""), Vec::<&str>::new());
        assert_eq!(split_lines("a\u{2028}b"), vec!["a", "b"]);
    }

    #[test]
    fn first_word() {
        assert_eq!(split_first_word("  add a0, a1 "), ("add", Some("a0, a1 ")));
        assert_eq!(split_first_word("nop"), ("nop", None));
    }
}
