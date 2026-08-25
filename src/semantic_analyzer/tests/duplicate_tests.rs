use super::*;

#[test]
pub fn objects_duplicate_test() {
    let domain = String::from(
        "(define (domain bal)
            (:types t1 t2)
            (:predicates 
                (at ?a )
                (hold ?a ?b)
            )
            (:action a_1
             :parameters (p_1 p_2  p_3)
             :precondition (not (at p_1))
             :effect (and (not (hold p_2 p_3)) (at p_2))
            )
            (:action a_2
             :parameters (p_1 p_2)
             :precondition (not (at p_1))
             :effect (and (not (at p_2)))
            )
            )
         ) ",
    )
    .into_bytes();
    let domain_lexer = LexicalAnalyzer::new(&domain);
    let domain_parser = Parser::new(domain_lexer);
    let domain_ast = domain_parser.parse().unwrap();
    match domain_ast {
        AbstractSyntaxTree::Domain(d) => {
            let semantic_parser = DomainSemanticAnalyzer::new(&d);
            let symbols = semantic_parser.verify_domain().unwrap();
            let problem = String::from(
                "(define (problem p1) (domain bal)
                    (:objects
                        a b c - t1
                        d - t1
                        t
                        a -t2
                        )
                    )",
            )
            .into_bytes();
            let problem_lexer = LexicalAnalyzer::new(&problem);
            let problem_parser = Parser::new(problem_lexer);
            let problem_ast = problem_parser.parse().unwrap();
            match problem_ast {
                AbstractSyntaxTree::Problem(p) => {
                    let semantic_parser = ProblemSemanticAnalyzer::new(&p, symbols);
                    match semantic_parser.verify_problem() {
                        Err(SemanticErrorType::DuplicateObjectDeclaration(x)) => {
                            assert_eq!(x.symbol, "a");
                            assert_eq!(x.first_pos.line, 3);
                            assert_eq!(x.second_pos.line, 6);
                        }
                        _ => panic!(),
                    }
                }
                _ => panic!("here"),
            }
        }
        _ => panic!(),
    }
}

#[test]
pub fn requirements_duplicate_test() {
    let program = String::from(
        "(define (domain bal)
            (:requirements :hierarchy :method-preconditions :hierarchy :negative-preconditions)

         ) ",
    )
    .into_bytes();
    let lexer = LexicalAnalyzer::new(&program);
    let parser = Parser::new(lexer);
    let ast = parser.parse().unwrap();
    match ast {
        AbstractSyntaxTree::Domain(d) => {
            let semantic_parser = DomainSemanticAnalyzer::new(&d);
            match semantic_parser.verify_domain() {
                Ok(_) => {
                    panic!("errors are not caught")
                }
                Err(error) => {
                    match error {
                        SemanticErrorType::DuplicateRequirementDeclaration(x) => {
                            assert!(matches!(x, RequirementType::Hierarchy))
                            // TODO: assert locality in future
                        }
                        _ => {
                            panic!("caught wrong error")
                        }
                    }
                }
            }
        }
        _ => panic!(),
    }
}

#[test]
pub fn predicates_duplicate_test() {
    let program = String::from(
        "(define (domain bal)
            (:types t_1 t_2)
            (:predicates
                (pred_1 ?a_1 ?a_2 - t_1 ?a_3 - t_2)
                (pred_2)
                (pred_1 a_1 a_2 a_3)
            )
         ) ",
    )
    .into_bytes();
    let lexer = LexicalAnalyzer::new(&program);
    let parser = Parser::new(lexer);
    let ast = parser.parse().unwrap();
    match ast {
        AbstractSyntaxTree::Domain(d) => {
            let semantic_parser = DomainSemanticAnalyzer::new(&d);
            match semantic_parser.verify_domain() {
                Ok(_) => {
                    panic!("errors are not caught")
                }
                Err(error) => match error {
                    SemanticErrorType::DuplicatePredicateDeclaration(x) => {
                        assert_eq!(x.symbol, "pred_1");
                        assert_eq!(x.first_pos.line, 4);
                        assert_eq!(x.second_pos.line, 6)
                    }
                    _ => {
                        panic!("caught wrong error")
                    }
                },
            }
        }
        _ => panic!(),
    }
}

#[test]
pub fn action_duplicate_test() {
    let program = String::from(
        "(define (domain bal)
            (:types t1 t2)
            (:predicates 
                (at ?a )
                (hold ?a ?b)
            )
            (:action a_1
             :parameters (p_1 p_2  p_3)
             :precondition (not (at p_1))
             :effect (and (not (hold p_2 p_3)) (at p_2))
            )
            (:action a_2
             :parameters (p_1 p_2)
             :precondition (not (at p_1))
             :effect (and (not (at p_2)))
            )
            (:action a_1
             :parameters (p_1 p_2 p_3 p4 p5)
             :precondition (not (at p_1))
             :effect (and (not (hold p_2 p_3)) (at p_2))
            )
         ) ",
    )
    .into_bytes();
    let lexer = LexicalAnalyzer::new(&program);
    let parser = Parser::new(lexer);
    let ast = parser.parse().unwrap();
    match ast {
        AbstractSyntaxTree::Domain(d) => {
            let semantic_parser = DomainSemanticAnalyzer::new(&d);
            match semantic_parser.verify_domain() {
                Ok(_) => {
                    panic!("errors are not caught")
                }
                Err(error) => match error {
                    SemanticErrorType::DuplicateActionDeclaration(x) => {
                        assert_eq!(x.symbol, "a_1");
                        assert_eq!(x.first_pos.line, 7);
                        assert_eq!(x.second_pos.line, 17);
                    }
                    token => {
                        panic!("{:?}", token)
                    }
                },
            }
        }
        _ => panic!(),
    }
}

