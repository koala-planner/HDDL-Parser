use crate::TokenPosition;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
                write!(
                    f,
                    "{}: '{}' is an invalid identifier.",
                    self.position, self.lexeme
                )
            }
            LexicalErrorType::InvalidKeyword => {
                write!(
                    f,
                    "{}: '{}' is an invalid keyword.",
                    self.position, self.lexeme
                )
            }
            LexicalErrorType::InvalidRealNumber => {
                write!(
                    f,
                    "{}: '{}' is an invalid real number. Expected digits after '.'",
                    self.position, self.lexeme
                )
            }
        }
    }
}
