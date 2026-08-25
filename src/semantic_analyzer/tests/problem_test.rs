use super::*;

fn get_domain() -> Vec<u8> {
    "
    (define (domain Depot)
            (:requirements :negative-preconditions
                :hierarchy
                :typing
                :method-preconditions
            )

            (:types place locatable - object depot distributor - place truck hoist surface - locatable pallet crate - surface)

            (:predicates (at ?x - locatable ?y - place) (on ?x - crate ?y - surface) (in ?x - crate ?y - truck) (lifting ?x - hoist ?y - crate) (available ?x - hoist) (clear ?x - surface))

            (:task do_put_on
            :parameters (?c - crate ?s2 - surface) )

            (:task do_clear
            :parameters (?s1 - surface ?p1 - place) )

            (:task do_get_truck
            :parameters (?t - truck ?p1 - place) )

            (:task do_lift_crate
            :parameters (?c - crate ?p - place ?h - hoist) )

            (:task do_load_truck
            :parameters (?c - crate ?s - surface ?p - place ?t - truck) )

            (:task do_unload_truck
            :parameters (?c - crate ?s - surface ?p - place ?t - truck) )

            (:method m0_do_put_on
            :parameters ( ?c - crate  ?s2 - surface )
            :task (do_put_on ?c ?s2)
            :precondition(and (on ?c ?s2))
            :ordered-subtasks(and (t1 (nop))) ) 

            (:method m1_do_put_on
            :parameters ( ?c - crate  ?s2 - surface ?p - place ?h - hoist )
            :task (do_put_on ?c ?s2)
            :precondition(and (at ?c ?p))
            :ordered-subtasks(and (t1 (do_clear ?c ?p)) (t2 (do_clear ?s2 ?p)) (t3 (do_lift_crate ?c ?p ?h)) (t4 (Drop ?h ?c ?s2 ?p))) ) 

            (:method m2_do_put_on
            :parameters ( ?c - crate  ?s2 - surface ?p - place ?t - truck ?h - hoist )
            :task (do_put_on ?c ?s2)
            :precondition(and (in ?c ?t))
            :ordered-subtasks(and (t1 (do_get_truck ?t ?p)) (t2 (do_clear ?s2 ?p)) (t3 (Unload ?h ?c ?t ?p)) (t4 (Drop ?h ?c ?s2 ?p))) ) 

            (:method m3_do_put_on
            :parameters ( ?c - crate  ?s2 - surface ?s1 - surface ?p1 - place ?t - truck ?p2 - place )
            :task (do_put_on ?c ?s2)
            :precondition()
            :ordered-subtasks(and (t1 (do_load_truck ?c ?s1 ?p1 ?t)) (t2 (Drive ?t ?p1 ?p2)) (t3 (do_unload_truck ?c ?s2 ?p2 ?t))) ) 

            (:method m4_do_clear
            :parameters ( ?s1 - surface  ?p1 - place )
            :task (do_clear ?s1 ?p1)
            :precondition(and (clear ?s1) (at ?s1 ?p1))
            :ordered-subtasks(and (t1 (nop))) ) 

            (:method m5_do_clear
            :parameters ( ?s1 - surface  ?p1 - place ?c - crate ?t - truck ?h1 - hoist )
            :task (do_clear ?s1 ?p1)
            :precondition(and (not (clear ?s1)) (on ?c ?s1) (at ?s1 ?p1) (at ?h1 ?p1))
            :ordered-subtasks(and (t1 (do_clear ?c ?p1)) (t2 (Lift ?h1 ?c ?s1 ?p1)) (t3 (do_get_truck ?t ?p1)) (t4 (Load ?h1 ?c ?t ?p1))) ) 

            (:method m6_do_get_truck
            :parameters ( ?t - truck  ?p1 - place )
            :task (do_get_truck ?t ?p1)
            :precondition(and (at ?t ?p1))
            :ordered-subtasks(and (t1 (nop))) ) 

            (:method m7_do_get_truck
            :parameters ( ?t - truck  ?p1 - place ?p2 - place )
            :task (do_get_truck ?t ?p1)
            :precondition(and (not (at ?t ?p1)))
            :ordered-subtasks(and (t1 (Drive ?t ?p2 ?p1))) ) 

            (:method m8_do_lift_crate
            :parameters ( ?c - crate  ?p - place  ?h - hoist ?t - truck )
            :task (do_lift_crate ?c ?p ?h)
            :precondition(and (in ?c ?t) (at ?h ?p))
            :ordered-subtasks(and (t1 (do_get_truck ?t ?p)) (t2 (Unload ?h ?c ?t ?p))) ) 

            (:method m9_do_lift_crate
            :parameters ( ?c - crate  ?p - place  ?h - hoist ?s - surface )
            :task (do_lift_crate ?c ?p ?h)
            :precondition(and (on ?c ?s) (at ?c ?p) (at ?s ?p) (at ?h ?p))
            :ordered-subtasks(and (t1 (Lift ?h ?c ?s ?p))) ) 

            (:method m10_do_load_trcaruck
            :parameters ( ?c - crate  ?s - surface  ?p - place  ?t - truck ?h - hoist )
            :task (do_load_truck ?c ?s ?p ?t)
            :precondition(and (at ?c ?p) (at ?s ?p) (on ?c ?s) (at ?h ?p))
            :ordered-subtasks(and (t1 (do_get_truck ?t ?p)) (t2 (do_clear ?c ?p)) (t3 (Lift ?h ?c ?s ?p)) (t4 (Load ?h ?c ?t ?p))) ) 

            (:method m11_do_unload_truck
            :parameters ( ?c - crate  ?s - surface  ?p - place  ?t - truck ?h - hoist )
            :task (do_unload_truck ?c ?s ?p ?t)
            :precondition(and (in ?c ?t) (at ?t ?p) (at ?h ?p) (at ?s ?p))
            :ordered-subtasks(and (t1 (do_clear ?s ?p)) (t2 (Unload ?h ?c ?t ?p)) (t3 (Drop ?h ?c ?s ?p))) ) 

            (:action Drive
            :parameters (?x - truck ?y - place ?z - place)
            :precondition (and (at ?x ?y))
            :effect (and (not (at ?x ?y)) (at ?x ?z)))

            (:action Lift
            :parameters (?x - hoist ?y - crate ?z - surface ?p - place)
            :precondition (and (at ?x ?p) (available ?x) (at ?y ?p) (on ?y ?z) (clear ?y))
            :effect (and (not (at ?y ?p)) (lifting ?x ?y) (not (clear ?y)) (not (available ?x)) (clear ?z) (not (on ?y ?z))))

            (:action Drop
            :parameters (?x - hoist ?y - crate ?z - surface ?p - place)
            :precondition (and (at ?x ?p) (at ?z ?p) (clear ?z) (lifting ?x ?y))
            :effect (and (available ?x) (not (lifting ?x ?y)) (at ?y ?p) (not (clear ?z)) (clear ?y) (on ?y ?z)))

            (:action Load
            :parameters (?x - hoist ?y - crate ?z - truck ?p - place)
            :precondition (and (at ?x ?p) (at ?z ?p) (lifting ?x ?y))
            :effect (and (not (lifting ?x ?y)) (in ?y ?z) (available ?x)))

            (:action Unload
            :parameters (?x - hoist ?y - crate ?z - truck ?p - place)
            :precondition (and (at ?x ?p) (at ?z ?p) (available ?x) (in ?y ?z))
            :effect (and (not (in ?y ?z)) (not (available ?x)) (lifting ?x ?y)))

            (:action nop
            :parameters ()
            :precondition ()
            :effect ())
))".as_bytes().to_vec()
}

