use std::fmt;

use super::*;
use crate::transpiler::{format_block, format_list};

#[derive(Debug, Serialize, Deserialize)]
pub struct DomainAST<'a> {
    pub name: String,
    #[serde(default)]
    pub types: Option<Vec<Symbol<'a>>>,
    #[serde(default)]
    pub constants: Option<Vec<Symbol<'a>>>,
    pub requirements: Vec<RequirementType>,
    #[serde(borrow)]
    pub predicates: Vec<Predicate<'a>>,
    pub compound_tasks: Vec<Task<'a>>,
    pub methods: Vec<Method<'a>>,
    pub actions: Vec<Action<'a>>,
    pub functions: Vec<Function<'a>>,
}

impl<'a> DomainAST<'a> {
    pub fn new(name: String) -> DomainAST<'a> {
        DomainAST {
            name,
            types: None,
            constants: None,
            requirements: vec![],
            predicates: vec![],
            compound_tasks: vec![],
            methods: vec![],
            actions: vec![],
            functions: vec![],
        }
    }

    pub fn add_requirement(&mut self, req: RequirementType) {
        self.requirements.push(req);
    }

    pub fn add_predicate(&mut self, predicate: Predicate<'a>) {
        self.predicates.push(predicate);
    }

    pub fn add_compound_task(&mut self, task: Task<'a>) {
        self.compound_tasks.push(task);
    }

    pub fn add_method(&mut self, method: Method<'a>) {
        self.methods.push(method)
    }

    pub fn add_action(&mut self, action: Action<'a>) {
        self.actions.push(action);
    }

    pub fn add_function(&mut self, function: Function<'a>) {
        self.functions.push(function);
    }

    pub fn add_var_type(&mut self, var: Symbol<'a>) {
        match self.types.as_mut() {
            Some(t) => {
                t.push(var);
            }
            None => self.types = Some(vec![var]),
        }
    }

    pub fn add_constant(&mut self, constant: Symbol<'a>) {
        match self.constants.as_mut() {
            Some(c) => {
                c.push(constant);
            }
            None => self.constants = Some(vec![constant]),
        }
    }
}

impl<'a> fmt::Display for DomainAST<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(define (domain {})\n", self.name)?;
        if !self.requirements.is_empty() {
            write!(
                f,
                "\n\t(:requirements {})\n",
                format_list(&self.requirements)
            )?;
        }
        if let Some(types) = &self.types {
            write!(
                f,
                "\n\t{}\n",
                format_block(":types", types).replace('\n', "\n\t")
            )?;
        }
        if let Some(constants) = &self.constants {
            write!(
                f,
                "\n\t{}\n",
                format_block(":constants", constants).replace('\n', "\n\t")
            )?;
        }
        if !self.predicates.is_empty() {
            write!(
                f,
                "\n\t{}\n",
                format_block(":predicates", &self.predicates).replace('\n', "\n\t")
            )?;
        }
        if !self.functions.is_empty() {
            write!(
                f,
                "\n\t{}\n",
                format_block(":functions", &self.functions).replace('\n', "\n\t")
            )?;
        }
        // members are multi-line: shift every line one level to keep their
        // parentheses and keyword colons vertically aligned
        for task in &self.compound_tasks {
            write!(f, "\n\t{}\n", task.to_string().replace('\n', "\n\t"))?;
        }
        for method in &self.methods {
            write!(f, "\n\t{}\n", method.to_string().replace('\n', "\n\t"))?;
        }
        for action in &self.actions {
            write!(f, "\n\t{}\n", action.to_string().replace('\n', "\n\t"))?;
        }
        write!(f, "\n)")
    }
}
