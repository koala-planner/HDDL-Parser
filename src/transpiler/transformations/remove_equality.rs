use std::vec;

use crate::*;

const PRED_NAME: &str = "EQUAL";

impl<'a> Transpiler<'a> {
    pub(crate) fn remove_equality_constraints(&mut self) -> Result<(), ParsingError> {
        let Some(problem) = self.program.problem.as_mut() else {
            return Err(ParsingError::Transformation(
                "remove-equality-constraints requires a problem input".to_string(),
            ));
        };

        remove_equality_constraints(&mut self.program.domain, problem);
        Ok(())
    }
}

fn remove_equality_constraints<'a>(domain: &mut DomainAST<'a>, problem: &mut ProblemAST<'a>) {
    // TODO: Check whether the predict exists before creating one
    domain.add_predicate(Predicate::new(
        PRED_NAME,
        TokenPosition::default(),
        vec![
            Symbol::new_untyped("?x", TokenPosition::default()),
            Symbol::new_untyped("?y", TokenPosition::default()),
        ],
    ));
    for action in domain.actions.iter_mut() {
        if let Some(precondition) = &mut action.preconditions {
            replace_equals(precondition, PRED_NAME);
        }
        if let Some(effect) = &mut action.effects {
            replace_equals(effect, PRED_NAME);
        }
    }
    for method in domain.methods.iter_mut() {
        if let Some(precondition) = &mut method.precondition {
            replace_equals(precondition, PRED_NAME);
        }
        // task-network constraints become precondition literals
        if let Some(constraints) = method.tn.constraints.take() {
            let literals: Vec<Formula<'a>> = constraints
                .into_iter()
                .map(|constraint| match constraint {
                    Constraint::Equal(lhs, rhs) => equal_atom(PRED_NAME, lhs, rhs),
                    Constraint::NotEqual(lhs, rhs) => {
                        Formula::Not(Box::new(equal_atom(PRED_NAME, lhs, rhs)))
                    }
                })
                .collect();
            if !literals.is_empty() {
                let addition = if literals.len() == 1 {
                    literals.into_iter().next().unwrap()
                } else {
                    Formula::And(literals.into_iter().map(Box::new).collect())
                };
                method.precondition = Some(match method.precondition.take() {
                    Some(precondition) => precondition.and(addition),
                    None => addition,
                });
            }
        }
    }
    if let Some(goal) = &mut problem.goal {
        replace_equals(goal, PRED_NAME);
    }

    let facts = problem.objects.iter().map(|o| {
        Predicate::new(
            PRED_NAME,
            TokenPosition::default(),
            vec![o.clone(), o.clone()],
        )
    });
    problem.extend_init_state(facts.collect());
}

fn equal_atom<'a>(name: &'a str, lhs: &'a str, rhs: &'a str) -> Formula<'a> {
    Formula::Atom(Predicate::new(
        name,
        TokenPosition::default(),
        vec![
            Symbol::new_untyped(lhs, TokenPosition::default()),
            Symbol::new_untyped(rhs, TokenPosition::default()),
        ],
    ))
}

fn replace_equals<'a>(formula: &mut Formula<'a>, name: &'a str) {
    match formula {
        Formula::Equals(lhs, rhs) => {
            let (lhs, rhs) = (*lhs, *rhs);
            *formula = equal_atom(name, lhs, rhs);
        }
        Formula::Empty | Formula::Atom(_) => {}
        Formula::Not(inner) | Formula::Probabilistic(_, inner) => replace_equals(inner, name),
        Formula::And(terms) | Formula::Or(terms) | Formula::Xor(terms) => {
            for term in terms.iter_mut() {
                replace_equals(term, name);
            }
        }
        Formula::Imply(lhs, rhs) => {
            replace_equals(lhs, name);
            replace_equals(rhs, name);
        }
        Formula::Exists(_, body) | Formula::ForAll(_, body) => replace_equals(body, name),
    }
}