#[test]
pub fn p_undeclared_type_test() {
    let program = get_domain();
    let problem = String::from(
        "(define (problem p1)
            (:domain d)
            (:objects
                x1 x2 - place
                truck1 truck2 -locatable
                ve1 - car 
            )
            (:htn
                :parameters ()
                :ordered-subtasks (and
                    (abs_1)
                )
            )
    ",
    )
    .into_bytes();
    let lexer = LexicalAnalyzer::new(&program);
    let parser = Parser::new(lexer);
    let d_ast = parser.parse().unwrap();
    match d_ast {
        AbstractSyntaxTree::Domain(d) => {
            let p_lexer = LexicalAnalyzer::new(&problem);
            let p_parser = Parser::new(p_lexer);
            let p_ast = p_parser.parse().unwrap();
            match p_ast {
                AbstractSyntaxTree::Problem(p) => {
                    let d_analyzer = DomainSemanticAnalyzer::new(&d);
                    let domain_symbols = d_analyzer.verify_domain().unwrap();
                    let p_analyzer = ProblemSemanticAnalyzer::new(&p, domain_symbols);
                    match p_analyzer.verify_problem() {
                        Ok(_) => {
                            panic!("error not found")
                        }
                        Err(d) => match d {
                            SemanticErrorType::UndefinedType(ty) => {
                                assert_eq!(ty.symbol, "car");
                                assert_eq!(ty.position.line, 6);
                            }
                            _ => panic!(),
                        },
                    }
                }
                _ => panic!(),
            }
        }
        AbstractSyntaxTree::Problem(_) => panic!(),
    }
}

