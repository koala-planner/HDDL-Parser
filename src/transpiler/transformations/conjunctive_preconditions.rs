use std::collections::HashSet;

use crate::transpiler::core::leak;
use crate::*;

impl<'a> Transpiler<'a> {
    // Brings every action and method precondition into a conjunction of literals
    pub(crate) fn conjunctive_preconditions(&mut self) -> Result<(), ParsingError> {
        // the requirement flags are recomputed first so they reflect what the
        // preconditions actually contain
        self.refresh_requirements();
        let mut compile_equality = false;
        let mut remove_quantifiers = false;
        for requirement in self.program.domain.requirements.iter() {
            match requirement {
                RequirementType::UniversalPreconditions
                | RequirementType::ExistentialPreconditions
                | RequirementType::QuantifiedPreconditions => remove_quantifiers = true,
                RequirementType::Equality => compile_equality = true,
                _ => {}
            }
        }
        if compile_equality {
            self.remove_equality_constraints()?;
        }

        if remove_quantifiers {
            self.remove_quantifiers()?;
        }

        // names already in use; synthesized copies must not collide with them
        let compound_task_names: HashSet<&'a str> = self
            .program
            .domain
            .compound_tasks
            .iter()
            .map(|task| task.name)
            .collect();
        let mut task_names: HashSet<&'a str> = self
            .program
            .domain
            .actions
            .iter()
            .map(|action| action.name)
            .chain(compound_task_names.iter().copied())
            .collect();
        let mut method_names: HashSet<&'a str> = self
            .program
            .domain
            .methods
            .iter()
            .map(|method| method.name.name)
            .collect();

        // Process Actions
        let actions = std::mem::take(&mut self.program.domain.actions);
        let mut kept_actions = Vec::with_capacity(actions.len());
        let mut wrapper_tasks = vec![];
        let mut wrapper_methods = vec![];
        for mut action in actions {
            let needs_split = action
                .preconditions
                .as_ref()
                .is_some_and(|precondition| !precondition.is_simple_conjunction());
            if !needs_split {
                kept_actions.push(action);
                continue;
            }
            let prec_dnf = action.preconditions.as_ref().unwrap().to_dnf();
            // a single cube needs no wrapper: rewrite the precondition in place
            if prec_dnf.len() == 1 {
                action.preconditions = prec_dnf.into_iter().next().unwrap().into_formula();
                kept_actions.push(action);
                continue;
            }

            if compound_task_names.contains(action.name) {
                return Err(ParsingError::Transformation(format!(
                    "conjunctive-preconditions: cannot wrap action '{}': a compound task of \
                     that name already exists",
                    action.name
                )));
            }
            let Action {
                name,
                name_pos,
                parameters,
                effects,
                ..
            } = action;
            wrapper_tasks.push(Task::new(name, name_pos, parameters.clone()));
            let terms: Vec<Symbol<'a>> = parameters.iter().map(untyped_ref).collect();
            for (cube_id, cube) in prec_dnf.into_iter().enumerate() {
                let copy_name = claim(&mut task_names, format!("{name}_a{cube_id}"));
                kept_actions.push(Action {
                    name: copy_name,
                    name_pos: TokenPosition::default(),
                    parameters: parameters.clone(),
                    preconditions: cube.into_formula(),
                    effects: effects.clone(),
                });
                wrapper_methods.push(Method {
                    name: Symbol::new_untyped(
                        claim(&mut method_names, format!("{name}_m{cube_id}")),
                        TokenPosition::default(),
                    ),
                    task: Symbol::new_untyped(name, TokenPosition::default()),
                    task_terms: terms.clone(),
                    params: parameters.clone(),
                    precondition: None,
                    tn: HTN {
                        subtasks: vec![Subtask {
                            id: None,
                            task: Symbol::new_untyped(copy_name, TokenPosition::default()),
                            terms: terms.clone(),
                        }],
                        ordering_pos: None,
                        orderings: TaskOrdering::Total,
                        constraints: None,
                    },
                });
            }
        }
        self.program.domain.actions = kept_actions;

        // method simply becomes one method per cube
        let methods = std::mem::take(&mut self.program.domain.methods);
        let mut kept_methods = Vec::with_capacity(methods.len());
        for mut method in methods {
            let needs_split = method
                .precondition
                .as_ref()
                .is_some_and(|precondition| !precondition.is_simple_conjunction());
            if !needs_split {
                kept_methods.push(method);
                continue;
            }
            let prec_dnf = method.precondition.as_ref().unwrap().to_dnf();
            if prec_dnf.len() == 1 {
                method.precondition = prec_dnf.into_iter().next().unwrap().into_formula();
                kept_methods.push(method);
                continue;
            }
            let Method {
                name,
                task,
                task_terms,
                params,
                tn,
                ..
            } = method;
            for (cube_id, cube) in prec_dnf.into_iter().enumerate() {
                kept_methods.push(Method {
                    name: Symbol::new_untyped(
                        claim(&mut method_names, format!("{}_m{}", name.name, cube_id)),
                        TokenPosition::default(),
                    ),
                    task: task.clone(),
                    task_terms: task_terms.clone(),
                    params: params.clone(),
                    precondition: cube.into_formula(),
                    tn: tn.clone(),
                });
            }
        }
        self.program.domain.methods = kept_methods;

        // install the wrappers, recompute requirement flags
        self.program.domain.compound_tasks.extend(wrapper_tasks);
        self.program.domain.methods.extend(wrapper_methods);
        self.refresh_requirements();
        Ok(())
    }
}

// a name-only symbol creator, for task terms
fn untyped_ref<'a>(param: &Symbol<'a>) -> Symbol<'a> {
    Symbol::new_untyped(param.name, TokenPosition::default())
}

// register the candidate in `taken`
fn claim<'a>(taken: &mut HashSet<&'a str>, candidate: String) -> &'a str {
    let name = if taken.contains(candidate.as_str()) {
        (0..)
            .map(|i| format!("{candidate}_{i}"))
            .find(|fallback| !taken.contains(fallback.as_str()))
            .unwrap()
    } else {
        candidate
    };
    let name = leak(name);
    taken.insert(name);
    name
}
