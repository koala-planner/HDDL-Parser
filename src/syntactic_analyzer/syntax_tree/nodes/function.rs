use std::fmt;

use serde::{Deserialize, Serialize};

use super::*;
use crate::TokenPosition;
use crate::transpiler::format_call;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Function<'a> {
    pub name: &'a str,
    #[serde(skip)]
    pub name_pos: TokenPosition,
    pub variables: Vec<Symbol<'a>>,
}

impl<'a> Function<'a> {
    // ground substitution of the variable `var` by `value` in the arguments
    pub fn substitute(&self, var: &str, value: &Symbol<'a>) -> Function<'a> {
        let mut function = self.clone();
        for symbol in function.variables.iter_mut() {
            if symbol.name == var {
                *symbol = value.clone();
            }
        }
        function
    }
}

impl<'a> fmt::Display for Function<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format_call(self.name, &self.variables))
    }
}
