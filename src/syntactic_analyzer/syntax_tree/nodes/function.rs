use serde::Serialize;

use crate::TokenPosition;
use super::*;

#[derive(Clone, Debug, Serialize)]
pub struct Function<'a> {
    pub name: &'a str,
    pub name_pos: TokenPosition,
    pub variables: Vec<Symbol<'a>>
}