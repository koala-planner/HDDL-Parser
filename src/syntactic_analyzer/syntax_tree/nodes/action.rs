use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::fmt;
use std::hash::Hash;

use crate::lexical_analyzer::TokenPosition;
use crate::transpiler::format_typed_list;

use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Action<'a> {
    pub name: &'a str,
    #[serde(skip)]
    pub name_pos: TokenPosition,
    pub parameters: Vec<Symbol<'a>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preconditions: Option<Formula<'a>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effects: Option<Formula<'a>>,
}

impl<'a> fmt::Display for Action<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(:action {}\n :parameters ({})",
            self.name,
            format_typed_list(&self.parameters)
        )?;
        if let Some(precondition) = &self.preconditions {
            write!(
                f,
                "\n :precondition {}",
                precondition.to_string().replace('\n', "\n ")
            )?;
        }
        if let Some(effect) = &self.effects {
            write!(f, "\n :effect {}", effect.to_string().replace('\n', "\n "))?;
        }
        write!(f, "\n)")
    }
}

impl<'a> Hash for Action<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

impl<'a> PartialEq for Action<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(other.name)
    }
}

impl<'a> Eq for Action<'a> {}

impl<'a> Borrow<&'a str> for &Action<'a> {
    fn borrow(&self) -> &&'a str {
        &self.name
    }
}
