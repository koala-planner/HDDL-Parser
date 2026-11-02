use std::{borrow::Borrow, fmt, hash::Hash, write};

use serde::{Deserialize, Serialize};

use crate::{transpiler::format_typed_list, TokenPosition};

use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Task<'a> {
    pub name: &'a str,
    #[serde(skip)]
    pub name_pos: TokenPosition,
    pub parameters: Vec<Symbol<'a>>,
}

impl<'a> Task<'a> {
    pub fn new(name: &'a str, name_pos: TokenPosition, parameters: Vec<Symbol<'a>>) -> Task<'a> {
        Task {
            name,
            name_pos,
            parameters,
        }
    }
}

impl<'a> Hash for Task<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

impl<'a> PartialEq for Task<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<'a> Eq for Task<'a> {}

impl<'a> Borrow<str> for &Task<'a> {
    fn borrow(&self) -> &'a str {
        &self.name
    }
}

impl<'a> Borrow<&'a str> for &Task<'a> {
    fn borrow(&self) -> &&'a str {
        &self.name
    }
}

impl<'a> fmt::Display for Task<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(:task {}\n :parameters ({})\n)",
            self.name,
            format_typed_list(&self.parameters)
        )
    }
}
