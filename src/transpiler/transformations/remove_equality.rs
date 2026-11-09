use std::panic;
use std::vec;

use crate::*;

const PRED_NAME: &str = "EQUAL";

impl<'a> Transpiler<'a> {
    pub(crate) fn remove_equality_constraints(&mut self) -> Result<(), ParsingError> {
        remove_equality_constraints(&mut self.program);
        Ok(())
    }
}

fn remove_equality_constraints<'a>(program: &mut HDDLProgram<'a>) {
    if program.problem.is_none() {
        panic!("Compiling inequiality requires a problem instance");
    }
    // TODO: Check whether the predict exists before creating one
    program.domain.add_predicate(Predicate::new(
        PRED_NAME,
        TokenPosition::default(),
        vec![
            Symbol::new_untyped("?x", TokenPosition::default()),
            Symbol::new_untyped("?y", TokenPosition::default()),
        ],
    ));
    for action in program.domain.actions.iter_mut() {
        if let Some(precondition) = &mut action.preconditions {
            replace_equals(precondition, PRED_NAME);
        }
        if let Some(effect) = &mut action.effects {
            replace_effect_equals(effect, PRED_NAME);
        }
    }
    for method in program.domain.methods.iter_mut() {
        if let Some(precondition) = &mut method.precondition {
            replace_equals(precondition, PRED_NAME);
        }
        // task-network constraints become precondition literals
        if let Some(constraints) = method.tn.constraints.take() {
            let literals: Vec<Precondition<'a>> = constraints
                .into_iter()
                .map(|constraint| match constraint {
                    Constraint::Equal(lhs, rhs) => {
                        Precondition::Atom(equal_predicate(PRED_NAME, lhs, rhs))
                    }
                    Constraint::NotEqual(lhs, rhs) => Precondition::Not(Box::new(
                        Precondition::Atom(equal_predicate(PRED_NAME, lhs, rhs)),
                    )),
                })
                .collect();
            if !literals.is_empty() {
                let addition = if literals.len() == 1 {
                    literals.into_iter().next().unwrap()
                } else {
                    Precondition::And(literals.into_iter().map(Box::new).collect())
                };
                method.precondition = Some(match method.precondition.take() {
                    Some(precondition) => precondition.and(addition),
                    None => addition,
                });
            }
        }
    }
    let problem = program.problem.as_mut().unwrap();
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

// the predicate standing in for (= lhs rhs)
fn equal_predicate<'a>(name: &'a str, lhs: &'a str, rhs: &'a str) -> Predicate<'a> {
    Predicate::new(
        name,
        TokenPosition::default(),
        vec![
            Symbol::new_untyped(lhs, TokenPosition::default()),
            Symbol::new_untyped(rhs, TokenPosition::default()),
        ],
    )
}

fn replace_equals<'a>(formula: &mut Precondition<'a>, name: &'a str) {
    match formula {
        Precondition::Equals(lhs, rhs) => {
            let (lhs, rhs) = (*lhs, *rhs);
            *formula = Precondition::Atom(equal_predicate(name, lhs, rhs));
        }
        Precondition::Empty | Precondition::Atom(_) => {}
        Precondition::Not(inner) => replace_equals(inner, name),
        Precondition::And(terms) | Precondition::Or(terms) | Precondition::Xor(terms) => {
            for term in terms.iter_mut() {
                replace_equals(term, name);
            }
        }
        Precondition::Imply(lhs, rhs) => {
            replace_equals(lhs, name);
            replace_equals(rhs, name);
        }
        Precondition::Exists(_, body) | Precondition::ForAll(_, body) => replace_equals(body, name),
    }
}

fn replace_effect_equals<'a>(effect: &mut Effect<'a>, name: &'a str) {
    match effect {
        Effect::Equals(lhs, rhs) => {
            let (lhs, rhs) = (*lhs, *rhs);
            *effect = Effect::Atom(equal_predicate(name, lhs, rhs));
        }
        Effect::Empty | Effect::Atom(_) => {}
        Effect::Not(inner) | Effect::Exists(_, inner) | Effect::ForAll(_, inner) => {
            replace_effect_equals(inner, name)
        }
        Effect::And(terms) | Effect::Or(terms) | Effect::Xor(terms) => {
            for term in terms.iter_mut() {
                replace_effect_equals(term, name);
            }
        }
        Effect::Imply(guard, body) => {
            replace_equals(guard, name);
            replace_effect_equals(body, name);
        }
        Effect::Probabilistic(branches) => {
            for (_, effect) in branches.iter_mut() {
                replace_effect_equals(effect, name);
            }
        }
    }
}
