use std::{borrow::Borrow, fmt, hash::Hash, write};

use serde::{Deserialize, Serialize};

use crate::TokenPosition;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Symbol<'a> {
    pub name: &'a str,
    #[serde(skip)]
    pub name_pos: TokenPosition,
    #[serde(borrow, default, skip_serializing_if = "Option::is_none")]
    pub symbol_type: Option<&'a str>,
    #[serde(skip)]
    pub type_pos: Option<TokenPosition>,
}

impl<'a> Symbol<'a> {
    pub fn new(
        name: &'a str,
        name_pos: TokenPosition,
        symbol_type: Option<&'a str>,
        type_pos: Option<TokenPosition>,
    ) -> Symbol<'a> {
        Symbol {
            name,
            name_pos,
            symbol_type,
            type_pos,
        }
    }
    pub fn new_untyped(name: &'a str, name_pos: TokenPosition) -> Symbol<'a> {
        Symbol {
            name,
            name_pos,
            symbol_type: None,
            type_pos: None,
        }
    }
}

impl<'a> Eq for Symbol<'a> {}

impl<'a> PartialEq for Symbol<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(other.name)
    }
}

impl<'a> Hash for Symbol<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

impl<'a> Borrow<&'a str> for &Symbol<'a> {
    fn borrow(&self) -> &&'a str {
        &self.name
    }
}

impl<'a> fmt::Display for Symbol<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.symbol_type {
            Some(t) => write!(f, "{} - {}", self.name, t),
            None => write!(f, "{}", self.name),
        }
    }
}