#[test]
pub fn p_undefined_object_test() {
    let program = get_domain();
    let problem = String::from(
        "(define (problem p1)
            (:domain d)
            (:objects
                x1 x2 - place
                truck1 truck2 -truck
            )
            (:htn
                :parameters ()
                :ordered-subtasks (and
                    (do_get_truck truck1 x3)
                )
            )
    ",
    )
    .into_bytes();
    let lexer = LexicalAnalyzer::new(&program);
    let parser = Parser::new(lexer);
    let d_ast = parser.parse().unwrap();
    match d_ast {
        AbstractSyntaxTree::Domain(d) => {
            let p_lexer = LexicalAnalyzer::new(&problem);
            let p_parser = Parser::new(p_lexer);
            let p_ast = p_parser.parse().unwrap();
            match p_ast {
                AbstractSyntaxTree::Problem(p) => {
                    let d_analyzer = DomainSemanticAnalyzer::new(&d);
                    let domain_symbols = d_analyzer.verify_domain().unwrap();
                    let p_analyzer = ProblemSemanticAnalyzer::new(&p, domain_symbols);
                    match p_analyzer.verify_problem() {
                        Ok(_) => {
                            panic!("error not found")
                        }
                        Err(d) => match d {
                            SemanticErrorType::UndefinedObject(undefined) => {
                                assert_eq!(undefined.symbol, "x3");
                                assert_eq!(undefined.position.line, 10);
                            }
                            token => panic!("{:?}", token),
                        },
                    }
                }
                _ => panic!(),
            }
        }
        AbstractSyntaxTree::Problem(_) => panic!(),
    }
}

