use std::fmt;

use super::*;
use crate::transpiler::{format_block, format_list};
use crate::TokenPosition;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProblemAST<'a> {
    #[serde(borrow, default)]
    pub name: &'a str,
    #[serde(borrow, default)]
    pub domain_name: &'a str,
    pub requirements: Vec<RequirementType>,
    #[serde(default)]
    pub init_tn: Option<InitialTaskNetwork<'a>>,
    pub init_state: Vec<Predicate<'a>>,
    #[serde(default)]
    pub goal: Option<Formula<'a>>,
    #[serde(borrow)]
    pub objects: Vec<Symbol<'a>>,
}

impl<'a> ProblemAST<'a> {
    pub fn new(name: &'a str, domain_name: &'a str) -> ProblemAST<'a> {
        ProblemAST {
            name,
            domain_name,
            requirements: vec![],
            init_tn: None,
            init_state: vec![],
            goal: None,
            objects: vec![],
        }
    }
    pub fn add_object(&mut self, name: &'a str, object_pos: TokenPosition) {
        let object = Symbol::new(name, object_pos, None, None);
        self.objects.push(object);
    }
    pub fn add_typed_object(
        &mut self,
        name: &'a str,
        name_pos: TokenPosition,
        object_type: &'a str,
        type_pos: TokenPosition,
    ) {
        let object = Symbol::new(name, name_pos, Some(object_type), Some(type_pos));
        self.objects.push(object);
    }
    pub fn add_init_tn(&mut self, tn: InitialTaskNetwork<'a>) {
        self.init_tn = Some(tn);
    }
    pub fn add_init_state(&mut self, state: Vec<Predicate<'a>>) {
        self.init_state = state;
    }
    pub fn extend_init_state(&mut self, state: Vec<Predicate<'a>>) {
        self.init_state.extend(state);
    }
    pub fn add_goal(&mut self, goal: Formula<'a>) {
        self.goal = Some(goal);
    }
    pub fn add_requirement(&mut self, req: RequirementType) {
        self.requirements.push(req);
    }
}

impl<'a> fmt::Display for ProblemAST<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "(define (problem {}) (:domain {})",
            self.name, self.domain_name
        )?;
        if !self.requirements.is_empty() {
            write!(
                f,
                "\n\t(:requirements {})\n",
                format_list(&self.requirements)
            )?;
        }
        if !self.objects.is_empty() {
            write!(
                f,
                "\n\t{}\n",
                format_block(":objects", &self.objects).replace('\n', "\n\t")
            )?;
        }
        if let Some(init_tn) = &self.init_tn {
            write!(f, "\n\t{}\n", init_tn.to_string().replace('\n', "\n\t"))?;
        }
        if !self.init_state.is_empty() {
            write!(
                f,
                "\n\t{}\n",
                format_block(":init", &self.init_state).replace('\n', "\n\t")
            )?;
        }
        // the parser only accepts :goal as the last block of a problem
        if let Some(goal) = &self.goal {
            write!(
                f,
                "\n\t(:goal {})\n",
                goal.to_string().replace('\n', "\n\t")
            )?;
        }
        write!(f, "\n)")
    }
}