#[test]
pub fn parameter_duplicate_test() {
    let program = String::from(
        "(define (domain bal)
            (:types t1 t2)
            (:predicates 
                (at ?a )
                (hold ?a ?b)
            )
            (:action a_1
             :parameters (?p_1 ?p_2 ?p_3 ?p_1)
             :precondition (not (at p_1))
             :effect (and (not (hold p_2 p_3)) (at p_2))
            )
         ) ",
    )
    .into_bytes();
    let lexer = LexicalAnalyzer::new(&program);
    let parser = Parser::new(lexer);
    let ast = parser.parse().unwrap();
    match ast {
        AbstractSyntaxTree::Domain(d) => {
            let semantic_parser = DomainSemanticAnalyzer::new(&d);
            match semantic_parser.verify_domain() {
                Ok(_) => {
                    panic!("errors are not caught")
                }
                Err(error) => match error {
                    SemanticErrorType::DuplicateParameterDeclaration(x) => {
                        assert_eq!(x.symbol, "?p_1");
                        assert_eq!(x.first_pos.line, 8);
                        assert_eq!(x.second_pos.line, 8);
                    }
                    token => {
                        panic!("{:?}", token)
                    }
                },
            }
        }
        _ => panic!(),
    }
}

#[test]
pub fn compound_task_duplicate_test() {
    let program = String::from(
        "(define (domain bal)
                (:task c_1
                 :parameters (p_1 p_2 p_3)
                )
                (:task c_2
                 :parameters (p_1)
                )
                (:task c_1
                 :parameters (p_1 p_2)
                )
             ) ",
    )
    .into_bytes();
    let lexer = LexicalAnalyzer::new(&program);
    let parser = Parser::new(lexer);
    let ast = parser.parse().unwrap();
    match ast {
        AbstractSyntaxTree::Domain(d) => {
            let semantic_parser = DomainSemanticAnalyzer::new(&d);
            match semantic_parser.verify_domain() {
                Ok(_) => {
                    panic!("errors are not caught")
                }
                Err(error) => match error {
                    SemanticErrorType::DuplicateCompoundTaskDeclaration(x) => {
                        assert_eq!(x.symbol, "c_1");
                        assert_eq!(x.first_pos.line, 2);
                        assert_eq!(x.second_pos.line, 8);
                    }
                    _ => {
                        panic!("caught wrong error")
                    }
                },
            }
        }
        _ => panic!(),
    }
}

#[test]
pub fn method_duplicate_test() {
    let program = String::from(
        "(define (domain bal)
                (:task deliver_abs :parameters (?a ?b ?c))
                (:action pickup
                    :parameters(?p1 ?l1)
                    :precondition ()
                )
                (:method m_1
                    :parameters (?p1 ?l1 ?l2 ?l3) 
                    :task (deliver_abs ?p1 ?l1 ?l2)
                    :subtasks (and
                        (pickup ?p1 ?l1)
                        (deliver_abs ?p1 ?l2 ?l3)
                    )
                )
                (:method m_2
                    :parameters (?p1 ?l1 ?l2) 
                    :task (deliver_abs ?p1 ?l1 ?l2)
                    :subtasks (and
                        (pickup ?p1 ?l1)
                    )
                )
                (:method m_1
                    :parameters (?p1 ?l1 ?l2 ?l3) 
                    :task (deliver_abs ?p1 ?l1 ?l2)
                    :subtasks (and
                        (deliver_abs ?p1 ?l2 ?l3)
                    )
                )
             ) ",
    )
    .into_bytes();
    let lexer = LexicalAnalyzer::new(&program);
    let parser = Parser::new(lexer);
    let ast = parser.parse().unwrap();
    match ast {
        AbstractSyntaxTree::Domain(d) => {
            let semantic_parser = DomainSemanticAnalyzer::new(&d);
            match semantic_parser.verify_domain() {
                Ok(_) => {
                    panic!("errors are not caught")
                }
                Err(error) => match error {
                    SemanticErrorType::DuplicateMethodDeclaration(x) => {
                        assert_eq!(x.symbol, "m_1");
                        assert_eq!(x.first_pos.line, 7);
                        assert_eq!(x.second_pos.line, 22);
                    }
                    _ => {
                        panic!("caught wrong error")
                    }
                },
            }
        }
        _ => panic!(),
    }
}
