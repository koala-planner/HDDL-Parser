use super::*;
use crate::lexical_analyzer::TokenPosition;

// a predicate or function, declared over or applied to a list of symbols; the
// declaration and type checks treat both alike and report through the error
// each one names here
pub trait Call<'a> {
    fn name(&self) -> &'a str;
    fn name_pos(&self) -> TokenPosition;
    fn symbols(&self) -> &[Symbol<'a>];
    fn undefined(error: UndefinedSymbolError) -> SemanticErrorType;
    fn inconsistent_arity(error: ArityError) -> SemanticErrorType;
    fn inconsistent_arg_type(error: TypeError) -> SemanticErrorType;
}

impl<'a> Call<'a> for Predicate<'a> {
    fn name(&self) -> &'a str {
        self.name
    }
    fn name_pos(&self) -> TokenPosition {
        self.name_pos
    }
    fn symbols(&self) -> &[Symbol<'a>] {
        &self.variables
    }
    fn undefined(error: UndefinedSymbolError) -> SemanticErrorType {
        SemanticErrorType::UndefinedPredicate(error)
    }
    fn inconsistent_arity(error: ArityError) -> SemanticErrorType {
        SemanticErrorType::InconsistentPredicateArity(error)
    }
    fn inconsistent_arg_type(error: TypeError) -> SemanticErrorType {
        SemanticErrorType::InconsistentPredicateArgType(error)
    }
}

impl<'a> Call<'a> for Function<'a> {
    fn name(&self) -> &'a str {
        self.name
    }
    fn name_pos(&self) -> TokenPosition {
        self.name_pos
    }
    fn symbols(&self) -> &[Symbol<'a>] {
        &self.variables
    }
    fn undefined(error: UndefinedSymbolError) -> SemanticErrorType {
        SemanticErrorType::UndefinedFunction(error)
    }
    fn inconsistent_arity(error: ArityError) -> SemanticErrorType {
        SemanticErrorType::InconsistentFunctionArity(error)
    }
    fn inconsistent_arg_type(error: TypeError) -> SemanticErrorType {
        SemanticErrorType::InconsistentFunctionArgType(error)
    }
}
