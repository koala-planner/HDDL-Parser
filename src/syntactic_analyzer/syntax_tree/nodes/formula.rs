use std::{collections::HashMap, fmt};

extern crate Robinson;
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

    pub fn to_cnf(&self) -> Formula<'a> {
        let simplified = self.simplify();
        let nnf = simplified.to_nnf();
        return nnf.distribute_disjunction();
    }

    fn simplify(&self) -> Formula<'a> {
        match self {
            Formula::Empty => Formula::Empty,
            Formula::Atom(_) => self.clone(),
            Formula::Not(f) => {
                match &**f {
                    Formula::Not(sub_f) => {
                        sub_f.as_ref().clone()
                    },
                    Formula::Equals(a, b) => {
                        Formula::Xor(vec![
                            Box::new(Formula::Atom(Predicate::new_dummy(a))),
                            Box::new(Formula::Atom(Predicate::new_dummy(b))),
                        ]).simplify()
                    }
                    _ => {
                        Formula::Not(Box::new(f.simplify()))
                    }
                }
            },
            Formula::And(fs) => {
                let mut conjuncts = vec![];
                for f in fs {
                    match &**f {
                        Formula::And(sub_fs) => {
                            for sub_f in sub_fs {
                                conjuncts.push(Box::new(sub_f.simplify()));
                            }
                        }
                        other => {
                            conjuncts.push(Box::new(other.simplify()));
                        }
                    }
                }
                Formula::And(conjuncts)
            },
            Formula::Or(fs) => {
                let mut disjuncts = vec![];
                for f in fs {
                    match &**f {
                        Formula::Or(sub_fs) => {
                            for sub_f in sub_fs {
                                disjuncts.push(Box::new(sub_f.simplify()));
                            }
                        }
                        other => {
                            disjuncts.push(Box::new(other.simplify()));
                        }
                    }
                }
                Formula::Or(disjuncts)
            },
            Formula::Xor(fs) => {
                // Convert XOR to a combination of AND, OR, and NOT
                let new_fs: Vec<Box<Formula>> = fs.iter().map(|f| {
                    Box::new(f.simplify())
                }).collect();
                let mut result = vec![];
                for (i, f_1) in new_fs.iter().enumerate() {
                    let mut clause = vec![f_1.clone()];
                    for (j, f_2) in new_fs.iter().enumerate() {
                        if i == j {
                            continue;
                        }
                        clause.push(Box::new(Formula::Not(f_2.clone())));
                    }
                    result.push(Box::new(Formula::Or(clause)));
                }
                result.push(Box::new(Formula::Or(new_fs)));
                Formula::And(result)
            }
            Formula::Imply(antecedents, consequents) => {
                let not_antecedents: Box<Formula<'a>> = Box::new(Formula::And(
                    antecedents.iter().map(|f| Box::new(f.simplify())).collect(),
                ));
                let consequents: Box<Formula<'a>> = Box::new(Formula::And(
                    consequents.iter().map(|f| Box::new(f.simplify())).collect(),
                ));
                Formula::Or(vec![not_antecedents, consequents])
            }
            Formula::Exists(quantifier, f) => {
                Formula::Exists(quantifier.clone(), Box::new(f.simplify()))
            }
            Formula::ForAll(quantifier, f) => {
                Formula::ForAll(quantifier.clone(), Box::new(f.simplify()))
            }
            Formula::Weighted(weight, f ) => {
                Formula::Weighted(weight.clone(), Box::new(f.simplify()))
            }
            Formula::Equals(a, b) => {
                // a = b -> (a ^ b) v (~a ^ ~b)
                let pred_a = Box::new(
                    Formula::Atom(Predicate::new_dummy(&a))
                );
                let pred_b = Box::new(
                    Formula::Atom(Predicate::new_dummy(&b))
                );
                let pos_conjunct = Formula::And(vec![pred_a.clone(), pred_b.clone()]);
                let not_a = Box::new(Formula::Not(pred_a));
                let not_b = Box::new(Formula::Not(pred_b));
                let neg_conjunct = Formula::And(vec![not_a, not_b]);
                Formula::Or(vec![Box::new(pos_conjunct), Box::new(neg_conjunct)])
            },
        }
    }

    fn to_nnf(&self) -> Formula<'a> {
        match self {
            Formula::Empty => Formula::Empty,
            Formula::Atom(_) => self.clone(),
            Formula::Not(f) => match &**f {
                Formula::Empty => self.clone(),
                Formula::Atom(p) => self.clone(),
                Formula::Not(g) => g.to_nnf(),
                Formula::And(fs) => Formula::Or(
                    fs.iter()
                        .map(|f| Box::new(Formula::Not(Box::new(f.to_nnf()))))
                        .collect(),
                ),
                Formula::Or(fs) => Formula::And(
                    fs.iter()
                        .map(|f| Box::new(Formula::Not(Box::new(f.to_nnf()))))
                        .collect(),
                ),
                Formula::Exists(quantifier, f) => Formula::ForAll(
                    quantifier.clone(),
                    Box::new(Formula::Not(Box::new(f.to_nnf()))),
                ),
                Formula::ForAll(quantifier, f) => Formula::Exists(
                    quantifier.clone(),
                    Box::new(Formula::Not(Box::new(f.to_nnf()))),
                ),
                Formula::Weighted(weight, f) => Formula::Weighted(
                    weight.clone(),
                    Box::new(f.to_nnf())
                ),
                //
                Formula::Xor(_) | Formula::Imply(_, _) | Formula::Equals(_, _) => unreachable!("not simplified")
            },
            Formula::And(fs) => Formula::And(fs.iter().map(|f| Box::new(f.to_nnf())).collect()),
            Formula::Or(fs) => Formula::Or(fs.iter().map(|f| Box::new(f.to_nnf())).collect()),
            Formula::ForAll(quantifier, f) => {
                Formula::ForAll(quantifier.clone(), Box::new(f.to_nnf()))
            }
            Formula::Exists(quantifier, f) => {
                Formula::Exists(quantifier.clone(), Box::new(f.to_nnf()))
            }
            _ => unreachable!("Formula is not simplified"),
        }
    }

    fn distribute_disjunction(&self) -> Formula<'a> {
        match self {
            Formula::Empty | Formula::Atom(_) | Formula::Not(_) => self.clone(),
            Formula::And(fs) => Formula::And(
                fs.iter()
                    .map(|f| Box::new(f.distribute_disjunction()))
                    .collect(),
            ),
            Formula::Or(fs) => {
                let distributed: Vec<Box<Formula<'a>>> = fs
                    .iter()
                    .map(|f| Box::new(f.distribute_disjunction()))
                    .collect();
                let mut result = Vec::new();
                let mut queue = vec![distributed];
                while let Some(current) = queue.pop() {
                    if let Some(position) =
                        current.iter().position(|f| matches!(**f, Formula::And(_)))
                    {
                        let Formula::And(conjuncts) = &*current[position] else {
                            unreachable!()
                        };
                        for conjunct in conjuncts {
                            let mut new_formula = current.clone();
                            new_formula[position] = conjunct.clone();
                            queue.push(new_formula);
                        }
                    } else {
                        result.push(Formula::Or(current));
                    }
                }
                if result.len() == 1 {
                    result.pop().unwrap()
                } else {
                    Formula::And(result.into_iter().map(Box::new).collect())
                }
            }
            Formula::Exists(q, vars) => {
                Formula::Exists(q.clone(), Box::new(vars.distribute_disjunction()))
            }
            Formula::ForAll(q, vars) => {
                Formula::ForAll(q.clone(), Box::new(vars.distribute_disjunction()))
            }
            _ => unreachable!("formula is not simplified"),
        }
    }

    fn drop_quantifiers(&self) -> Formula<'a> {
        match self {
            Formula::Empty => {},
            Formula::Atom(_) => {},
            Formula::Not(f) => {
                return Formula::Not(Box::new(f.drop_quantifiers()))
            }
            Formula::And(fs) => {
                return Formula::And(fs.iter().map(|f| {
                    Box::new(f.drop_quantifiers())
                }).collect());
            }
            Formula::Or(fs) => {
                return Formula::Or(fs.iter().map(|f| {
                    Box::new(f.drop_quantifiers())
                }).collect());
            }
            Formula::Xor(fs) => {
                return Formula::Xor(fs.iter().map(|f| {
                    Box::new(f.drop_quantifiers())
                }).collect());
            }
            Formula::Imply(ps, qs) => {
                let new_ps = ps.iter().map(|p| {
                    Box::new(p.drop_quantifiers())
                }).collect();
                let new_qs = qs.iter().map(|q| {
                    Box::new(q.drop_quantifiers())
                }).collect();
                return Formula::Imply(new_ps, new_qs);
            }
            Formula::Equals(_, _) => {}
            Formula::ForAll(_, _) | Formula::Exists(_, _) | Formula::Weighted(_, _) => {return Formula::Empty}
        }
        self.clone()
    }

    fn to_clauses(&self) -> (u32, Vec<Vec<i32>>) {
        let mut literal_ids = HashMap::new();
        let mut clauses: Vec<Vec<i32>> = vec![];
        let mut count = 1;
        let propositional = self.drop_quantifiers();
        match propositional.to_cnf() {
            Formula::Empty => {
                return (0, vec![]);
            }
            Formula::And(subformula) => {
                for f in subformula {
                    let mut clause: Vec<i32> = vec![];
                    match *f {
                        Formula::Empty => {}
                        Formula::Atom(predicate) => {
                            if !literal_ids.contains_key(&predicate.to_string()) {
                                literal_ids.insert(predicate.to_string(), count);
                                count+=1;
                            }
                            clause.push(*literal_ids.get(&predicate.to_string()).unwrap());
                        }
                        Formula::Not(pred_box) => {
                            if let Formula::Atom(predicate) = *pred_box {
                                if !literal_ids.contains_key(&predicate.to_string()) {
                                    literal_ids.insert(predicate.to_string(), count);
                                    count+=1;
                                }
                                clause.push(-1 * literal_ids.get(&predicate.to_string()).unwrap().clone());
                            } else {
                                panic!("not simplified")
                            }
                        }
                        Formula::Or(disjuncts) => {
                            for disjunct in disjuncts {
                                match *disjunct {
                                    Formula::Atom(predicate) => {
                                        if !literal_ids.contains_key(&predicate.to_string()) {
                                            literal_ids.insert(predicate.to_string(), count);
                                            count+=1;
                                        }
                                        clause.push(*literal_ids.get(&predicate.to_string()).unwrap());
                                    },
                                    Formula::Not(pred_box) => {
                                        if let Formula::Atom(predicate) = *pred_box {
                                            if !literal_ids.contains_key(&predicate.to_string()) {
                                                literal_ids.insert(predicate.to_string(), count);
                                                count+=1;
                                            }
                                            clause.push(-1 * literal_ids.get(&predicate.to_string()).unwrap().clone());
                                        } else {
                                            panic!("not simplified: {:?}", *pred_box)
                                        }
                                    }
                                    _ => panic!("not in CNF")
                                }
                            }
                        }
                        token => panic!("not in CNF, found {:?}", token)

                    }
                    clauses.push(clause);
                }
            }
            Formula::Or(disjuncts) => {
                for disjunct in disjuncts.iter() {
                    let mut clause = vec![];
                    match &**disjunct {
                        Formula::Atom(predicate) => {
                            if !literal_ids.contains_key(&predicate.to_string()) {
                                literal_ids.insert(predicate.to_string(), count);
                                count+=1;
                            }
                        },
                        Formula::Not(pred_box) => {
                            if let Formula::Atom(predicate) = &**pred_box {
                                if !literal_ids.contains_key(&predicate.to_string()) {
                                    literal_ids.insert(predicate.to_string(), count);
                                    count+=1;
                                }
                                clause.push(-1 * literal_ids.get(&predicate.to_string()).unwrap());
                            } else {
                                panic!("not simplified")
                            }
                        }
                        _ => panic!("not in CNF")
                    }
                    return (disjuncts.len() as u32, vec![clause]);
                }
            }
            Formula::Not(p) => {
                if let Formula::Atom(predicate) = *p {
                    if !literal_ids.contains_key(&predicate.to_string()) {
                        literal_ids.insert(predicate.to_string(), count);
                        count+=1;
                    }
                    let clause = vec![-1 * literal_ids.get(&predicate.to_string()).unwrap()];
                    clauses.push(clause);
                } else {
                    panic!("not simplified")
                }
            }
            Formula::Atom(predicate) => {
                if !literal_ids.contains_key(&predicate.to_string()) {
                    literal_ids.insert(predicate.to_string(), count);
                    count+=1;
                }
                let clause = vec![literal_ids.get(&predicate.to_string()).unwrap().clone()];
                clauses.push(clause);
            }
            token => panic!("unexpected {:?}", token)
        }
        ((count - 1) as u32, clauses)
    }

    pub fn is_sat(&self) -> bool {
        let cnf = self.to_cnf();
        let (var_count, mut clauses) = cnf.to_clauses();
        Robinson::parser::preproc_and_solve(clauses.as_mut(), var_count as usize)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn demorgan_rules_test() {
        // ~(a^b) = ~a v ~b
        let f1 = Formula::Not(Box::new(Formula::And(vec![
            Box::new(Formula::Atom(Predicate::new_dummy("a"))),
            Box::new(Formula::Atom(Predicate::new_dummy("b"))),
        ])));
        let cnf = f1.to_cnf();
        match cnf {
            Formula::Or(a) => {
                match &*a[0] {
                    Formula::Not(p) => match &**p {
                        Formula::Atom(p) => {
                            assert_eq!(p.name, "a")
                        }
                        _ => panic!("wrong result"),
                    },
                    _ => panic!("wrong result"),
                }
                match &*a[1] {
                    Formula::Not(p) => match &**p {
                        Formula::Atom(p) => {
                            assert_eq!(p.name, "b")
                        }
                        _ => panic!("wrong result"),
                    },
                    _ => panic!("wrong result"),
                }
            }
            _ => panic!("wrong result"),
        }

        // ~(a v b) = ~a ^ ~b
        let f2 = Formula::Not(Box::new(Formula::Or(vec![
            Box::new(Formula::Atom(Predicate::new_dummy("a"))),
            Box::new(Formula::Atom(Predicate::new_dummy("b"))),
        ])));
        let cnf = f2.to_cnf();
        match cnf {
            Formula::And(a) => {
                match &*a[0] {
                    Formula::Not(p) => match &**p {
                        Formula::Atom(p) => {
                            assert_eq!(p.name, "a")
                        }
                        _ => panic!("wrong result"),
                    },
                    _ => panic!("wrong result"),
                }
                match &*a[1] {
                    Formula::Not(p) => match &**p {
                        Formula::Atom(p) => {
                            assert_eq!(p.name, "b")
                        }
                        _ => panic!("wrong result"),
                    },
                    _ => panic!("wrong result"),
                }
            }
            _ => panic!("wrong result"),
        }
    }

    #[test]
    pub fn cnf_clause_test() {
        let cnf = Formula::And(vec![
            Box::new(Formula::Or(vec![
                Box::new(Formula::Atom(Predicate::new_dummy("a"))),
                Box::new(Formula::Not(Box::new(Formula::Atom(Predicate::new_dummy("c"))))),
            ])),
            Box::new(Formula::Or(vec![
                Box::new(Formula::Atom(Predicate::new_dummy("b"))),
                Box::new(Formula::Atom(Predicate::new_dummy("c"))),
                Box::new(Formula::Not(Box::new(Formula::Atom(Predicate::new_dummy("a"))))),
            ])),
        ]);
        let (var_count, clauses) = cnf.to_clauses();
        assert_eq!(var_count, 3);
        assert_eq!(clauses.len(), 2);
        assert_eq!(clauses[0], vec![1, -2]);
        assert_eq!(clauses[1], vec![3, 2, -1]);
    }

    #[test]
    pub fn is_sat_test() {
        let cnf = Formula::And(vec![
            Box::new(Formula::Or(vec![
                Box::new(Formula::Atom(Predicate::new_dummy("a"))),
                Box::new(Formula::Not(Box::new(Formula::Atom(Predicate::new_dummy("c"))))),
            ])),
            Box::new(Formula::Or(vec![
                Box::new(Formula::Atom(Predicate::new_dummy("b"))),
                Box::new(Formula::Atom(Predicate::new_dummy("c"))),
                Box::new(Formula::Not(Box::new(Formula::Atom(Predicate::new_dummy("a"))))),
            ])),
        ]);
        assert_eq!(cnf.is_sat(), true);
    }

    #[test]
    pub fn xor_simplification_test() {
        // (a xor b) = (a+b).(~a+~b)
        let f1 = Formula::Xor(vec![
            Box::new(Formula::Atom(Predicate::new_dummy("a"))),
            Box::new(Formula::Atom(Predicate::new_dummy("b"))),
        ]);
        let cnf = f1.to_cnf();
        match cnf {
            Formula::And(a) => {
                match &*a[0] {
                    Formula::Or(disjuncts) => {
                        assert_eq!(disjuncts.len(), 2);
                        match &*disjuncts[0] {
                            Formula::Atom(predicate) => {
                                assert_eq!(predicate.name, "a");
                            }
                            token => panic!("{:?}", token)
                        }
                        match &*disjuncts[1] {
                            Formula::Not(pred_box) => {
                                if let Formula::Atom(predicate) = pred_box.as_ref() {
                                    assert_eq!(predicate.name, "b");
                                } else {
                                    panic!()
                                }
                            }
                            token => panic!("{:?}", token)
                        }
                    },
                    token => panic!("{:?}", token),
                }
                match &*a[1] {
                    Formula::Or(disjuncts) => {
                        assert_eq!(disjuncts.len(), 2);
                        match &*disjuncts[0] {
                            Formula::Atom(predicate) => {
                                assert_eq!(predicate.name, "b");
                            }
                            token => panic!("{:?}", token)
                        }
                        match &*disjuncts[1] {
                            Formula::Not(pred_box) => {
                                if let Formula::Atom(predicate) = pred_box.as_ref() {
                                    assert_eq!(predicate.name, "a");
                                } else {
                                    panic!()
                                }
                            }
                            token => panic!("{:?}", token)
                        }
                    },
                    token => panic!("{:?}", token),
                }
                match &*a[2] {
                    Formula::Or(disjuncts) => {
                        assert_eq!(disjuncts.len(), 2);
                        match &*disjuncts[0] {
                            Formula::Atom(predicate) => {
                                assert_eq!(predicate.name, "a");
                            }
                            token => panic!("{:?}", token)
                        }
                        match &*disjuncts[1] {
                            Formula::Atom(predicate) => {
                                assert_eq!(predicate.name, "b");
                            }
                            token => panic!("{:?}", token)
                        }
                    },
                    token => panic!("{:?}", token),
                }
            }
            token => panic!("{:?}", token),
        }
    }
}
