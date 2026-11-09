use super::*;

#[cfg(test)]
mod tests {

    use std::{assert_eq, panic, vec};

    use super::*;

    fn atom(name: &str) -> Box<Precondition<'_>> {
        Box::new(Precondition::Atom(Predicate::new_dummy(name)))
    }

    fn flat(f: &impl std::fmt::Display) -> String {
        f.to_string()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .replace(" )", ")")
    }

    #[test]
    pub fn conjunction_of_literals_test() {
        let formula =
            Precondition::And(vec![atom("p_1"), Box::new(Precondition::Not(atom("p_2")))]);
        assert!(formula.is_simple_conjunction());
    }

    #[test]
    pub fn bare_literal_conjunction_test() {
        assert!(Precondition::Atom(Predicate::new_dummy("p_1")).is_simple_conjunction());
        assert!(Precondition::Not(atom("p_1")).is_simple_conjunction());
    }

    #[test]
    pub fn vacuous_conjunction_test() {
        assert!(Precondition::Empty.is_simple_conjunction());
        assert!(Precondition::And(vec![]).is_simple_conjunction());
    }

    #[test]
    pub fn disjunction_not_conjunction_test() {
        let disjunction = Precondition::Or(vec![atom("p_1"), atom("p_2")]);
        assert!(!disjunction.is_simple_conjunction());
        let conjoined_disjunction = Precondition::And(vec![
            Box::new(Precondition::Or(vec![atom("p_1"), atom("p_2")])),
            atom("p_3"),
        ]);
        assert!(!conjoined_disjunction.is_simple_conjunction());
        let xor = Precondition::Xor(vec![atom("p_1"), atom("p_2")]);
        assert!(!xor.is_simple_conjunction());
        let imply = Precondition::Imply(atom("p_1"), atom("p_2"));
        assert!(!imply.is_simple_conjunction());
    }

    #[test]
    pub fn nested_conjunction_not_simple_test() {
        let nested = Precondition::And(vec![
            Box::new(Precondition::And(vec![atom("p_1"), atom("p_2")])),
            atom("p_3"),
        ]);
        assert!(!nested.is_simple_conjunction());
        let double_negation = Precondition::Not(Box::new(Precondition::Not(atom("p_1"))));
        assert!(!double_negation.is_simple_conjunction());
    }

    #[test]
    pub fn quantified_not_conjunction_test() {
        let var = Symbol::new("x", TokenPosition::default(), None, None);
        let exists = Precondition::Exists(vec![var.clone()], atom("p_1"));
        assert!(!exists.is_simple_conjunction());
        let for_all = Precondition::ForAll(vec![var], atom("p_1"));
        assert!(!for_all.is_simple_conjunction());
    }

    #[test]
    pub fn parsed_precondition_conjunction_test() {
        let program = String::from(
            "(define (domain bal)
                (:action a_1
                 :parameters (p_1 p_2 - t1)
                 :precondition (and (at p_1) (not (at p_2)))
                 :effect (at p_2)
                )
                (:action a_2
                 :parameters (p_1 p_2 - t1)
                 :precondition (or (at p_1) (at p_2))
                 :effect (at p_2)
                )
             ) ",
        )
        .into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match Parser::new(lexer).parse() {
            Ok(AbstractSyntaxTree::Domain(ast)) => {
                assert_eq!(ast.actions.len(), 2);
                let a_1 = &ast.actions[0];
                assert!(a_1.preconditions.as_ref().unwrap().is_simple_conjunction());
                assert!(a_1.effects.as_ref().unwrap().is_simple_conjunction());
                let a_2 = &ast.actions[1];
                assert!(!a_2.preconditions.as_ref().unwrap().is_simple_conjunction());
            }
            _ => panic!("parsing errors"),
        }
    }

    #[test]
    fn nnf_then_dnf_test() {
        // ¬(a → (b ∧ ¬c))
        let f = Precondition::Not(Box::new(Precondition::Imply(
            atom("a"),
            Box::new(Precondition::And(vec![
                atom("b"),
                Box::new(Precondition::Not(atom("c"))),
            ])),
        )));
        let nnf = f.to_nnf(false);
        assert_eq!(flat(&nnf), "(and (a) (or (not (b)) (c)))");
        let dnf = f.to_dnf();
        assert_eq!(dnf.len(), 2);
        assert_eq!(flat(&dnf), "(or (and (a) (not (b))) (and (a) (c)))");
    }

    #[test]
    fn dnf_formula_cube_test() {
        let f = Precondition::Or(vec![
            Box::new(Precondition::And(vec![
                atom("a"),
                Box::new(Precondition::Not(atom("b"))),
            ])),
            atom("c"),
        ]);
        let dnf = f.to_dnf();
        assert_eq!(dnf.len(), 2);
        let first = &dnf.cubes[0];
        assert_eq!(first.len(), 2);
        assert!(first.values[0].is_positive());
        assert_eq!(first.values[0].predicate().name, "a");
        assert!(!first.values[1].is_positive());
        assert_eq!(first.values[1].predicate().name, "b");
        assert_eq!(flat(&first.to_formula()), "(and (a) (not (b)))");
        // a single-literal cube converts back to a bare literal, not (and ...)
        assert_eq!(flat(&dnf.cubes[1].to_formula()), "(c)");
        // iteration yields the cubes in order
        let sizes: Vec<usize> = dnf.into_iter().map(|cube| cube.len()).collect();
        assert_eq!(sizes, vec![2, 1]);
    }

    #[test]
    fn dnf_cubes_test() {
        // the cubes are exactly the disjuncts of to_dnf
        let f = Precondition::And(vec![
            atom("a"),
            Box::new(Precondition::Or(vec![atom("b"), atom("c")])),
        ]);
        let cubes = f.dnf_cubes();
        assert_eq!(cubes.len(), 2);
        assert_eq!(flat(&cubes[0][0]), "(a)");
        assert_eq!(flat(&cubes[0][1]), "(b)");
        assert_eq!(flat(&cubes[1][0]), "(a)");
        assert_eq!(flat(&cubes[1][1]), "(c)");
    }

    #[test]
    fn substitute_test() {
        let predicate1 = Box::new(Precondition::Atom(Predicate {
            name: "a",
            name_pos: TokenPosition::default(),
            variables: vec![
                Symbol::new_untyped("?x", TokenPosition::default()),
                Symbol::new_untyped("?y", TokenPosition::default()),
            ],
        }));
        let predicate2 = Box::new(Precondition::Atom(Predicate {
            name: "b",
            name_pos: TokenPosition::default(),
            variables: vec![
                Symbol::new_untyped("?z", TokenPosition::default()),
                Symbol::new_untyped("?x", TokenPosition::default()),
            ],
        }));
        let formula = Precondition::And(vec![predicate1, Box::new(Precondition::Not(predicate2))]);
        match formula.substitute(
            "?x",
            &Symbol::new_untyped("ground", TokenPosition::default()),
        ) {
            Precondition::And(inner) => {
                assert_eq!(2, inner.len());
                match &*inner[0] {
                    Precondition::Atom(pred) => {
                        assert_eq!("a", pred.name);
                        assert_eq!(2, pred.variables.len());
                        assert_eq!("ground", pred.variables[0].name);
                        assert!(pred.variables[0].symbol_type.is_none());
                        assert_eq!("?y", pred.variables[1].name);
                    }
                    _ => panic!(),
                }
                match &*inner[1] {
                    Precondition::Not(negated) => match &**negated {
                        Precondition::Atom(pred) => {
                            assert_eq!("b", pred.name);
                            assert_eq!(2, pred.variables.len());
                            assert_eq!("?z", pred.variables[0].name);
                            assert_eq!("ground", pred.variables[1].name);
                            assert!(pred.variables[1].symbol_type.is_none());
                        }
                        _ => panic!(),
                    },
                    _ => panic!(),
                }
            }
            _ => panic!(),
        }
    }
}
