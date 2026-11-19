//! The assembler's error, formatted as the Python assembler's AsmError.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsmError {
    pub message: String,
    pub line: Option<usize>,
    pub file: Option<String>,
    /// The source line, shown under the message.
    pub source: String,
}

impl AsmError {
    pub fn new(message: impl Into<String>, line: usize, source: &str, file: Option<&str>) -> Self {
        AsmError {
            message: message.into(),
            line: Some(line),
            file: file.map(str::to_string),
            source: source.to_string(),
        }
    }
}

impl fmt::Display for AsmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.file, self.line) {
            (Some(file), Some(line)) => write!(f, "{file}:{line}: ")?,
            (None, Some(line)) => write!(f, "line {line}: ")?,
            (Some(file), None) => write!(f, "{file}: ")?,
            (None, None) => {}
        }
        write!(f, "{}", self.message)?;
        if !self.source.is_empty() {
            write!(f, "\n    {}", crate::text::strip(&self.source))?;
        }
        Ok(())
    }
}

impl std::error::Error for AsmError {}
