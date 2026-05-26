use std::fmt;

use serde::Serialize;

use crate::lexical_analyzer::NumberType;

use super::*;

#[derive(Clone, Debug, Serialize)]
pub enum Formula<'a> {
    Empty,
    Atom(Predicate<'a>),
    Not(Box<Formula<'a>>),
    And(Vec<Box<Formula<'a>>>),
    Or(Vec<Box<Formula<'a>>>),
    Xor(Vec<Box<Formula<'a>>>),
    // formula -> formula'
    Imply(Vec<Box<Formula<'a>>>, Vec<Box<Formula<'a>>>),
    // ∃vars: formula
    Exists(Vec<Symbol<'a>>, Box<Formula<'a>>),
    // ∀vars: formula
    ForAll(Vec<Symbol<'a>>, Box<Formula<'a>>),
    // probability, weights, and other quantities.
    Weighted(NumberType, Box<Formula<'a>>),
    // formula = formula'
    Equals(&'a str, &'a str),
}

impl<'a> Formula<'a> {
    pub fn get_propositional_predicates(&self) -> Vec<&Predicate<'a>> {
        let mut predicates = vec![];
        match &*self {
            Formula::Empty => {}
            Formula::Atom(predicate) => {
                predicates.push(predicate);
            }
            Formula::Not(new_formula) => {
                predicates.extend(new_formula.get_propositional_predicates().iter());
            }
            Formula::And(new_formula) | Formula::Or(new_formula) | Formula::Xor(new_formula) => {
                for f in new_formula {
                    predicates.extend(f.get_propositional_predicates().iter());
                }
            }
            Formula::Imply(ps, qs) => {
                for p in ps {
                    predicates.extend(p.get_propositional_predicates().iter());
                }
                for q in qs {
                    predicates.extend(q.get_propositional_predicates().iter());
                }
            },
            Formula::Weighted(_, q) => {
                predicates.extend(q.get_propositional_predicates().iter());
            }
            Formula::Equals(_, _) => {}
            // not propositional
            Formula::ForAll(_, _) | Formula::Exists(_, _) => {}
        }
        return predicates;
    }
}

impl<'a> fmt::Display for Formula<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Formula::Empty => write!(f, "∅"),
            Formula::Atom(predicate) => write!(f, "{}", predicate.to_string()),
            Formula::Not(inner) => write!(f, "¬({})", inner),
            Formula::And(terms) => {
                let terms_str = terms.iter().map(|term| format!("{}", term)).collect::<Vec<_>>().join(" ∧ ");
                write!(f, "({})", terms_str)
            }
            Formula::Or(terms) => {
                let terms_str = terms.iter().map(|term| format!("{}", term)).collect::<Vec<_>>().join(" ∨ ");
                write!(f, "({})", terms_str)
            }
            Formula::Xor(terms) => {
                let terms_str = terms.iter().map(|term| format!("{}", term)).collect::<Vec<_>>().join(" ⊕ ");
                write!(f, "({})", terms_str)
            }
            Formula::Imply(lhs, rhs) => {
                let lhs_str = lhs.iter().map(|term| format!("{}", term)).collect::<Vec<_>>().join(" ∧ ");
                let rhs_str = rhs.iter().map(|term| format!("{}", term)).collect::<Vec<_>>().join(" ∧ ");
                write!(f, "({}) ⇒ ({})", lhs_str, rhs_str)
            }
            Formula::Exists(vars, inner) => {
                let vars_str = vars.iter().map(|var| format!("{}", var.name)).collect::<Vec<_>>().join(", ");
                write!(f, "∃{}: {}", vars_str, inner)
            }
            Formula::ForAll(vars, inner) => {
                let vars_str = vars.iter().map(|var| format!("{}", var.name)).collect::<Vec<_>>().join(", ");
                write!(f, "∀{}: {}", vars_str, inner)
            }
            Formula::Weighted(weight, terms) => {
                write!(f, "weight: {}, formula: {}", weight, terms)
            }
            Formula::Equals(lhs, rhs) => write!(f, "{} = {}", lhs, rhs),
        }
    }
}