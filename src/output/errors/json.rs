use crate::TokenPosition;
use std::fmt;

#[derive(Debug)]
pub struct JsonError {
    /// serde_json's rendered message; already includes "at line L column C".
    pub message: String,
    /// where serde_json detected the error; zero when it has no location
    pub position: TokenPosition,
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
