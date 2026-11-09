use std::fmt;

use serde::{Deserialize, Serialize};

use crate::NumberType;
use crate::transpiler::format_typed_list;

use super::*;

// how a numeric effect combines a fluent's current value with its operand
// (PDDL's <assign-op>)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssignmentOperator {
    Assign,
    Increase,
    Decrease,
    ScaleUp,
    ScaleDown,
}

impl fmt::Display for AssignmentOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keyword = match self {
            AssignmentOperator::Assign => "assign",
            AssignmentOperator::Increase => "increase",
            AssignmentOperator::Decrease => "decrease",
            AssignmentOperator::ScaleUp => "scale-up",
            AssignmentOperator::ScaleDown => "scale-down",
        };
        write!(f, "{}", keyword)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Effect<'a> {
    Empty,
    Atom(Predicate<'a>),
    Not(Box<Effect<'a>>),
    And(Vec<Box<Effect<'a>>>),
    Or(Vec<Box<Effect<'a>>>),
    Xor(Vec<Box<Effect<'a>>>),
    // (when guard body): the body applies when the guard, a precondition, holds
    Imply(Box<Precondition<'a>>, Box<Effect<'a>>),
    // ∃vars: formula
    Exists(Vec<Symbol<'a>>, Box<Effect<'a>>),
    // ∀vars: formula
    ForAll(Vec<Symbol<'a>>, Box<Effect<'a>>),
    // exactly one branch applies, each with its probability in [0, 1]; the
    // remaining mass is no effect
    Probabilistic(Vec<(NumberType, Box<Effect<'a>>)>),
    // formula = formula'
    Equals(#[serde(borrow)] &'a str, &'a str),
    // (op fluent operand): a numeric effect updating the fluent's value,
    // e.g. (increase (total-cost) (road-length ?l1 ?l2))
    Numeric(AssignmentOperator, Function<'a>, NumericExpr<'a>),
}

impl<'a> Effect<'a> {
    // the predicates mentioned by the effect; those under a quantifier are
    // only included on request, as their bound variables are not parameters
    pub fn get_predicates(&self, include_quantified: bool) -> Vec<&Predicate<'a>> {
        let mut predicates = vec![];
        match self {
            Effect::Empty | Effect::Equals(_, _) | Effect::Numeric(_, _, _) => {}
            Effect::Atom(predicate) => {
                predicates.push(predicate);
            }
            Effect::Not(inner) => {
                predicates.extend(inner.get_predicates(include_quantified));
            }
            Effect::And(terms) | Effect::Or(terms) | Effect::Xor(terms) => {
                for term in terms {
                    predicates.extend(term.get_predicates(include_quantified));
                }
            }
            Effect::Imply(guard, body) => {
                predicates.extend(guard.get_predicates(include_quantified));
                predicates.extend(body.get_predicates(include_quantified));
            }
            Effect::ForAll(_, body) | Effect::Exists(_, body) => {
                if include_quantified {
                    predicates.extend(body.get_predicates(true));
                }
            }
            Effect::Probabilistic(branches) => {
                for (_, effect) in branches {
                    predicates.extend(effect.get_predicates(include_quantified));
                }
            }
        }
        predicates
    }

    // the fluents mentioned by the effect, both updated and read
    pub fn get_fluents(&self, include_quantified: bool) -> Vec<&Function<'a>> {
        let mut fluents = vec![];
        match self {
            Effect::Empty | Effect::Atom(_) | Effect::Equals(_, _) => {}
            Effect::Numeric(_, fluent, operand) => {
                fluents.push(fluent);
                if let NumericExpr::Fluent(read) = operand {
                    fluents.push(read);
                }
            }
            Effect::Not(inner) => {
                fluents.extend(inner.get_fluents(include_quantified));
            }
            Effect::And(terms) | Effect::Or(terms) | Effect::Xor(terms) => {
                for term in terms {
                    fluents.extend(term.get_fluents(include_quantified));
                }
            }
            // the guard is a precondition, which holds no fluents
            Effect::Imply(_, body) => {
                fluents.extend(body.get_fluents(include_quantified));
            }
            Effect::ForAll(_, body) | Effect::Exists(_, body) => {
                if include_quantified {
                    fluents.extend(body.get_fluents(true));
                }
            }
            Effect::Probabilistic(branches) => {
                for (_, effect) in branches {
                    fluents.extend(effect.get_fluents(include_quantified));
                }
            }
        }
        fluents
    }

    pub fn is_simple_conjunction(&self) -> bool {
        match self {
            Effect::Empty => true,
            Effect::And(conjuncts) => conjuncts.iter().all(|c| c.is_literal()),
            other => other.is_literal(),
        }
    }

    pub fn and(self, rhs: Effect<'a>) -> Effect<'a> {
        let mut conjuncts = match self {
            Effect::And(conjuncts) => conjuncts,
            other => vec![Box::new(other)],
        };
        match rhs {
            Effect::And(rhs) => conjuncts.extend(rhs),
            other => conjuncts.push(Box::new(other)),
        }
        Effect::And(conjuncts)
    }

    pub(crate) fn is_literal(&self) -> bool {
        match self {
            Effect::Atom(_) | Effect::Numeric(_, _, _) => true,
            Effect::Not(inner) => matches!(&**inner, Effect::Atom(_) | Effect::Equals(_, _)),
            _ => false,
        }
    }

    pub fn is_quantified(&self) -> bool {
        match self {
            Effect::Empty | Effect::Atom(_) | Effect::Equals(_, _) | Effect::Numeric(_, _, _) => {
                false
            }
            Effect::And(inner) | Effect::Or(inner) | Effect::Xor(inner) => {
                inner.iter().any(|x| x.is_quantified())
            }
            Effect::Exists(_, _) | Effect::ForAll(_, _) => true,
            Effect::Not(inner) => inner.is_quantified(),
            Effect::Imply(guard, body) => guard.is_quantified() || body.is_quantified(),
            Effect::Probabilistic(branches) => {
                branches.iter().any(|(_, effect)| effect.is_quantified())
            }
        }
    }

    // ground substitution of the variable `var` by `value`
    pub fn substitute(&self, var: &str, value: &Symbol<'a>) -> Effect<'a> {
        match self {
            Effect::Empty => Effect::Empty,
            Effect::Atom(predicate) => Effect::Atom(predicate.substitute(var, value)),
            Effect::Not(inner) => Effect::Not(Box::new(inner.substitute(var, value))),
            Effect::And(terms) => Effect::And(
                terms
                    .iter()
                    .map(|term| Box::new(term.substitute(var, value)))
                    .collect(),
            ),
            Effect::Or(terms) => Effect::Or(
                terms
                    .iter()
                    .map(|term| Box::new(term.substitute(var, value)))
                    .collect(),
            ),
            Effect::Xor(terms) => Effect::Xor(
                terms
                    .iter()
                    .map(|term| Box::new(term.substitute(var, value)))
                    .collect(),
            ),
            Effect::Imply(guard, body) => Effect::Imply(
                Box::new(guard.substitute(var, value)),
                Box::new(body.substitute(var, value)),
            ),
            Effect::Exists(vars, body) => {
                if vars.iter().any(|v| v.name == var) {
                    self.clone()
                } else {
                    Effect::Exists(vars.clone(), Box::new(body.substitute(var, value)))
                }
            }
            Effect::ForAll(vars, body) => {
                if vars.iter().any(|v| v.name == var) {
                    self.clone()
                } else {
                    Effect::ForAll(vars.clone(), Box::new(body.substitute(var, value)))
                }
            }
            Effect::Probabilistic(branches) => Effect::Probabilistic(
                branches
                    .iter()
                    .map(|(probability, effect)| {
                        (probability.clone(), Box::new(effect.substitute(var, value)))
                    })
                    .collect(),
            ),
            Effect::Equals(lhs, rhs) => Effect::Equals(
                if *lhs == var { value.name } else { lhs },
                if *rhs == var { value.name } else { rhs },
            ),
            Effect::Numeric(op, fluent, operand) => Effect::Numeric(
                *op,
                fluent.substitute(var, value),
                operand.substitute(var, value),
            ),
        }
    }

    // whether any subformula (including self) satisfies the predicate; the
    // walk descends into quantifier bodies and probabilistic branches but not
    // into the guard of a conditional effect, which is a precondition: a
    // predicate interested in guards inspects them on the Imply node
    pub fn any_subformula<F: FnMut(&Effect<'a>) -> bool>(&self, predicate: &mut F) -> bool {
        if predicate(self) {
            return true;
        }
        match self {
            Effect::Empty | Effect::Atom(_) | Effect::Equals(_, _) | Effect::Numeric(_, _, _) => {
                false
            }
            Effect::Not(inner)
            | Effect::Imply(_, inner)
            | Effect::Exists(_, inner)
            | Effect::ForAll(_, inner) => inner.any_subformula(predicate),
            Effect::And(terms) | Effect::Or(terms) | Effect::Xor(terms) => {
                terms.iter().any(|term| term.any_subformula(predicate))
            }
            Effect::Probabilistic(branches) => branches
                .iter()
                .any(|(_, effect)| effect.any_subformula(predicate)),
        }
    }
}

impl<'a> fmt::Display for Effect<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // nested renderings are shifted one level so multi-line children align
        fn shift<T: fmt::Display>(term: &T) -> String {
            term.to_string().replace('\n', "\n\t")
        }
        // a connective over shifted terms: a single term stays inline, more
        // get one line each
        fn connective(name: &str, terms: impl Iterator<Item = String>) -> String {
            let terms: Vec<String> = terms.collect();
            match terms.as_slice() {
                [] => format!("({} )", name),
                [single] => format!("({} {})", name, single),
                _ => {
                    let lines: String = terms.iter().map(|term| format!("\n\t{}", term)).collect();
                    format!("({}{}\n)", name, lines)
                }
            }
        }
        match self {
            Effect::Empty => write!(f, "()"),
            Effect::Atom(predicate) => {
                write!(f, "({}", predicate.name)?;
                for var in predicate.variables.iter() {
                    write!(f, " {}", var.name)?;
                }
                write!(f, ")")
            }
            Effect::Not(inner) => write!(f, "(not {})", shift(inner)),
            Effect::And(terms) => write!(f, "{}", connective("and", terms.iter().map(shift))),
            Effect::Or(terms) => write!(f, "{}", connective("or", terms.iter().map(shift))),
            Effect::Xor(terms) => write!(f, "{}", connective("oneof", terms.iter().map(shift))),
            Effect::Imply(guard, body) => write!(f, "(when {} {})", shift(guard), shift(body)),
            Effect::Exists(vars, inner) => {
                write!(f, "(exists ({}) {})", format_typed_list(vars), shift(inner))
            }
            Effect::ForAll(vars, inner) => {
                write!(f, "(forall ({}) {})", format_typed_list(vars), shift(inner))
            }
            Effect::Probabilistic(branches) => {
                let terms = branches
                    .iter()
                    .map(|(probability, effect)| format!("{} {}", probability, shift(effect)));
                write!(f, "{}", connective("probabilistic", terms))
            }
            Effect::Equals(lhs, rhs) => write!(f, "(= {} {})", lhs, rhs),
            Effect::Numeric(op, fluent, operand) => write!(f, "({} {} {})", op, fluent, operand),
        }
    }
}
