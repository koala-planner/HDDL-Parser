#[cfg(test)]
mod tests;
mod token_pos;
mod token_types;
mod tokenizer;

pub use crate::output::{LexicalError, LexicalErrorType};
pub use token_pos::*;
pub use token_types::*;
pub use tokenizer::LexicalAnalyzer;
