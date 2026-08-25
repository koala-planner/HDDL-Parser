use core::panic;
use std::collections::HashMap;
use std::vec;

use crate::*;
impl<'a> Transpiler<'a> {
    pub fn remove_types(&mut self) {
        if self.program.domain.types.is_none() {
            return;
        }
        // the checker owns its copy of the hierarchy, so it stays usable
        // after the typings are dropped below
        let checker = TypeChecker::new(&self.program.domain.types);

        // each type becomes a unary predicate; when the type's name is
        // already declared as a unary predicate, a fresh name is used instead
        let mut type_predicates: HashMap<&'a str, &'a str> = HashMap::new();
        for type_name in checker.get_types() {
            let predicate_name = self.fresh_predicate_name(type_name, 1);
            self.program.domain.add_predicate(Predicate::new(
                predicate_name,
                TokenPosition::default(),
                vec![Symbol::new("?var", TokenPosition::default(), None, None)],
            ));
            type_predicates.insert(type_name, predicate_name);
        }
        // util for creating predicate calls
        let type_atom = |ty: &'a str, name| {
            Predicate::new(
                type_predicates[ty],
                TokenPosition::default(),
                vec![Symbol::new(name, TokenPosition::default(), None, None)],
            )
        };
        // Drop typings
        self.program.domain.types = None;
        self.program
            .domain
            .requirements
            .retain(|x| *x != RequirementType::TypedObjects);
        // declaration typings carry no constraint; they are simply cleared
        for predicate in self.program.domain.predicates.iter_mut() {
            for var in predicate.variables.iter_mut() {
                var.symbol_type = None;
                var.type_pos = None;
            }
        }
        for function in self.program.domain.functions.iter_mut() {
            for var in function.variables.iter_mut() {
                var.symbol_type = None;
                var.type_pos = None;
            }
        }
        // convert typed domain constants to untyped
        let mut init = Vec::new();
        let program = &mut self.program;
        if let Some(constants) = &mut program.domain.constants {
            match &mut program.problem {
                Some(_) => {
                    for constant in constants.iter_mut() {
                        constant.type_pos = None;
                        let name = constant.name;
                        let Some(t) = constant.symbol_type.take() else {
                            continue;
                        };
                        init.extend(
                            std::iter::once(t)
                                .chain(checker.get_supertypes(t))
                                .map(|ty| type_atom(ty, name)),
                        );
                    }
                }
                None => {
                    let untyped_constants = constants.iter().all(|x| x.symbol_type.is_none());
                    if !untyped_constants {
                        panic!("Typed constants can only be dropped from a domain if a problem is provided")
                    }
                }
            }
        }
        // convert typed action params to untyped
        for action in program.domain.actions.iter_mut() {
            if let Some(precondition) = &mut action.preconditions {
                Transpiler::untype_formula(precondition, &checker, &type_predicates);
            }
            if let Some(effect) = &mut action.effects {
                Transpiler::untype_formula(effect, &checker, &type_predicates);
            }
            let constraints = Transpiler::collect_param_constraints(&mut action.parameters);
            Transpiler::apply_constraints(
                &mut action.preconditions,
                Transpiler::minimize_constraints(constraints, &checker),
                &type_predicates,
            );
        }
        // an init htn block is hoisted into a fresh domain task with a single method
        // whose parameters take over the typings
        if let Some(htn) = program.problem.as_mut().and_then(|p| p.init_tn.as_mut()) {
            let has_typed_params = htn
                .parameters
                .iter()
                .flatten()
                .any(|param| param.symbol_type.is_some());
            if has_typed_params {
                let task_taken = program
                    .domain
                    .compound_tasks
                    .iter()
                    .any(|task| task.name == HDDLProgram::HTN_TOP_TASK);
                let method_taken = program
                    .domain
                    .methods
                    .iter()
                    .any(|method| method.name.name == HDDLProgram::HTN_TOP_METHOD);
                if task_taken || method_taken {
                    panic!(
                        "unable to convert. {} or {} is already defined.",
                        HDDLProgram::HTN_TOP_TASK,
                        HDDLProgram::HTN_TOP_METHOD
                    );
                }
                let top = Symbol::new_untyped(HDDLProgram::HTN_TOP_TASK, TokenPosition::default());
                let entry_tn = HTN {
                    subtasks: vec![Subtask {
                        id: None,
                        task: top.clone(),
                        terms: vec![],
                    }],
                    ordering_pos: None,
                    orderings: TaskOrdering::Total,
                    constraints: None,
                };
                let params = htn.parameters.take().unwrap();
                let tn = std::mem::replace(&mut htn.tn, entry_tn);
                program.domain.add_compound_task(Task::new(
                    HDDLProgram::HTN_TOP_TASK,
                    TokenPosition::default(),
                    vec![],
                ));
                program.domain.add_method(Method {
                    name: Symbol::new_untyped(
                        HDDLProgram::HTN_TOP_METHOD,
                        TokenPosition::default(),
                    ),
                    task: top,
                    task_terms: vec![],
                    params,
                    precondition: None,
                    tn,
                });
            }
        }
        // convert typed tasks to untyped
        let mut task_param_types: HashMap<&'a str, Vec<Option<&'a str>>> = HashMap::new();
        for task in program.domain.compound_tasks.iter_mut() {
            let mut types = vec![];
            for param in task.parameters.iter_mut() {
                param.type_pos = None;
                types.push(param.symbol_type.take());
            }
            task_param_types.insert(task.name, types);
        }
        // convert typed methods to untyped;
        // each method also receives the typings of its task's terms
        for method in program.domain.methods.iter_mut() {
            if let Some(precondition) = &mut method.precondition {
                Transpiler::untype_formula(precondition, &checker, &type_predicates);
            }
            let mut constraints = Transpiler::collect_param_constraints(&mut method.params);
            if let Some(types) = task_param_types.get(method.task.name) {
                for (term, t) in method.task_terms.iter().zip(types) {
                    if let Some(t) = *t {
                        constraints.push((term.name, t));
                    }
                }
            }
            Transpiler::apply_constraints(
                &mut method.precondition,
                Transpiler::minimize_constraints(constraints, &checker),
                &type_predicates,
            );
        }
        // Convert typed objects to untyped
        if let Some(problem) = &mut program.problem {
            if let Some(goal) = &mut problem.goal {
                Transpiler::untype_formula(goal, &checker, &type_predicates);
            }
            for object in problem.objects.iter_mut() {
                object.type_pos = None;
                let name = object.name;
                let Some(t) = object.symbol_type.take() else {
                    continue;
                };
                init.extend(
                    std::iter::once(t)
                        .chain(checker.get_supertypes(t))
                        .map(|ty| type_atom(ty, name)),
                );
            }
            problem.extend_init_state(init);
        }
    }

    // strips the parameter typings, returning them as (variable, type) constraints
    fn collect_param_constraints(parameters: &mut Vec<Symbol<'a>>) -> Vec<(&'a str, &'a str)> {
        let mut constraints = vec![];
        for param in parameters.iter_mut() {
            param.type_pos = None;
            let Some(t) = param.symbol_type.take() else {
                continue;
            };
            constraints.push((param.name, t));
        }
        constraints
    }

    // drops constraints implied by a more specific constraint on the same variable
    fn minimize_constraints(
        constraints: Vec<(&'a str, &'a str)>,
        checker: &TypeChecker<'a>,
    ) -> Vec<(&'a str, &'a str)> {
        let mut kept: Vec<(&'a str, &'a str)> = vec![];
        for (var, t) in constraints {
            let implied = kept
                .iter()
                .any(|(kv, kt)| *kv == var && (*kt == t || checker.get_supertypes(kt).contains(t)));
            if implied {
                continue;
            }
            kept.retain(|(kv, kt)| !(*kv == var && checker.get_supertypes(t).contains(kt)));
            kept.push((var, t));
        }
        kept
    }

    fn constraint_atoms(
        constraints: Vec<(&'a str, &'a str)>,
        type_predicates: &HashMap<&'a str, &'a str>,
    ) -> Vec<Formula<'a>> {
        constraints
            .into_iter()
            .map(|(var, t)| {
                Formula::Atom(Predicate::new(
                    type_predicates[t],
                    TokenPosition::default(),
                    vec![Symbol::new_untyped(var, TokenPosition::default())],
                ))
            })
            .collect()
    }

    fn apply_constraints(
        precondition: &mut Option<Formula<'a>>,
        constraints: Vec<(&'a str, &'a str)>,
        type_predicates: &HashMap<&'a str, &'a str>,
    ) {
        let conjuncts = Formula::And(
            Transpiler::constraint_atoms(constraints, type_predicates)
                .into_iter()
                .map(Box::new)
                .collect(),
        );
        *precondition = Some(match precondition.take() {
            Some(prec) => prec.and(conjuncts),
            None => conjuncts,
        });
    }

    fn untype_formula(
        formula: &mut Formula<'a>,
        checker: &TypeChecker<'a>,
        type_predicates: &HashMap<&'a str, &'a str>,
    ) {
        match formula {
            Formula::Empty | Formula::Atom(_) | Formula::Equals(_, _) => {}
            Formula::Not(inner) | Formula::Probabilistic(_, inner) => {
                Transpiler::untype_formula(inner, checker, type_predicates)
            }
            Formula::And(terms) | Formula::Or(terms) | Formula::Xor(terms) => {
                for term in terms.iter_mut() {
                    Transpiler::untype_formula(term, checker, type_predicates);
                }
            }
            Formula::Imply(lhs, rhs) => {
                Transpiler::untype_formula(lhs, checker, type_predicates);
                Transpiler::untype_formula(rhs, checker, type_predicates);
            }
            Formula::Exists(vars, body) => {
                Transpiler::untype_formula(body, checker, type_predicates);
                let constraints = Transpiler::minimize_constraints(
                    Transpiler::collect_param_constraints(vars),
                    checker,
                );
                if !constraints.is_empty() {
                    let mut conjuncts = Transpiler::constraint_atoms(constraints, type_predicates);
                    conjuncts.push(std::mem::replace(&mut **body, Formula::Empty));
                    **body = Formula::And(conjuncts.into_iter().map(Box::new).collect());
                }
            }
            Formula::ForAll(vars, body) => {
                Transpiler::untype_formula(body, checker, type_predicates);
                let constraints = Transpiler::minimize_constraints(
                    Transpiler::collect_param_constraints(vars),
                    checker,
                );
                if !constraints.is_empty() {
                    let mut atoms = Transpiler::constraint_atoms(constraints, type_predicates);
                    let lhs = match atoms.len() {
                        1 => Box::new(atoms.pop().unwrap()),
                        _ => Box::new(Formula::And(atoms.into_iter().map(Box::new).collect())),
                    };
                    let rhs = Box::new(std::mem::replace(&mut **body, Formula::Empty));
                    **body = Formula::Imply(lhs, rhs);
                }
            }
        }
    }
}
