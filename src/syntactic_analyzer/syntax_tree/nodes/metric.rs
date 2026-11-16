use std::fmt;

use serde::{Deserialize, Serialize};

use super::*;

// the plan quality a problem asks to optimise (PDDL's :metric): a direction
// and the expression measured at the end of the plan, or the makespan of the
// plan
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Metric<'a> {
    Minimize(#[serde(borrow)] NumericExpr<'a>),
    Maximize(NumericExpr<'a>),
    // (:metric minimize (makespan)): makespan is a property of the plan's
    // ordering, not a fluent of the state, and is only ever minimised
    Makespan,
}

impl<'a> fmt::Display for Metric<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Metric::Minimize(expression) => write!(f, "(:metric minimize {})", expression),
            Metric::Maximize(expression) => write!(f, "(:metric maximize {})", expression),
            Metric::Makespan => write!(f, "(:metric minimize (makespan))"),
        }
    }
}