#[test]
pub fn p_duplicate_object_definition_test() {
    let program = get_domain();
    let problem = String::from(
        "
        (define (problem p1)
            (:domain d)
            (:objects
                x1 x2 - place
                truck1 truck2 -locatable
                x1 - object
            )
            (:htn
                :parameters ()
                :ordered-subtasks (and
                    (abs_1)
                )
            )
    ",
    )
    .into_bytes();
    let lexer = LexicalAnalyzer::new(&program);
    let parser = Parser::new(lexer);
    let d_ast = parser.parse().unwrap();
    match d_ast {
        AbstractSyntaxTree::Domain(d) => {
            let p_lexer = LexicalAnalyzer::new(&problem);
            let p_parser = Parser::new(p_lexer);
            let p_ast = p_parser.parse().unwrap();
            match p_ast {
                AbstractSyntaxTree::Problem(p) => {
                    let d_analyzer = DomainSemanticAnalyzer::new(&d);
                    let domain_symbols = d_analyzer.verify_domain().unwrap();
                    let p_analyzer = ProblemSemanticAnalyzer::new(&p, domain_symbols);
                    match p_analyzer.verify_problem() {
                        Ok(_) => {
                            panic!("error not found")
                        }
                        Err(d) => match d {
                            SemanticErrorType::DuplicateObjectDeclaration(ty) => {
                                if ty.symbol != "x1" {
                                    panic!("wrong error")
                                }
                            }
                            _ => panic!(),
                        },
                    }
                }
                _ => panic!(),
            }
        }
        AbstractSyntaxTree::Problem(_) => panic!(),
    }
}

#[test]
pub fn p_cyclic_init_tn_ordering_test() {
    let program = get_domain();
    let problem = String::from(
        "(define (problem p1)
            (:domain d)
            (:objects
                x1 x2 - place
                truck1 truck2 -locatable
                crate1 crate2 - crate
            )
            (:htn
                :subtasks (and
                    (t1 (do_put_on crate1 truck1))
                    (t2 (do_put_on crate2 truck1))
                    (t3 (do_put_on crate2 truck1))
                )
                :ordering (and
                    (< t1 t2)
                    (< t2 t3)
                    (< t3 t1)
                )
            )
    ",
    )
    .into_bytes();
    let lexer = LexicalAnalyzer::new(&program);
    let parser = Parser::new(lexer);
    let d_ast = parser.parse().unwrap();
    match d_ast {
        AbstractSyntaxTree::Domain(d) => {
            let p_lexer = LexicalAnalyzer::new(&problem);
            let p_parser = Parser::new(p_lexer);
            let p_ast = p_parser.parse().unwrap();
            match p_ast {
                AbstractSyntaxTree::Problem(p) => {
                    let d_analyzer = DomainSemanticAnalyzer::new(&d);
                    let domain_symbols = d_analyzer.verify_domain().unwrap();
                    let p_analyzer = ProblemSemanticAnalyzer::new(&p, domain_symbols);
                    match p_analyzer.verify_problem() {
                        Ok(_) => {
                            panic!("error not found")
                        }
                        Err(d) => match d {
                            SemanticErrorType::CyclicOrderingDeclaration(pos) => {
                                assert_eq!(pos.line, 14);
                            }
                            _ => panic!(),
                        },
                    }
                }
                _ => panic!(),
            }
        }
        AbstractSyntaxTree::Problem(_) => panic!(),
    }
}

#[test]
pub fn p_inconsistent_goal_predicate_test() {
    let program = get_domain();
    let problem = String::from(
        "
        (define (problem p1)
            (:domain d)
            (:objects
                x1 x2 - place
                truck1 truck2 -locatable
            )
            (:htn
                :parameters ()
                :ordered-subtasks ()
            )
            (:goal
                (and
                    (at truck1 x1)
                    (at truck1)
                )
            )
    ",
    )
    .into_bytes();
    let lexer = LexicalAnalyzer::new(&program);
    let parser = Parser::new(lexer);
    let d_ast = parser.parse().unwrap();
    match d_ast {
        AbstractSyntaxTree::Domain(d) => {
            let p_lexer = LexicalAnalyzer::new(&problem);
            let p_parser = Parser::new(p_lexer);
            let p_ast = p_parser.parse().unwrap();
            match p_ast {
                AbstractSyntaxTree::Problem(p) => {
                    let d_analyzer = DomainSemanticAnalyzer::new(&d);
                    let domain_symbols = d_analyzer.verify_domain().unwrap();
                    let p_analyzer = ProblemSemanticAnalyzer::new(&p, domain_symbols);
                    match p_analyzer.verify_problem() {
                        Ok(_) => {
                            panic!("error not found")
                        }
                        Err(d) => match d {
                            SemanticErrorType::UndefinedPredicate(x) => {
                                assert_eq!(x.symbol, "at");
                            }
                            _ => panic!(),
                        },
                    }
                }
                _ => panic!(),
            }
        }
        AbstractSyntaxTree::Problem(_) => panic!(),
    }
}

#[test]
pub fn p_inconsistent_init_predicate_test() {
    let program = get_domain();
    let problem = String::from(
        "
        (define (problem p1)
            (:domain d)
            (:objects
                x1 x2 - place
                truck1 truck2 -locatable
                crate1 - crate
            )
            (:htn
                :parameters ()
                :ordered-subtasks ()
            )
            (:init
                (at truck1 crate1)
            )
    ",
    )
    .into_bytes();
    let lexer = LexicalAnalyzer::new(&program);
    let parser = Parser::new(lexer);
    let d_ast = parser.parse().unwrap();
    match d_ast {
        AbstractSyntaxTree::Domain(d) => {
            let p_lexer = LexicalAnalyzer::new(&problem);
            let p_parser = Parser::new(p_lexer);
            let p_ast = p_parser.parse().unwrap();
            match p_ast {
                AbstractSyntaxTree::Problem(p) => {
                    let d_analyzer = DomainSemanticAnalyzer::new(&d);
                    let domain_symbols = d_analyzer.verify_domain().unwrap();
                    let p_analyzer = ProblemSemanticAnalyzer::new(&p, domain_symbols);
                    match p_analyzer.verify_problem() {
                        Ok(_) => {
                            panic!("error not found")
                        }
                        Err(d) => match d {
                            SemanticErrorType::InconsistentPredicateArgType(ty) => {
                                if ty.var_name != "at" {
                                    panic!("wrong error")
                                }
                            }
                            token => panic!("{:?}", token),
                        },
                    }
                }
                _ => panic!(),
            }
        }
        AbstractSyntaxTree::Problem(_) => panic!(),
    }
}

#[test]
pub fn p_inconsistent_subtask_test() {
    let program = get_domain();
    let problem = String::from(
        "
        (define (problem p1)
            (:domain d)
            (:objects
                x1 x2 - place
                truck1 truck2 -locatable
                crate1 crate2 - crate
            )
            (:htn
                :parameters ()
                :subtasks (and
                    (t1 (do_get_truck truck1 truck2))
                )
            )
    ",
    )
    .into_bytes();
    let lexer = LexicalAnalyzer::new(&program);
    let parser = Parser::new(lexer);
    let d_ast = parser.parse().unwrap();
    match d_ast {
        AbstractSyntaxTree::Domain(d) => {
            let p_lexer = LexicalAnalyzer::new(&problem);
            let p_parser = Parser::new(p_lexer);
            let p_ast = p_parser.parse().unwrap();
            match p_ast {
                AbstractSyntaxTree::Problem(p) => {
                    let d_analyzer = DomainSemanticAnalyzer::new(&d);
                    let domain_symbols = d_analyzer.verify_domain().unwrap();
                    let p_analyzer = ProblemSemanticAnalyzer::new(&p, domain_symbols);
                    match p_analyzer.verify_problem() {
                        Ok(_) => {
                            panic!("error not found")
                        }
                        Err(d) => match d {
                            SemanticErrorType::InconsistentTaskArgType(ty) => {
                                if ty.var_name != "do_get_truck" {
                                    panic!("wrong error")
                                }
                            }
                            token => panic!("{:?}", token),
                        },
                    }
                }
                _ => panic!(),
            }
        }
        AbstractSyntaxTree::Problem(_) => panic!(),
    }
}
