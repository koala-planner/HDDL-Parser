use crate::lexical_analyzer::TokenPosition;

#[derive(Debug, Clone)]
pub enum WarningType {
    // Action Errors
    // TODO: implement
    ImmutablePredicate(String),
    // Compound Task errors
    NoPrimitiveRefinement(WarningInfo),
    // Redundant Elements
    // TODO: implement
    UnusedType(String),
    // TODO: implement
    UnusedPredicate(String),
    // TODO: implement
    UnusedParameter(String),
    // TODO: implement
    RedundantEffect,
}

impl std::fmt::Display for WarningType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::ImmutablePredicate(predicate) => {
                write!(
                    f,
                    "Predicate {} does not appear in the effect of any action",
                    predicate
                )
            }
            Self::NoPrimitiveRefinement(info) => {
                write!(
                    f,
                    "line {}: compound task {} does not have a primitive refinement",
                    info.position.line, info.symbol
                )
            }
            Self::UnusedType(type_name) => {
                write!(f, "Type {} is declared, but never used", type_name)
            }
            Self::UnusedPredicate(predicate) => {
                write!(f, "Predicate {} is declared, but never used", predicate)
            }
            Self::UnusedParameter(parameter) => {
                write!(f, "Parameter {} is declared, but never used", parameter)
            }
            Self::RedundantEffect => {
                // TODO:
                todo!()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct WarningInfo {
    pub symbol: String,
    pub position: TokenPosition,
}
