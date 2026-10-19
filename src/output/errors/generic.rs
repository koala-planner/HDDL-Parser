use super::*;

#[derive(Debug)]
pub enum ParsingError {
    Lexiacal(LexicalError),
    Syntactic(SyntacticError),
    Semantic(SemanticErrorType),
    JSON(JsonError),
    // a transformation could not be applied to the (otherwise valid) program
    Transformation(String),
}

impl From<LexicalError> for ParsingError {
    fn from(value: LexicalError) -> Self {
        ParsingError::Lexiacal(value)
    }
}

impl From<SyntacticError> for ParsingError {
    fn from(value: SyntacticError) -> Self {
        ParsingError::Syntactic(value)
    }
}

impl From<SemanticErrorType> for ParsingError {
    fn from(value: SemanticErrorType) -> Self {
        ParsingError::Semantic(value)
    }
}

impl From<JsonError> for ParsingError {
    fn from(value: JsonError) -> Self {
        ParsingError::JSON(value)
    }
}

impl From<serde_json::Error> for ParsingError {
    fn from(value: serde_json::Error) -> Self {
        ParsingError::JSON(JsonError {
            line: value.line() as u32,
            message: value.to_string(),
        })
    }
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lexiacal(error) => write!(f, "{}", error),
            Self::Syntactic(error) => write!(f, "{}", error),
            Self::Semantic(error) => write!(f, "{}", error),
            Self::JSON(error) => write!(f, "{}", error),
            Self::Transformation(message) => write!(f, "{}", message),
        }
    }
}
