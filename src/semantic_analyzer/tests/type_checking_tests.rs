use std::collections::HashSet;

use super::*;

#[test]
pub fn untyped_parameter_accepts_any_argument_test() {
    let types = Some(vec![Symbol::new_untyped("block", TokenPosition::default())]);
    let checker = TypeChecker::new(&types);
    assert!(checker.is_var_type_consistent(Some("block"), None));
    assert!(!checker.is_var_type_consistent(None, Some("block")));
    assert!(checker.is_var_type_consistent(None, None));
    assert!(checker.is_var_type_consistent(Some("block"), Some("object")));
}

#[test]
pub fn basic_type_checking_test() {
    let program = String::from(
        "(define (domain bal)
            (:types
            t1 t2 - t3
            t4 t5 - t6
            t3 t6 - t7
            )
            (:predicates 
                (at ?l - t1)
            )
            (:action test1
            :parameters(?l1 - t2)
            :precondition (at ?l1)
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
                    SemanticErrorType::InconsistentPredicateArgType(t_err) => {
                        assert_eq!(t_err.var_name, "?l1");
                        assert_eq!(t_err.found.unwrap(), "t2");
                        assert_eq!(t_err.expected.unwrap(), "t1");
                        assert_eq!(t_err.position.line, 12);
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
pub fn effect_type_checking_test() {
    let program = String::from(
        "(define (domain bal)
            (:types
            t1 t2 - t3
            t4 t5 - t6
            t3 t6 - t7
            )
            (:predicates 
                (at ?l - t1)
            )
            (:action test1
            :parameters(?l1 ?l2 - t1)
            :precondition (at ?l2)
            :effect (not(at ?l1))
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
                Ok(_) => {}
                Err(error) => {
                    panic!("{:?}", error)
                }
            }
        }
        _ => panic!(),
    }
}

#[test]
pub fn inconsistent_predicate_arity_test() {
    let program = String::from(
        "(define (domain bal)
            (:types
            t1 t2 - t3
            t4 t5 - t6
            t3 t6 - t7
            )
            (:predicates 
                (at ?l - t1)
            )
            (:action test1
            :parameters(?l1 ?l2 - t1)
            :precondition (at ?l2)
            :effect (and 
                (not (at ?l1 ?l2) )
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
                Err(SemanticErrorType::InconsistentPredicateArity(x)) => {
                    assert_eq!(x.symbol, "at");
                    assert_eq!(x.expected_arity, 1);
                    assert_eq!(x.found_arity, 2);
                    assert_eq!(x.position.line, 14)
                }
                _ => {
                    panic!()
                }
            }
        }
        _ => panic!(),
    }
}

#[test]
pub fn inconsistent_subtask_arity_test() {
    let program = String::from(
        "(define (domain bal)
            (:types
            t1 t2 - t3
            t4 t5 - t6
            t3 t6 - t7
            )
            (:predicates 
                (at ?l - t7)
            )
            (:task abs :parameters(?a - t1))
            (:action test1
            :parameters(?l1 ?l2 - t5)
            :precondition (at ?l2)
            :effect (not(at ?l1))
            )
            (:method m1
                :parameters(?l2 - t1 ?l1 - t5)
                :task (abs ?l2)
                :tasks (and
                    (test1 ?l1 ?l1)
                    (test1 ?l1)
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
                Err(SemanticErrorType::InconsistentTaskArity(x)) => {
                    assert_eq!(x.symbol, "test1");
                    assert_eq!(x.expected_arity, 2);
                    assert_eq!(x.found_arity, 1);
                    assert_eq!(x.position.line, 21);
                }
                Err(token) => panic!("{:?}", token),
                Ok(_) => panic!("error not found"),
            }
        }
        _ => panic!(),
    }
}

#[test]
pub fn method_prec_type_checking_test() {
    let program = String::from(
        "(define (domain bal)
            (:types
            t1 t2 - t3
            t4 t5 - t6
            t3 t6 - t7
            )
            (:predicates 
                (at ?l - t7)
            )
            (:task abs :parameters(?a))
            (:action test1
            :parameters(?l1 ?l2 - t5)
            :precondition (at ?l2)
            :effect (not(at ?l1))
            )
            (:method m1
                :parameters(?l2 - t1 ?l1 - t5 ?l4)
                :task (abs ?l4)
                :precondition (at ?l2)
                :tasks (test1 ?l1 ?l1)
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
                Ok(_) => {}
                Err(error) => {
                    panic!("{:?}", error)
                }
            }
        }
        _ => panic!(),
    }
}

#[test]
pub fn method_subtask_checking_test() {
    let program = String::from(
        "(define (domain bal)
            (:types
            t1 t2 - t3
            t4 t5 - t6
            t3 t6 - t7
            )
            (:predicates 
                (at ?l - t7)
            )
            (:task abs :parameters(?a - t1))
            (:action test1
            :parameters(?l1 ?l2 - t3)
            :precondition (at ?l2)
            :effect (not(at ?l1))
            )
            (:action test2
            :parameters(?l1 ?l2 - t2)
            :precondition (at ?l2)
            :effect (not(at ?l1))
            )
            (:method m1
                :parameters(?l2 - t1 ?l1 - t2 ?l3 - t6)
                :task (abs ?l2)
                :precondition ()
                :tasks (and
                    (test1 ?l1 ?l1)
                    (test2 ?l3 ?l1)
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
                    panic!("error are not caught")
                }
                Err(error) => match error {
                    SemanticErrorType::InconsistentTaskArgType(t_error) => {
                        assert_eq!(t_error.expected.unwrap(), "t2");
                        assert_eq!(t_error.found.unwrap(), "t6");
                        assert_eq!(t_error.var_name, "?l3");
                        assert_eq!(t_error.position.line, 27);
                    }
                    any => {
                        panic!("{:?}", any)
                    }
                },
            }
        }
        _ => panic!(),
    }
}
#[test]
pub fn get_supertypes_test() {
    let types = Some(vec![
        Symbol::new("truck", TokenPosition::default(), Some("vehicle"), None),
        Symbol::new("loc", TokenPosition::default(), None, None),
    ]);
    let checker = TypeChecker::new(&types);
    assert_eq!(
        checker.get_types(),
        HashSet::from_iter(vec!["truck", "vehicle", "object", "loc"])
    );

    assert_eq!(
        checker.get_supertypes("truck"),
        HashSet::from_iter(vec!["vehicle", "object"])
    );
    assert_eq!(
        checker.get_supertypes("vehicle"),
        HashSet::from_iter(vec!["object"])
    );
    assert_eq!(
        checker.get_supertypes("loc"),
        HashSet::from_iter(vec!["object"])
    );
    assert_eq!(
        checker.get_supertypes("object"),
        HashSet::from_iter(Vec::<&str>::new())
    );
}
