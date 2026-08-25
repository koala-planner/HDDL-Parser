use std::fmt;

use serde::{Deserialize, Serialize};

use super::*;
use crate::transpiler::{format_call, format_typed_list};

#[derive(Debug, Serialize, Deserialize)]
pub struct Method<'a> {
    #[serde(borrow)]
    pub name: Symbol<'a>,
    pub task: Symbol<'a>,
    pub task_terms: Vec<Symbol<'a>>,
    pub params: Vec<Symbol<'a>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub precondition: Option<Formula<'a>>,
    pub tn: HTN<'a>,
}

impl<'a> fmt::Display for Method<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(:method {}\n :parameters ({})\n :task {}",
            self.name,
            format_typed_list(&self.params),
            format_call(self.task.name, &self.task_terms),
        )?;
        if let Some(precondition) = &self.precondition {
            write!(
                f,
                "\n :precondition {}",
                precondition.to_string().replace('\n', "\n ")
            )?;
        }
        write!(f, "\n {}\n)", self.tn)
    }
}
