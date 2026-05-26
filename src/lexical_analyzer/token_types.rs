use std::fmt;

use serde::Serialize;

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    Keyword(KeywordName),
    Identifier(&'a str),
    Number(NumberType),
    Operator(OperationType),
    Punctuator(PunctuationType),
    Requirement(RequirementType),
    EOF
}

impl <'a> fmt::Display for Token<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Token::Keyword(keyword) => write!(fmt, "Keyword {}", keyword),
            Token::Identifier(id) => write!(fmt, "Identifier {}", id),
            Token::Number(n) => write!(fmt, "Number {}", n),
            Token::Operator(op) => write!(fmt, "{}", op),
            Token::Punctuator(punc) => write!(fmt, "{}", punc),
            Token::Requirement(req) => write!(fmt, "Requirement {}", req),
            Token::EOF => write!(fmt, "End of file"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PunctuationType {
    Dash,
    LParentheses,
    RParentheses,
}

impl fmt::Display for PunctuationType {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            PunctuationType::Dash => write!(fmt, "-"),
            PunctuationType::LParentheses => write!(fmt, "("),
            PunctuationType::RParentheses => write!(fmt, ")"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum OperationType {
    // Logic
    Or,
    Not,
    And,
    Xor,
    ForAll,
    Exists,
    Implication,
    // Stochastic
    Probabilistic,
    // Ordering
    Equal,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
}


impl fmt::Display for OperationType {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            OperationType::Or => write!(fmt, "or"),
            OperationType::Not => write!(fmt, "not"),
            OperationType::And => write!(fmt, "and"),
            OperationType::Xor => write!(fmt, "oneof"),
            OperationType::ForAll => write!(fmt, "forall"),
            OperationType::Exists => write!(fmt, "exists"),
            OperationType::Implication => write!(fmt, "when"),
            OperationType::Probabilistic => write!(fmt, "probabilistic"),
            OperationType::Equal => write!(fmt, "="),
            OperationType::LessThan => write!(fmt, "<"),
            OperationType::GreaterThan => write!(fmt, ">"),
            OperationType::LessThanOrEqual => write!(fmt, "<="),
            OperationType::GreaterThanOrEqual => write!(fmt, ">="),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize)]
pub enum RequirementType {
    MethodPreconditions,
    Hierarchy,
    TypedObjects,
    NegativePreconditions,
    UniversalPreconditions,
    Equality,
    STRIPS,
    ProbabilisticEffects,
}

impl fmt::Display for RequirementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let requirement = match self {
            RequirementType::MethodPreconditions => ":method-preconditions",
            RequirementType::Hierarchy => ":hierarchy",
            RequirementType::TypedObjects => ":typing",
            RequirementType::NegativePreconditions => ":negative-preconditions",
            RequirementType::UniversalPreconditions => ":universal-preconditions",
            RequirementType::Equality => ":equality",
            RequirementType::STRIPS => ":strips",
            RequirementType::ProbabilisticEffects => ":probabilistic-effects",
        };
        write!(f, "{}", requirement)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum KeywordName {
    Define,
    Domain,
    Problem,
    Requirements,
    Objects,
    Types,
    Task,
    Constants,
    Predicates,
    Functions,
    Init,
    HTN,
    Action,
    Parameters,
    Method,
    Precondition,
    Effect,
    Subtasks, // either "tasks" or "subtasks"
    OrderedSubtasks, // either "ordered-tasks" or "ordered-subtasks"
    Ordering,
    Constraints,
    Goal
}

impl fmt::Display for KeywordName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keyword = match self {
            KeywordName::Define => ":define",
            KeywordName::Domain => ":domain",
            KeywordName::Problem => ":problem",
            KeywordName::Requirements => ":requirements",
            KeywordName::Objects => ":objects",
            KeywordName::Types => ":types",
            KeywordName::Task => ":task",
            KeywordName::Constants => ":constants",
            KeywordName::Predicates => ":predicates",
            KeywordName::Functions => ":functions",
            KeywordName::Init => ":init",
            KeywordName::HTN => ":htn",
            KeywordName::Action => ":action",
            KeywordName::Parameters => ":parameters",
            KeywordName::Method => ":method",
            KeywordName::Precondition => ":precondition",
            KeywordName::Effect => ":effect",
            KeywordName::Subtasks => ":(sub)tasks", // Note: Could be ":tasks" in some systems
            KeywordName::OrderedSubtasks => ":ordered-(sub)tasks", // Note: Could be ":ordered-tasks" in some systems
            KeywordName::Ordering => ":ordering",
            KeywordName::Constraints => ":constraints",
            KeywordName::Goal => ":goal",
        };
        write!(f, "{}", keyword)
    }
}

// TODO: Add support for lexing negative numbers
#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum NumberType {
    Real(f64),
    Integer(i64)
}

impl fmt::Display for NumberType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumberType::Integer(n) => return write!(f, "{}", n),
            NumberType::Real(r) => return write!(f, "{}", r)
        }
    }
}