use core::panic;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::transpiler::format_typed_list;
use crate::NumberType;

use super::*;

// a formula in disjunctive normal form: a disjunction of cubes, each a
// conjunction of literals. Transient computation result (never serialized).
#[derive(Clone, Debug)]
pub struct DNFFormula<'a> {
    pub cubes: Vec<Cube<'a>>,
}

#[derive(Clone, Debug)]
pub enum Literal<'a> {
    Positive(Predicate<'a>),
    Negative(Predicate<'a>),
}

#[derive(Clone, Debug)]
pub struct Cube<'a> {
    pub values: Vec<Literal<'a>>,
}

impl<'a> Literal<'a> {
    pub fn predicate(&self) -> &Predicate<'a> {
        match self {
            Literal::Positive(predicate) | Literal::Negative(predicate) => predicate,
        }
    }

    pub fn is_positive(&self) -> bool {
        matches!(self, Literal::Positive(_))
    }

    pub fn into_formula(self) -> Formula<'a> {
        match self {
            Literal::Positive(predicate) => Formula::Atom(predicate),
            Literal::Negative(predicate) => Formula::Not(Box::new(Formula::Atom(predicate))),
        }
    }
}

impl<'a> Cube<'a> {
    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn into_formula(mut self) -> Option<Formula<'a>> {
        match self.values.len() {
            0 => None,
            1 => Some(self.values.pop().unwrap().into_formula()),
            _ => Some(Formula::And(
                self.values
                    .into_iter()
                    .map(|literal| Box::new(literal.into_formula()))
                    .collect(),
            )),
        }
    }

    pub fn to_formula(&self) -> Formula<'a> {
        self.clone().into_formula().unwrap_or(Formula::Empty)
    }
}

impl<'a> DNFFormula<'a> {
    pub fn len(&self) -> usize {
        self.cubes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cubes.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Cube<'a>> {
        self.cubes.iter()
    }
}

impl<'a> IntoIterator for DNFFormula<'a> {
    type Item = Cube<'a>;
    type IntoIter = std::vec::IntoIter<Cube<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.cubes.into_iter()
    }
}

impl<'a, 'b> IntoIterator for &'b DNFFormula<'a> {
    type Item = &'b Cube<'a>;
    type IntoIter = std::slice::Iter<'b, Cube<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.cubes.iter()
    }
}

