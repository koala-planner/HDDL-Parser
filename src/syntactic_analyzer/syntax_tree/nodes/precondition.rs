use core::panic;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::transpiler::format_typed_list;

use super::*;

// a formula in disjunctive normal form: a disjunction of cubes
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

    pub fn into_formula(self) -> Precondition<'a> {
        match self {
            Literal::Positive(predicate) => Precondition::Atom(predicate),
            Literal::Negative(predicate) => {
                Precondition::Not(Box::new(Precondition::Atom(predicate)))
            }
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

    pub fn into_formula(mut self) -> Option<Precondition<'a>> {
        match self.values.len() {
            0 => None,
            1 => Some(self.values.pop().unwrap().into_formula()),
            _ => Some(Precondition::And(
                self.values
                    .into_iter()
                    .map(|literal| Box::new(literal.into_formula()))
                    .collect(),
            )),
        }
    }

    pub fn to_formula(&self) -> Precondition<'a> {
        self.clone().into_formula().unwrap_or(Precondition::Empty)
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
            Precondition::Or(
                self.cubes
                    .iter()
                    .map(|cube| Box::new(cube.to_formula()))
                    .collect()
            )
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Precondition<'a> {
    Empty,
    Atom(Predicate<'a>),
    Not(Box<Precondition<'a>>),
    And(Vec<Box<Precondition<'a>>>),
    Or(Vec<Box<Precondition<'a>>>),
    Xor(Vec<Box<Precondition<'a>>>),
    // formula -> formula'
    Imply(Box<Precondition<'a>>, Box<Precondition<'a>>),
    // ∃vars: formula
    Exists(Vec<Symbol<'a>>, Box<Precondition<'a>>),
    // ∀vars: formula
    ForAll(Vec<Symbol<'a>>, Box<Precondition<'a>>),
    // formula = formula'
    Equals(#[serde(borrow)] &'a str, &'a str),
}

impl<'a> Precondition<'a> {
    pub fn to_dnf(&self) -> DNFFormula<'a> {
        let cubes = self
            .dnf_cubes()
            .into_iter()
            .map(|cube| Cube {
                values: cube
                    .into_iter()
                    .filter_map(|entry| match entry {
                        Precondition::Atom(predicate) => Some(Literal::Positive(predicate)),
                        Precondition::Not(inner) => match *inner {
                            Precondition::Atom(predicate) => Some(Literal::Negative(predicate)),
                            other => panic!(
                                "to_dnf: (not {other}) is not in NNF"
                            ),
                        },
                        Precondition::Empty => None,
                        Precondition::Equals(lhs, rhs) => panic!(
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
    pub fn dnf_cubes(&self) -> Vec<Vec<Precondition<'a>>> {
        fn cubes<'a>(f: &Precondition<'a>) -> Vec<Vec<Precondition<'a>>> {
            match f {
                Precondition::Or(fs) => fs.iter().flat_map(|f| cubes(f)).collect(),
                Precondition::And(fs) => fs.iter().fold(vec![vec![]], |acc, f| {
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
                Precondition::Exists(vars, f) => cubes(f)
                    .into_iter()
                    .map(|cube| {
                        vec![Precondition::Exists(
                            vars.clone(),
                            Box::new(Precondition::And(cube.into_iter().map(Box::new).collect())),
                        )]
                    })
                    .collect(),
                literal => vec![vec![literal.clone()]],
            }
        }

        cubes(&self.to_nnf(false))
    }

    pub fn to_nnf(&self, negated: bool) -> Precondition<'a> {
        match (self, negated) {
            (Precondition::Empty | Precondition::Atom(_), false) => self.clone(),
            (Precondition::Empty | Precondition::Atom(_), true) => {
                Precondition::Not(Box::new(self.clone()))
            }
            (Precondition::Not(f), _) => f.to_nnf(!negated),
            (Precondition::And(fs), false) | (Precondition::Or(fs), true) => {
                Precondition::And(fs.iter().map(|f| Box::new(f.to_nnf(negated))).collect())
            }
            (Precondition::Or(fs), false) | (Precondition::And(fs), true) => {
                Precondition::Or(fs.iter().map(|f| Box::new(f.to_nnf(negated))).collect())
            }
            (Precondition::ForAll(vars, f), false) | (Precondition::Exists(vars, f), true) => {
                Precondition::ForAll(vars.clone(), Box::new(f.to_nnf(negated)))
            }
            (Precondition::Exists(vars, f), false) | (Precondition::ForAll(vars, f), true) => {
                Precondition::Exists(vars.clone(), Box::new(f.to_nnf(negated)))
            }
            (Precondition::Imply(ps, qs), _) => {
                Precondition::Or(vec![Box::new(Precondition::Not(ps.clone())), qs.clone()])
                    .to_nnf(negated)
            }
            (Precondition::Xor(fs), false) => {
                // exactly one: some fᵢ holds and every other operand fails
                let mut disjuncts = vec![];
                for i in 0..fs.len() {
                    let mut conjuncts = vec![Box::new(fs[i].to_nnf(false))];
                    for j in (0..fs.len()).filter(|&j| j != i) {
                        conjuncts.push(Box::new(fs[j].to_nnf(true)));
                    }
                    disjuncts.push(Box::new(Precondition::And(conjuncts)));
                }
                Precondition::Or(disjuncts)
            }
            (Precondition::Xor(fs), true) => {
                // negated: none holds, or some pair holds (~f1 ^ ... ^ ~fn) v (f1 ^ f2) v ... (fn-1 ^ fn)
                let mut disjuncts = vec![Box::new(Precondition::And(
                    fs.iter().map(|f| Box::new(f.to_nnf(true))).collect(),
                ))];
                for i in 0..fs.len() {
                    for j in i + 1..fs.len() {
                        disjuncts.push(Box::new(Precondition::And(vec![
                            Box::new(fs[i].to_nnf(false)),
                            Box::new(fs[j].to_nnf(false)),
                        ])));
                    }
                }
                Precondition::Or(disjuncts)
            }
            (Precondition::Equals(_, _), false) => self.clone(),
            (Precondition::Equals(_, _), true) => Precondition::Not(Box::new(self.clone())),
        }
    }

    // the predicates mentioned by the formula; those under a quantifier are
    // only included on request, as their bound variables are not parameters
    pub fn get_predicates(&self, include_quantified: bool) -> Vec<&Predicate<'a>> {
        let mut predicates = vec![];
        match self {
            Precondition::Empty | Precondition::Equals(_, _) => {}
            Precondition::Atom(predicate) => {
                predicates.push(predicate);
            }
            Precondition::Not(inner) => {
                predicates.extend(inner.get_predicates(include_quantified));
            }
            Precondition::And(terms) | Precondition::Or(terms) | Precondition::Xor(terms) => {
                for term in terms {
                    predicates.extend(term.get_predicates(include_quantified));
                }
            }
            Precondition::Imply(lhs, rhs) => {
                predicates.extend(lhs.get_predicates(include_quantified));
                predicates.extend(rhs.get_predicates(include_quantified));
            }
            Precondition::ForAll(_, body) | Precondition::Exists(_, body) => {
                if include_quantified {
                    predicates.extend(body.get_predicates(true));
                }
            }
        }
        predicates
    }

    pub fn is_simple_conjunction(&self) -> bool {
        match self {
            Precondition::Empty => true,
            Precondition::And(conjuncts) => conjuncts.iter().all(|c| c.is_literal()),
            other => other.is_literal(),
        }
    }

    pub fn and(self, rhs: Precondition<'a>) -> Precondition<'a> {
        let mut conjuncts = match self {
            Precondition::And(conjuncts) => conjuncts,
            other => vec![Box::new(other)],
        };
        match rhs {
            Precondition::And(rhs) => conjuncts.extend(rhs),
            other => conjuncts.push(Box::new(other)),
        }
        Precondition::And(conjuncts)
    }

    pub(crate) fn is_literal(&self) -> bool {
        match self {
            Precondition::Atom(_) => true,
            Precondition::Not(inner) => {
                matches!(&**inner, Precondition::Atom(_) | Precondition::Equals(_, _))
            }
            _ => false,
        }
    }

    pub fn is_quantified(&self) -> bool {
        match self {
            Precondition::Empty | Precondition::Atom(_) | Precondition::Equals(_, _) => false,
            Precondition::And(inner) | Precondition::Or(inner) | Precondition::Xor(inner) => {
                inner.iter().any(|x| x.is_quantified())
            }
            Precondition::Exists(_, _) | Precondition::ForAll(_, _) => true,
            Precondition::Not(inner) => inner.is_quantified(),
            Precondition::Imply(lhs, rhs) => lhs.is_quantified() || rhs.is_quantified(),
        }
    }

    // ground substitution of the variable `var` by `value`
    pub fn substitute(&self, var: &str, value: &Symbol<'a>) -> Precondition<'a> {
        match self {
            Precondition::Empty => Precondition::Empty,
            Precondition::Atom(predicate) => Precondition::Atom(predicate.substitute(var, value)),
            Precondition::Not(inner) => Precondition::Not(Box::new(inner.substitute(var, value))),
            Precondition::And(terms) => Precondition::And(
                terms
                    .iter()
                    .map(|term| Box::new(term.substitute(var, value)))
                    .collect(),
            ),
            Precondition::Or(terms) => Precondition::Or(
                terms
                    .iter()
                    .map(|term| Box::new(term.substitute(var, value)))
                    .collect(),
            ),
            Precondition::Xor(terms) => Precondition::Xor(
                terms
                    .iter()
                    .map(|term| Box::new(term.substitute(var, value)))
                    .collect(),
            ),
            Precondition::Imply(lhs, rhs) => Precondition::Imply(
                Box::new(lhs.substitute(var, value)),
                Box::new(rhs.substitute(var, value)),
            ),
            Precondition::Exists(vars, body) => {
                if vars.iter().any(|v| v.name == var) {
                    self.clone()
                } else {
                    Precondition::Exists(vars.clone(), Box::new(body.substitute(var, value)))
                }
            }
            Precondition::ForAll(vars, body) => {
                if vars.iter().any(|v| v.name == var) {
                    self.clone()
                } else {
                    Precondition::ForAll(vars.clone(), Box::new(body.substitute(var, value)))
                }
            }
            Precondition::Equals(lhs, rhs) => Precondition::Equals(
                if *lhs == var { value.name } else { lhs },
                if *rhs == var { value.name } else { rhs },
            ),
        }
    }

    // whether any subformula (including self) satisfies the predicate; the
    // walk descends into quantifier bodies.
    pub fn any_subformula<F: FnMut(&Precondition<'a>) -> bool>(&self, predicate: &mut F) -> bool {
        if predicate(self) {
            return true;
        }
        match self {
            Precondition::Empty | Precondition::Atom(_) | Precondition::Equals(_, _) => false,
            Precondition::Not(inner)
            | Precondition::Exists(_, inner)
            | Precondition::ForAll(_, inner) => inner.any_subformula(predicate),
            Precondition::And(terms) | Precondition::Or(terms) | Precondition::Xor(terms) => {
                terms.iter().any(|term| term.any_subformula(predicate))
            }
            Precondition::Imply(lhs, rhs) => {
                lhs.any_subformula(predicate) || rhs.any_subformula(predicate)
            }
        }
    }
}

impl<'a> fmt::Display for Precondition<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // nested renderings are shifted one level so multi-line children align
        fn shift<T: fmt::Display>(term: &T) -> String {
            term.to_string().replace('\n', "\n\t")
        }
        // a connective with a single term stays inline, more get one line each
        fn connective(name: &str, terms: &[Box<Precondition>]) -> String {
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
            Precondition::Empty => write!(f, "()"),
            Precondition::Atom(predicate) => {
                write!(f, "({}", predicate.name)?;
                for var in predicate.variables.iter() {
                    write!(f, " {}", var.name)?;
                }
                write!(f, ")")
            }
            Precondition::Not(inner) => write!(f, "(not {})", shift(inner)),
            Precondition::And(terms) => write!(f, "{}", connective("and", terms)),
            Precondition::Or(terms) => write!(f, "{}", connective("or", terms)),
            Precondition::Xor(terms) => write!(f, "{}", connective("oneof", terms)),
            Precondition::Imply(lhs, rhs) => {
                write!(
                    f,
                    "(when {} {})",
                    shift(&lhs.to_string()),
                    shift(&rhs.to_string())
                )
            }
            Precondition::Exists(vars, inner) => {
                write!(f, "(exists ({}) {})", format_typed_list(vars), shift(inner))
            }
            Precondition::ForAll(vars, inner) => {
                write!(f, "(forall ({}) {})", format_typed_list(vars), shift(inner))
            }
            Precondition::Equals(lhs, rhs) => write!(f, "(= {} {})", lhs, rhs),
        }
    }
}
