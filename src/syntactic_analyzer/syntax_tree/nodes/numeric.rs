use std::fmt;

use serde::{Deserialize, Serialize};

use crate::NumberType;

use super::*;

// a numeric-valued term (PDDL's <f-exp>, without arithmetic): a literal or
// the current value of a fluent
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NumericExpr<'a> {
    Number(NumberType),
    // a function applied to its arguments (PDDL's <f-head>)
    Fluent(#[serde(borrow)] Function<'a>),
}

impl<'a> NumericExpr<'a> {
    // ground substitution of the variable `var` by `value`
    pub fn substitute(&self, var: &str, value: &Symbol<'a>) -> NumericExpr<'a> {
        match self {
            NumericExpr::Number(number) => NumericExpr::Number(number.clone()),
            NumericExpr::Fluent(fluent) => NumericExpr::Fluent(fluent.substitute(var, value)),
        }
    }
}

impl<'a> fmt::Display for NumericExpr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumericExpr::Number(number) => write!(f, "{}", number),
            NumericExpr::Fluent(fluent) => write!(f, "{}", fluent),
        }
    }
}
