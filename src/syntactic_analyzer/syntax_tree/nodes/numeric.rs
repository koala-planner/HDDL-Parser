use std::fmt;

use serde::{Deserialize, Serialize};

use crate::NumberType;
use crate::transpiler::format_list;

use super::*;

// a numeric-valued term (PDDL's <f-exp>): a literal, the current value of a
// fluent, or a sum or product of such terms
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NumericExpr<'a> {
    Number(NumberType),
    // a function applied to its arguments (PDDL's <f-head>)
    Fluent(#[serde(borrow)] Function<'a>),
    // (+ e1 e2 ...) and (* e1 e2 ...), each over at least two operands
    Sum(Vec<NumericExpr<'a>>),
    Product(Vec<NumericExpr<'a>>),
}

impl<'a> NumericExpr<'a> {
    // the fluents whose values the expression reads
    pub fn get_fluents(&self) -> Vec<&Function<'a>> {
        match self {
            NumericExpr::Number(_) => vec![],
            NumericExpr::Fluent(fluent) => vec![fluent],
            NumericExpr::Sum(operands) | NumericExpr::Product(operands) => operands
                .iter()
                .flat_map(|operand| operand.get_fluents())
                .collect(),
        }
    }

    // ground substitution of the variable `var` by `value`
    pub fn substitute(&self, var: &str, value: &Symbol<'a>) -> NumericExpr<'a> {
        let substitute_all = |operands: &[NumericExpr<'a>]| -> Vec<NumericExpr<'a>> {
            operands
                .iter()
                .map(|operand| operand.substitute(var, value))
                .collect()
        };
        match self {
            NumericExpr::Number(number) => NumericExpr::Number(number.clone()),
            NumericExpr::Fluent(fluent) => NumericExpr::Fluent(fluent.substitute(var, value)),
            NumericExpr::Sum(operands) => NumericExpr::Sum(substitute_all(operands)),
            NumericExpr::Product(operands) => NumericExpr::Product(substitute_all(operands)),
        }
    }
}

impl<'a> fmt::Display for NumericExpr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumericExpr::Number(number) => write!(f, "{}", number),
            NumericExpr::Fluent(fluent) => write!(f, "{}", fluent),
            NumericExpr::Sum(operands) => write!(f, "(+ {})", format_list(operands)),
            NumericExpr::Product(operands) => write!(f, "(* {})", format_list(operands)),
        }
    }
}

// the value a fluent holds in the initial state, (= fluent number)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FluentValue<'a> {
    #[serde(borrow)]
    pub fluent: Function<'a>,
    pub value: NumberType,
}

impl<'a> fmt::Display for FluentValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(= {} {})", self.fluent, self.value)
    }
}
