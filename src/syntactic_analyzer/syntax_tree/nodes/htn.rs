use std::fmt;

use petgraph::algo::toposort;
use petgraph::prelude::GraphMap;
use petgraph::Directed;
use serde::{Deserialize, Serialize};

use super::*;
use crate::transpiler::{format_call, format_typed_list};
use crate::TokenPosition;

#[derive(Debug, Serialize, Deserialize)]
pub struct InitialTaskNetwork<'a> {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<Symbol<'a>>>,
    #[serde(borrow)]
    pub tn: HTN<'a>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HTN<'a> {
    #[serde(borrow)]
    pub subtasks: Vec<Subtask<'a>>,
    #[serde(skip)]
    pub ordering_pos: Option<TokenPosition>,
    pub orderings: TaskOrdering<'a>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constraints: Option<Vec<Constraint<'a>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtask<'a> {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<Symbol<'a>>,
    #[serde(borrow)]
    pub task: Symbol<'a>,
    pub terms: Vec<Symbol<'a>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Constraint<'a> {
    Equal(#[serde(borrow)] &'a str, &'a str),
    NotEqual(#[serde(borrow)] &'a str, &'a str),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskOrdering<'a> {
    Total,
    Partial(#[serde(borrow)] Vec<(&'a str, &'a str)>),
}

impl<'a> fmt::Display for InitialTaskNetwork<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(:htn")?;
        if let Some(parameters) = &self.parameters {
            write!(f, "\n :parameters ({})", format_typed_list(parameters))?;
        }
        write!(f, "\n {}\n)", self.tn)
    }
}

impl<'a> fmt::Display for HTN<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keyword = match self.orderings {
            TaskOrdering::Total => ":ordered-subtasks",
            TaskOrdering::Partial(_) => ":subtasks",
        };
        if self.subtasks.is_empty() {
            write!(f, "{} ()", keyword)?;
        } else {
            let subtasks = self
                .subtasks
                .iter()
                .map(|st| st.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            write!(f, "{} (and {})", keyword, subtasks)?;
        }
        if let TaskOrdering::Partial(orderings) = &self.orderings {
            if !orderings.is_empty() {
                let orderings = orderings
                    .iter()
                    .map(|(t1, t2)| format!("(< {} {})", t1, t2))
                    .collect::<Vec<_>>()
                    .join(" ");
                write!(f, "\n :ordering (and {})", orderings)?;
            }
        }
        if let Some(constraints) = &self.constraints {
            if constraints.is_empty() {
                write!(f, "\n :constraints ()")?;
            } else {
                let constraints = constraints
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                write!(f, "\n :constraints (and {})", constraints)?;
            }
        }
        Ok(())
    }
}

impl<'a> fmt::Display for Subtask<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let call = format_call(self.task.name, &self.terms);
        match &self.id {
            Some(id) => write!(f, "({} {})", id, call),
            None => write!(f, "{}", call),
        }
    }
}

impl<'a> fmt::Display for Constraint<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Constraint::Equal(a, b) => write!(f, "(= {} {})", a, b),
            Constraint::NotEqual(a, b) => write!(f, "(not (= {} {}))", a, b),
        }
    }
}

impl<'a> TaskOrdering<'a> {
    pub fn is_acyclic(&self) -> bool {
        match &self {
            TaskOrdering::Total => true,
            TaskOrdering::Partial(orderings) => {
                let ordering_graph = GraphMap::<_, (), Directed>::from_edges(orderings);
                toposort(&ordering_graph, None).is_ok()
            }
        }
    }
}
