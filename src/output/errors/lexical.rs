use crate::TokenPosition;
use std::fmt;

#[derive(Debug)]
pub enum LexicalErrorType {
    InvalidIdentifier,
    InvalidRealNumber,
    InvalidKeyword,
}

#[derive(Debug)]
pub struct LexicalError {
    pub error_type: LexicalErrorType,
    pub lexeme: String,
    pub position: TokenPosition,
}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.error_type {
            LexicalErrorType::InvalidIdentifier => {
                write!(f, "line {}: '{}' is an invalid identifier.", self.position.line, self.lexeme)
            }
            LexicalErrorType::InvalidKeyword => {
                write!(f, "line {}: '{}' is an invalid keyword.", self.position.line, self.lexeme)
            }
            LexicalErrorType::InvalidRealNumber => {
                write!(f, "line {}: '{}' is an invalid real number. Expected digits after '.'", self.position.line, self.lexeme)
            }
        }
    }
}