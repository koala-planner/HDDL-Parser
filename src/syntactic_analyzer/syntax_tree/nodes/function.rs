use std::fmt;

use serde::{Deserialize, Serialize};

use super::*;
use crate::transpiler::format_call;
use crate::TokenPosition;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Function<'a> {
    pub name: &'a str,
    #[serde(skip)]
    pub name_pos: TokenPosition,
    pub variables: Vec<Symbol<'a>>,
}

impl<'a> fmt::Display for Function<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format_call(self.name, &self.variables))
    }
}
