use super::*;

#[test]
pub fn cyclic_method_ordering_test() {
    let program = String::from(
        "(define (domain bal)
                (:predicates 
                    (hold ?a_1 ?a_2)
                    (pred_2)
                    (at ?a_1)
                )
                (:task deliver_abs_1 :parameters(?p1))
                (:task deliver_abs_2 :parameters(?p1))
                (:task deliver_abs_3 :parameters(?p1))
                (:task deliver_abs_4 :parameters(?p1))

                (:method m_1
                    :parameters (?p1 ?p2 ?p3 ?p4) 
                    :task (deliver_abs_1 ?p1)
                    :subtasks (and
                        (t1 (deliver_abs_1 ?p1))
                        (t2 (deliver_abs_2 ?p2))
                        (t3 (deliver_abs_3 ?p3))
                        (t4 (deliver_abs_4 ?p4))
                    )
                    :ordering (and
                        (< t1 t2)
                        (< t2 t3)
                        (< t3 t4)
                        (< t4 t1)
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
                    SemanticErrorType::CyclicOrderingDeclaration(pos) => {
                        assert_eq!(pos.line, 21);
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
pub fn cyclic_types_test() {
    let program = String::from(
        "(define (domain bal)
            (:types
            t1 t2 - t3
            t4 t5 - t6
            t3 t6 - t7
            t7 - t1
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
                        SemanticErrorType::CyclicTypeDeclaration => {
                            // TODO: assert locality in future
                        }
                        _ => {
                            panic!("caught wrong error")
                        }
                    }
                }
            }
        }
        _ => {}
    }
}