impl<'a> fmt::Display for DNFFormula<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            Formula::Or(
                self.cubes
                    .iter()
                    .map(|cube| Box::new(cube.to_formula()))
                    .collect()
            )
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Formula<'a> {
    Empty,
    Atom(Predicate<'a>),
    Not(Box<Formula<'a>>),
    And(Vec<Box<Formula<'a>>>),
    Or(Vec<Box<Formula<'a>>>),
    Xor(Vec<Box<Formula<'a>>>),
    // formula -> formula'
    Imply(Box<Formula<'a>>, Box<Formula<'a>>),
    // ∃vars: formula
    Exists(Vec<Symbol<'a>>, Box<Formula<'a>>),
    // ∀vars: formula
    ForAll(Vec<Symbol<'a>>, Box<Formula<'a>>),
    // a formula holding with the given probability in [0, 1]
    Probabilistic(NumberType, Box<Formula<'a>>),
    // formula = formula'
    Equals(#[serde(borrow)] &'a str, &'a str),
}

impl<'a> Formula<'a> {
    pub fn to_dnf(&self) -> DNFFormula<'a> {
        let cubes = self
            .dnf_cubes()
            .into_iter()
            .map(|cube| Cube {
                values: cube
                    .into_iter()
                    .filter_map(|entry| match entry {
                        Formula::Atom(predicate) => Some(Literal::Positive(predicate)),
                        Formula::Not(inner) => match *inner {
                            Formula::Atom(predicate) => Some(Literal::Negative(predicate)),
                            other => panic!(
                                "to_dnf: (not {other}) is not in NNF"
                            ),
                        },
                        Formula::Empty => None,
                        Formula::Equals(lhs, rhs) => panic!(
                            "to_dnf: (= {lhs} {rhs}) is not a DNF literal; run remove-equality-constraints first"
                        ),
                        other => panic!("to_dnf: non-literal {other} in a cube"),
                    })
                    .collect(),
            })
            .collect();
        DNFFormula { cubes }
    }

    // the disjuncts of the formula's DNF, each a list of literals and
    // existentially quantified conjunctions
    pub fn dnf_cubes(&self) -> Vec<Vec<Formula<'a>>> {
        fn cubes<'a>(f: &Formula<'a>) -> Vec<Vec<Formula<'a>>> {
            match f {
                Formula::Or(fs) => fs.iter().flat_map(|f| cubes(f)).collect(),
                Formula::And(fs) => fs.iter().fold(vec![vec![]], |acc, f| {
                    let rights = cubes(f);
                    acc.iter()
                        .flat_map(|left| {
                            rights
                                .iter()
                                .map(move |right| left.iter().chain(right).cloned().collect())
                        })
                        .collect()
                }),
                // ∃x (C₁ ∨ … ∨ Cₖ) ≡ (∃x C₁) ∨ … ∨ (∃x Cₖ)
                Formula::Exists(vars, f) => cubes(f)
                    .into_iter()
                    .map(|cube| {
                        vec![Formula::Exists(
                            vars.clone(),
                            Box::new(Formula::And(cube.into_iter().map(Box::new).collect())),
                        )]
                    })
                    .collect(),
                literal => vec![vec![literal.clone()]],
            }
        }

        cubes(&self.to_nnf(false))
    }

    pub fn to_nnf(&self, negated: bool) -> Formula<'a> {
        match (self, negated) {
            (Formula::Empty | Formula::Atom(_), false) => self.clone(),
            (Formula::Empty | Formula::Atom(_), true) => Formula::Not(Box::new(self.clone())),
            (Formula::Not(f), _) => f.to_nnf(!negated),
            (Formula::And(fs), false) | (Formula::Or(fs), true) => {
                Formula::And(fs.iter().map(|f| Box::new(f.to_nnf(negated))).collect())
            }
            (Formula::Or(fs), false) | (Formula::And(fs), true) => {
                Formula::Or(fs.iter().map(|f| Box::new(f.to_nnf(negated))).collect())
            }
            (Formula::ForAll(vars, f), false) | (Formula::Exists(vars, f), true) => {
                Formula::ForAll(vars.clone(), Box::new(f.to_nnf(negated)))
            }
            (Formula::Exists(vars, f), false) | (Formula::ForAll(vars, f), true) => {
                Formula::Exists(vars.clone(), Box::new(f.to_nnf(negated)))
            }
            (Formula::Imply(ps, qs), _) => {
                Formula::Or(vec![Box::new(Formula::Not(ps.clone())), qs.clone()]).to_nnf(negated)
            }
            (Formula::Xor(fs), false) => {
                // exactly one: some fᵢ holds and every other operand fails
                let mut disjuncts = vec![];
                for i in 0..fs.len() {
                    let mut conjuncts = vec![Box::new(fs[i].to_nnf(false))];
                    for j in (0..fs.len()).filter(|&j| j != i) {
                        conjuncts.push(Box::new(fs[j].to_nnf(true)));
                    }
                    disjuncts.push(Box::new(Formula::And(conjuncts)));
                }
                Formula::Or(disjuncts)
            }
            (Formula::Xor(fs), true) => {
                // negated: none holds, or some pair holds (~f1 ^ ... ^ ~fn) v (f1 ^ f2) v ... (fn-1 ^ fn)
                let mut disjuncts = vec![Box::new(Formula::And(
                    fs.iter().map(|f| Box::new(f.to_nnf(true))).collect(),
                ))];
                for i in 0..fs.len() {
                    for j in i + 1..fs.len() {
                        disjuncts.push(Box::new(Formula::And(vec![
                            Box::new(fs[i].to_nnf(false)),
                            Box::new(fs[j].to_nnf(false)),
                        ])));
                    }
                }
                Formula::Or(disjuncts)
            }
            (Formula::Equals(_, _), false) => self.clone(),
            (Formula::Equals(_, _), true) => Formula::Not(Box::new(self.clone())),
            (Formula::Probabilistic(_, _), _) => {
                panic!("probabilistic formulae are not supported.")
            }
        }
    }

    pub fn get_propositional_predicates(&self) -> Vec<&Predicate<'a>> {
        let mut predicates = vec![];
        match self {
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
                predicates.extend(ps.get_propositional_predicates().iter());

                predicates.extend(qs.get_propositional_predicates().iter());
            }
            Formula::Probabilistic(_, q) => {
                predicates.extend(q.get_propositional_predicates().iter());
            }
            Formula::Equals(_, _) => {}
            // not propositional
            Formula::ForAll(_, _) | Formula::Exists(_, _) => {}
        }
        predicates
    }

    pub fn is_simple_conjunction(&self) -> bool {
        match self {
            Formula::Empty => true,
            Formula::And(conjuncts) => conjuncts.iter().all(|c| c.is_literal()),
            other => other.is_literal(),
        }
    }

    pub fn and(self, rhs: Formula<'a>) -> Formula<'a> {
        let mut conjuncts = match self {
            Formula::And(conjuncts) => conjuncts,
            other => vec![Box::new(other)],
        };
        match rhs {
            Formula::And(rhs) => conjuncts.extend(rhs),
            other => conjuncts.push(Box::new(other)),
        }
        Formula::And(conjuncts)
    }

    pub(crate) fn is_literal(&self) -> bool {
        match self {
            Formula::Atom(_) => true,
            Formula::Not(inner) => matches!(&**inner, Formula::Atom(_) | Formula::Equals(_, _)),
            _ => false,
        }
    }

    pub fn is_quantified(&self) -> bool {
        match self {
            Formula::Empty | Formula::Atom(_) | Formula::Equals(_, _) => false,
            Formula::And(inner) | Formula::Or(inner) | Formula::Xor(inner) => {
                let any_quantified = inner.iter().any(|x| x.is_quantified());
                any_quantified
            }
            Formula::Exists(_, _) | Formula::ForAll(_, _) => true,
            Formula::Not(inner) => inner.is_quantified(),
            Formula::Imply(lhs, rhs) => lhs.is_quantified() || rhs.is_quantified(),
            Formula::Probabilistic(_, inner) => inner.is_quantified(),
        }
    }

    // ground substitution of the variable `var` by `value`
    pub fn substitute(&self, var: &str, value: &Symbol<'a>) -> Formula<'a> {
        match self {
            Formula::Empty => Formula::Empty,
            Formula::Atom(predicate) => {
                let mut predicate = predicate.clone();
                for symbol in predicate.variables.iter_mut() {
                    if symbol.name == var {
                        *symbol = value.clone();
                    }
                }
                Formula::Atom(predicate)
            }
            Formula::Not(inner) => Formula::Not(Box::new(inner.substitute(var, value))),
            Formula::And(terms) => Formula::And(
                terms
                    .iter()
                    .map(|term| Box::new(term.substitute(var, value)))
                    .collect(),
            ),
            Formula::Or(terms) => Formula::Or(
                terms
                    .iter()
                    .map(|term| Box::new(term.substitute(var, value)))
                    .collect(),
            ),
            Formula::Xor(terms) => Formula::Xor(
                terms
                    .iter()
                    .map(|term| Box::new(term.substitute(var, value)))
                    .collect(),
            ),
            Formula::Imply(lhs, rhs) => Formula::Imply(
                Box::new(lhs.substitute(var, value)),
                Box::new(rhs.substitute(var, value)),
            ),
            Formula::Exists(vars, body) => {
                if vars.iter().any(|v| v.name == var) {
                    self.clone()
                } else {
                    Formula::Exists(vars.clone(), Box::new(body.substitute(var, value)))
                }
            }
            Formula::ForAll(vars, body) => {
                if vars.iter().any(|v| v.name == var) {
                    self.clone()
                } else {
                    Formula::ForAll(vars.clone(), Box::new(body.substitute(var, value)))
                }
            }
            Formula::Probabilistic(probability, inner) => {
                Formula::Probabilistic(probability.clone(), Box::new(inner.substitute(var, value)))
            }
            Formula::Equals(lhs, rhs) => Formula::Equals(
                if *lhs == var { value.name } else { lhs },
                if *rhs == var { value.name } else { rhs },
            ),
        }
    }

    // whether any subformula (including self) satisfies the predicate; unlike
    // get_propositional_predicates, the walk descends into quantifier bodies.
    pub fn any_subformula<F: FnMut(&Formula<'a>) -> bool>(&self, predicate: &mut F) -> bool {
        if predicate(self) {
            return true;
        }
        match self {
            Formula::Empty | Formula::Atom(_) | Formula::Equals(_, _) => false,
            Formula::Not(inner)
            | Formula::Probabilistic(_, inner)
            | Formula::Exists(_, inner)
            | Formula::ForAll(_, inner) => inner.any_subformula(predicate),
            Formula::And(terms) | Formula::Or(terms) | Formula::Xor(terms) => {
                terms.iter().any(|term| term.any_subformula(predicate))
            }
            Formula::Imply(lhs, rhs) => {
                lhs.any_subformula(predicate) || rhs.any_subformula(predicate)
            }
        }
    }
}

impl<'a> fmt::Display for Formula<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // nested renderings are shifted one level so multi-line children align
        fn shift<T: fmt::Display>(term: &T) -> String {
            term.to_string().replace('\n', "\n\t")
        }
        // a connective with a single term stays inline, more get one line each
        fn connective(name: &str, terms: &[Box<Formula>]) -> String {
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
            Formula::Empty => write!(f, "()"),
            Formula::Atom(predicate) => {
                write!(f, "({}", predicate.name)?;
                for var in predicate.variables.iter() {
                    write!(f, " {}", var.name)?;
                }
                write!(f, ")")
            }
            Formula::Not(inner) => write!(f, "(not {})", shift(inner)),
            Formula::And(terms) => write!(f, "{}", connective("and", terms)),
            Formula::Or(terms) => write!(f, "{}", connective("or", terms)),
            Formula::Xor(terms) => write!(f, "{}", connective("oneof", terms)),
            Formula::Imply(lhs, rhs) => {
                write!(
                    f,
                    "(when {} {})",
                    shift(&lhs.to_string()),
                    shift(&rhs.to_string())
                )
            }
            Formula::Exists(vars, inner) => {
                write!(f, "(exists ({}) {})", format_typed_list(vars), shift(inner))
            }
            Formula::ForAll(vars, inner) => {
                write!(f, "(forall ({}) {})", format_typed_list(vars), shift(inner))
            }
            Formula::Probabilistic(probability, terms) => {
                write!(f, "(probabilistic {} {})", probability, shift(terms))
            }
            Formula::Equals(lhs, rhs) => write!(f, "(= {} {})", lhs, rhs),
        }
    }
}
