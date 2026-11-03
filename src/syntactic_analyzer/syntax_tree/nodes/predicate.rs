use std::{fmt, hash::Hash};

use serde::{Deserialize, Serialize};

use crate::TokenPosition;
use crate::transpiler::format_call;

use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Predicate<'a> {
    pub name: &'a str,
    #[serde(skip)]
    pub name_pos: TokenPosition,
    pub variables: Vec<Symbol<'a>>,
}

impl<'a> Predicate<'a> {
    pub fn new(
        name: &'a str,
        name_pos: TokenPosition,
        variables: Vec<Symbol<'a>>,
    ) -> Predicate<'a> {
        Predicate {
            name,
            name_pos,
            variables,
        }
    }
    pub fn new_dummy(name: &'a str) -> Predicate<'a> {
        Predicate {
            name,
            name_pos: TokenPosition { line: 0 },
            variables: vec![],
        }
    }

    pub fn arity(&self) -> usize {
        self.variables.len()
    }
}

impl<'a> PartialEq for Predicate<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(other.name) && self.arity() == other.arity()
    }
}

impl<'a> Eq for Predicate<'a> {}

impl<'a> Hash for Predicate<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

impl<'a> fmt::Display for Predicate<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format_call(self.name, &self.variables))
    }
}
