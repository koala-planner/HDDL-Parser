use super::*;
use std::fmt;

#[derive(Debug)]
pub struct SyntacticError {
    pub expected: String,
    pub found: String,
    pub position: TokenPosition,
}

impl fmt::Display for SyntacticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "line {}: expected '{}', but found '{}'",
            self.position.line, self.expected, self.found
        )
    }
}
