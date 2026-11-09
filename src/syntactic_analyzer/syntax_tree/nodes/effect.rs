use std::fmt;

use serde::{Deserialize, Serialize};

use crate::NumberType;
use crate::transpiler::format_typed_list;

use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Effect<'a> {
    Empty,
    Atom(Predicate<'a>),
    Not(Box<Effect<'a>>),
    And(Vec<Box<Effect<'a>>>),
    Or(Vec<Box<Effect<'a>>>),
    Xor(Vec<Box<Effect<'a>>>),
    // formula -> formula'
    Imply(Box<Effect<'a>>, Box<Effect<'a>>),
    // ∃vars: formula
    Exists(Vec<Symbol<'a>>, Box<Effect<'a>>),
    // ∀vars: formula
    ForAll(Vec<Symbol<'a>>, Box<Effect<'a>>),
    // a formula holding with the given probability in [0, 1]
    Probabilistic(NumberType, Box<Effect<'a>>),
    // formula = formula'
    Equals(#[serde(borrow)] &'a str, &'a str),
}

impl<'a> Effect<'a> {
    // the predicates mentioned by the effect; those under a quantifier are
    // only included on request, as their bound variables are not parameters
    pub fn get_predicates(&self, include_quantified: bool) -> Vec<&Predicate<'a>> {
        let mut predicates = vec![];
        match self {
            Effect::Empty | Effect::Equals(_, _) => {}
            Effect::Atom(predicate) => {
                predicates.push(predicate);
            }
            Effect::Not(inner) | Effect::Probabilistic(_, inner) => {
                predicates.extend(inner.get_predicates(include_quantified));
            }
            Effect::And(terms) | Effect::Or(terms) | Effect::Xor(terms) => {
                for term in terms {
                    predicates.extend(term.get_predicates(include_quantified));
                }
            }
            Effect::Imply(lhs, rhs) => {
                predicates.extend(lhs.get_predicates(include_quantified));
                predicates.extend(rhs.get_predicates(include_quantified));
            }
            Effect::ForAll(_, body) | Effect::Exists(_, body) => {
                if include_quantified {
                    predicates.extend(body.get_predicates(true));
                }
            }
        }
        predicates
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
            Effect::Atom(_) => true,
            Effect::Not(inner) => matches!(&**inner, Effect::Atom(_) | Effect::Equals(_, _)),
            _ => false,
        }
    }

    pub fn is_quantified(&self) -> bool {
        match self {
            Effect::Empty | Effect::Atom(_) | Effect::Equals(_, _) => false,
            Effect::And(inner) | Effect::Or(inner) | Effect::Xor(inner) => {
                inner.iter().any(|x| x.is_quantified())
            }
            Effect::Exists(_, _) | Effect::ForAll(_, _) => true,
            Effect::Not(inner) => inner.is_quantified(),
            Effect::Imply(lhs, rhs) => lhs.is_quantified() || rhs.is_quantified(),
            Effect::Probabilistic(_, inner) => inner.is_quantified(),
        }
    }

    // ground substitution of the variable `var` by `value`
    pub fn substitute(&self, var: &str, value: &Symbol<'a>) -> Effect<'a> {
        match self {
            Effect::Empty => Effect::Empty,
            Effect::Atom(predicate) => {
                let mut predicate = predicate.clone();
                for symbol in predicate.variables.iter_mut() {
                    if symbol.name == var {
                        *symbol = value.clone();
                    }
                }
                Effect::Atom(predicate)
            }
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
            Effect::Imply(lhs, rhs) => Effect::Imply(
                Box::new(lhs.substitute(var, value)),
                Box::new(rhs.substitute(var, value)),
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
            Effect::Probabilistic(probability, inner) => {
                Effect::Probabilistic(probability.clone(), Box::new(inner.substitute(var, value)))
            }
            Effect::Equals(lhs, rhs) => Effect::Equals(
                if *lhs == var { value.name } else { lhs },
                if *rhs == var { value.name } else { rhs },
            ),
        }
    }

    // whether any subformula (including self) satisfies the predicate; the
    // walk descends into quantifier bodies.
    pub fn any_subformula<F: FnMut(&Effect<'a>) -> bool>(&self, predicate: &mut F) -> bool {
        if predicate(self) {
            return true;
        }
        match self {
            Effect::Empty | Effect::Atom(_) | Effect::Equals(_, _) => false,
            Effect::Not(inner)
            | Effect::Probabilistic(_, inner)
            | Effect::Exists(_, inner)
            | Effect::ForAll(_, inner) => inner.any_subformula(predicate),
            Effect::And(terms) | Effect::Or(terms) | Effect::Xor(terms) => {
                terms.iter().any(|term| term.any_subformula(predicate))
            }
            Effect::Imply(lhs, rhs) => {
                lhs.any_subformula(predicate) || rhs.any_subformula(predicate)
            }
        }
    }
}

impl<'a> fmt::Display for Effect<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // nested renderings are shifted one level so multi-line children align
        fn shift<T: fmt::Display>(term: &T) -> String {
            term.to_string().replace('\n', "\n\t")
        }
        // a connective with a single term stays inline, more get one line each
        fn connective(name: &str, terms: &[Box<Effect>]) -> String {
            match terms {
                [] => format!("({} )", name),
                [single] => format!("({} {})", name, shift(single)),
                _ => {
                    let lines: String = terms
                        .iter()
                        .map(|term| format!("\n\t{}", shift(term)))
                        .collect();
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
            Effect::And(terms) => write!(f, "{}", connective("and", terms)),
            Effect::Or(terms) => write!(f, "{}", connective("or", terms)),
            Effect::Xor(terms) => write!(f, "{}", connective("oneof", terms)),
            Effect::Imply(lhs, rhs) => {
                write!(
                    f,
                    "(when {} {})",
                    shift(&lhs.to_string()),
                    shift(&rhs.to_string())
                )
            }
            Effect::Exists(vars, inner) => {
                write!(f, "(exists ({}) {})", format_typed_list(vars), shift(inner))
            }
            Effect::ForAll(vars, inner) => {
                write!(f, "(forall ({}) {})", format_typed_list(vars), shift(inner))
            }
            Effect::Probabilistic(probability, terms) => {
                write!(f, "(probabilistic {} {})", probability, shift(terms))
            }
            Effect::Equals(lhs, rhs) => write!(f, "(= {} {})", lhs, rhs),
        }
    }
}
