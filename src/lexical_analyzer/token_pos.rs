use serde::Deserialize;
use std::fmt;

// where a token starts and how many characters it spans.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
pub struct TokenPosition {
    pub line: u32,
    pub column: u32,
    pub length: u32,
}

// "[line:first-last]" with inclusive columns, or "[line:column]" when the
// span is at most one character wide; 1-based as editors display them, while
// the fields are 0-based like LSP positions
impl fmt::Display for TokenPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}:{}", self.line + 1, self.column + 1)?;
        if self.length > 1 {
            write!(f, "-{}", self.column + self.length)?;
        }
        write!(f, "]")
    }
}
