use super::*;

#[test]
pub fn undefined_predicate_action_precondition_test() {
    let program = String::from(
        "(define (domain bal)
            (:predicates 
                (hold ?a_1 ?a_2)
                (pred_2)
                (at a_1)
            )
            (:action a_1
             :parameters (p_1 p_2 p_3)
             :precondition (and (not (at p_1)) (pred_5))
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
                    SemanticErrorType::UndefinedPredicate(x) => {
                        assert_eq!(x.symbol, "pred_5");
                        assert_eq!(x.position.line, 9);
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
pub fn inconsistent_predicate_action_effect_test() {
    let program = String::from(
        "(define (domain bal)
            (:predicates 
                (hold ?a_1 ?a_2)
                (pred_2)
                (at ?a_1)
            )
            (:action a_1
             :parameters (?p_1 ?p_2 ?p_3)
             :precondition (and (not (at ?p_1)) (hold ?p_1 ?p_2))
             :effect (and (not (hold ?p_2 ?p_3 p_2)) (at ?p_2))
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
                Err(error) => {
                    match error {
                        SemanticErrorType::InconsistentPredicateArity(ar_error) => {
                            assert_eq!(ar_error.symbol, "hold");
                            assert_eq!(ar_error.expected_arity, 2);
                            assert_eq!(ar_error.found_arity, 3)
                            // TODO: assert locality in future
                        }
                        token => {
                            panic!("{:?}", token)
                        }
                    }
                }
            }
        }
        _ => panic!(),
    }
}

#[test]
pub fn undefined_predicate_method_precondition_test() {
    let program = String::from(
        "(define (domain bal)
                (:predicates 
                    (hold ?a_1 ?a_2)
                    (pred_2)
                    (at ?a_1)
                )
                (:method m_1
                    :parameters (?p1 ?l1 ?l2 ?l3) 
                    :task (deliver_abs ?p1 ?l1 ?l2)
                    :precondition (oneof (and (not (hold ?p_2 ?p_3)) (at ?p_2)) (pred_5))
                    :subtasks (and
                        (pickup ?p1 ?l1)
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
                    SemanticErrorType::UndefinedPredicate(x) => {
                        assert_eq!(x.symbol, "pred_5");
                        assert_eq!(x.position.line, 10);
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
pub fn undefined_method_parameters_test() {
    let program = String::from(
        "(define (domain bal)
                (:predicates 
                    (hold ?a_1 ?a_2)
                    (pred_2)
                    (at ?a_1)
                )
                (:task deliver_abs :parameters (?p1 ?l1 ?l2))
                (:action pickup
                    :parameters(?p1 ?l1)
                    :precondition(hold ?p1 ?l1)
                )
                (:method m_1
                    :parameters (?p1 ?p2 ?p3 ?l1 ?l2 ?l3) 
                    :task (deliver_abs ?p1 ?l1 ?l2)
                    :precondition (oneof (and (not (hold ?p2 ?p3)) (at ?p5)))
                    :subtasks (and
                        (pickup ?p1 ?l1)
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
                    SemanticErrorType::UndefinedParameter(x) => {
                        assert_eq!(x.symbol, "?p5");
                        assert_eq!(x.position.line, 15);
                    }
                    x => {
                        panic!("{:?}", x)
                    }
                },
            }
        }
        _ => panic!(),
    }
}

#[test]
pub fn undefined_subtask_test() {
    let program = String::from(
        "(define (domain bal)
                (:predicates 
                    (hold ?a_1 ?a_2)
                    (pred_2)
                    (at ?a_1)
                )
                (:task c_1
                    :parameters (?p_1 ?p_2 ?p_3)
                )
                (:task c_2
                    :parameters (?p_1)
                )
                (:method m_1
                    :parameters (?p1 ?p2 ?p3 ?l1 ?l2 ?l3) 
                    :task (c_2 ?p1)
                    :precondition (oneof (and (not (hold ?p2 ?p3)) (at ?p2)) (pred_2))
                    :subtasks (and
                        (c_1 ?p1 ?l1 ?l2)
                        (c_2 ?p1)
                        (c_3)
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
                    SemanticErrorType::UndefinedSubtask(x) => {
                        assert_eq!(x.symbol, "c_3");
                        assert_eq!(x.position.line, 20);
                    }
                    error => {
                        panic!("{:?}", error)
                    }
                },
            }
        }
        _ => panic!(),
    }
}

#[test]
pub fn inconsistent_subtask_arity_test() {
    let program = String::from(
        "(define (domain bal)
                (:predicates 
                    (hold ?a_1 ?a_2)
                    (pred_2)
                    (at ?a_1)
                )
                (:task c_1
                    :parameters (?p_1 ?p_2 ?p_3)
                )
                (:task c_2
                    :parameters (?p_1)
                )
                (:method m_1
                    :parameters (?p1 ?p2 ?p3 ?l1 ?l2 ?l3) 
                    :task (c_1 ?p1 ?l1 ?l2)
                    :precondition (oneof (and (not (hold ?p2 ?p3)) (at ?p2)) (pred_2))
                    :subtasks (and
                        (c_1 ?p1 ?l1 ?l2)
                        (c_2 ?p1 ?l3)
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
                Err(error) => {
                    match error {
                        SemanticErrorType::InconsistentTaskArity(ar_error) => {
                            assert_eq!(ar_error.symbol, "c_2");
                            assert_eq!(ar_error.expected_arity, 1);
                            assert_eq!(ar_error.found_arity, 2);
                            // TODO: assert locality in future
                        }
                        error => {
                            panic!("{:?}", error)
                        }
                    }
                }
            }
        }
        _ => panic!(),
    }
}

#[test]
pub fn undefined_type_compound_task_test() {
    let program = String::from(
        "(define (domain bal)
                (:types t1)
                (:predicates 
                    (hold ?a_1 ?a_2)
                    (pred_2)
                    (at ?a_1)
                )
                (:task c_1
                    :parameters (?p_1 ?p_2 ?p_3 - t1 ?p4 - t5)
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
                    SemanticErrorType::UndefinedType(x) => {
                        assert_eq!(x.symbol, "t5");
                        assert_eq!(x.position.line, 9);
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
pub fn undefined_type_predicate_test() {
    let program = String::from(
        "(define (domain bal)
                (:types t1)
                (:predicates 
                    (pred_2)
                    (at ?a_1)
                    (hold ?a_1 ?a_2 - t1 ?a_3 - t2)
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
                    SemanticErrorType::UndefinedType(x) => {
                        assert_eq!(x.symbol, "t2");
                        assert_eq!(x.position.line, 6);
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
pub fn undefined_predicate_forall_quantification_test() {
    let program = String::from(
        "(define (domain bal)
                (:predicates 
                    (hold ?a_1 ?a_2)
                    (pred_2)
                    (at ?a_1)
                )
                (:task c_1
                    :parameters (?p_1 ?p_2 ?p_3)
                )
                (:task c_2
                    :parameters ()
                )
                (:method m_1
                    :parameters () 
                    :task (c_2)
                    :precondition (forall (?pos - location) (and (not (at ?pos)) (wro ?pos)))
                    :subtasks (and
                        (c_1 ?p1 ?l1 ?l2)
                        (c_2)
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
                    SemanticErrorType::UndefinedPredicate(x) => {
                        assert_eq!(x.symbol, "wro");
                        assert_eq!(x.position.line, 16)
                    }
                    error => {
                        panic!("{:?}", error)
                    }
                },
            }
        }
        _ => panic!(),
    }
}

#[test]
pub fn undefined_method_task_test() {
    let program = String::from(
        "(define (domain bal)
                (:predicates 
                    (hold ?a_1 ?a_2)
                    (pred_2)
                    (at ?a_1)
                )
                (:task c_1
                    :parameters (?p_1 ?p_2 ?p_3)
                )
                (:method m_1
                    :parameters (?p1 ?l1 ?l2 ?l3) 
                    :task (deliver_abs ?p1 ?l1 ?l2)
                    :subtasks (and
                        (c_1 ?p1 ?l1 ?l2)
                        (c_1 ?p1 ?l2 ?l3)
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
                    SemanticErrorType::UndefinedTask(x) => {
                        assert_eq!(x.symbol, "deliver_abs");
                        assert_eq!(x.position.line, 12)
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
