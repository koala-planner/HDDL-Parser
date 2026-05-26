use super::*;

#[derive(Debug, Serialize)]
pub struct DomainAST<'a> {
    pub name: String,
    pub types: Option<Vec<Symbol<'a>>>,
    pub constants: Option<Vec<Symbol<'a>>>,
    pub requirements: Vec<RequirementType>,
    pub predicates: Vec<Predicate<'a>>,
    pub compound_tasks: Vec<Task<'a>>,
    pub methods: Vec<Method<'a>>,
    pub actions: Vec<Action<'a>>,
    pub functions: Vec<Function<'a>>
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
            functions: vec![]
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

    pub fn add_var_type(&mut self, var: Symbol<'a>){
        match self.types.as_mut() {
            Some(t) => {
                t.push(var);
            }
            None => {
                self.types = Some(vec![var])
            }
        }
    }

    pub fn add_constant(&mut self, constant: Symbol<'a>){
        match self.constants.as_mut() {
            Some(c) => {
                c.push(constant);
            }
            None => {
                self.constants = Some(vec![constant])
            }
        }
    }
}