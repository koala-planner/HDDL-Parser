mod analyzers;
mod tdg;
#[cfg(test)]
mod tests;
mod type_checker;
mod undefined_elements;

use crate::output::*;
use crate::syntactic_analyzer::*;
use type_checker::*;
use undefined_elements::*;

extern crate petgraph;

pub use analyzers::*;
pub use tdg::Tdg;
pub use type_checker::TypeChecker;
