use super::*;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    #[ignore = "stupid test, rewrite from scratch"]
    pub fn file_type_test() {
        let program = String::from("(define (domain jajaja) (:predicates ()) ) ").into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        let parser = Parser::new(lexer);
        match parser.parse() {
            Ok(_) => {}
            _ => panic!("parsing error"),
        }
        let program = String::from("(define (problem jajaja2) (domain blahblah)) ").into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        let parser = Parser::new(lexer);
        match parser.parse() {
            Ok(_) => {}
            _ => panic!("parsing error"),
        }
    }

    #[test]
    pub fn objects_list_test() {
        let program = String::from(
            "(define (problem p1) (domain bal)
                            (:objects a
                            b c 
                            - d
                            s - f t)
                          )",
        )
        .into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match Parser::new(lexer).parse() {
            Ok(AbstractSyntaxTree::Problem(symbols)) => {
                assert_eq!(symbols.objects[0].name, "a");
                assert_eq!(symbols.objects[0].name_pos.line, 2);
                assert_eq!(symbols.objects[0].symbol_type.unwrap(), "d");
                assert_eq!(symbols.objects[0].type_pos.unwrap().line, 4);
                assert_eq!(symbols.objects[1].name, "b");
                assert_eq!(symbols.objects[1].name_pos.line, 3);
                assert_eq!(symbols.objects[1].symbol_type.unwrap(), "d");
                assert_eq!(symbols.objects[1].type_pos.unwrap().line, 4);
                assert_eq!(symbols.objects[2].name, "c");
                assert_eq!(symbols.objects[2].name_pos.line, 3);
                assert_eq!(symbols.objects[2].symbol_type.unwrap(), "d");
                assert_eq!(symbols.objects[2].type_pos.unwrap().line, 4);
                assert_eq!(symbols.objects[3].name, "s");
                assert_eq!(symbols.objects[3].name_pos.line, 5);
                assert_eq!(symbols.objects[3].symbol_type.unwrap(), "f");
                assert_eq!(symbols.objects[3].type_pos.unwrap().line, 5);
                assert_eq!(symbols.objects[4].name, "t");
                assert_eq!(symbols.objects[4].name_pos.line, 5);
                assert_eq!(symbols.objects[4].symbol_type.is_none(), true);
                assert_eq!(symbols.objects[4].type_pos.is_none(), true);
            }
            _ => panic!("parsing errors"),
        }
    }

    #[test]
    pub fn untyped_objects_list_test() {
        let program = String::from(
            "(define
            (problem p1) (domain bal)
            (:objects a b
            c) )",
        )
        .into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match Parser::new(lexer).parse() {
            Ok(AbstractSyntaxTree::Problem(symbols)) => {
                assert_eq!(symbols.objects[0].name, "a");
                assert_eq!(symbols.objects[0].name_pos.line, 3);
                assert_eq!(symbols.objects[1].name, "b");
                assert_eq!(symbols.objects[1].name_pos.line, 3);
                assert_eq!(symbols.objects[2].name, "c");
                assert_eq!(symbols.objects[2].name_pos.line, 4);
            }
            _ => panic!("parsing errors"),
        }
    }

    #[test]
    pub fn requirement_parsing_test() {
        let program = String::from(
            "(define (problem p1) (domain bal)
                (:requirements :hierarchy :method-preconditions :typing :negative-preconditions)

             ) ",
        )
        .into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match Parser::new(lexer).parse() {
            Ok(AbstractSyntaxTree::Problem(symbols)) => {
                assert_eq!(symbols.requirements.len(), 4);
                assert_eq!(
                    symbols.requirements.contains(&RequirementType::Hierarchy),
                    true
                );
                assert_eq!(
                    symbols
                        .requirements
                        .contains(&RequirementType::MethodPreconditions),
                    true
                );
                assert_eq!(
                    symbols
                        .requirements
                        .contains(&RequirementType::NegativePreconditions),
                    true
                );
                assert_eq!(
                    symbols
                        .requirements
                        .contains(&RequirementType::TypedObjects),
                    true
                );
            }
            _ => panic!("parsing errors"),
        }
    }

    #[test]
    pub fn predicate_parsing_test() {
        let program = String::from(
            "(define (domain bal)
                (:predicates 
                    (pred_1 ?a_1 ?a_2 - t_1 ?a_3 - t_2)
                    (pred_2)
                    (pred_3 a_1 a_2)
                )
             ) ",
        )
        .into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match Parser::new(lexer).parse() {
            Ok(AbstractSyntaxTree::Domain(symbols)) => {
                assert_eq!(symbols.predicates.len(), 3);
                for predicate in symbols.predicates {
                    let items: Vec<(&str, Option<&str>)> = predicate
                        .variables
                        .iter()
                        .map(|x| (x.name, x.symbol_type))
                        .collect();
                    if predicate.name == "pred_1" {
                        assert_eq!(
                            items,
                            vec![
                                ("a_1", Some("t_1")),
                                ("a_2", Some("t_1")),
                                ("a_3", Some("t_2"))
                            ]
                        );
                        assert_eq!(predicate.name_pos.line, 3);
                    } else if predicate.name == "pred_2" {
                        assert_eq!(predicate.variables.len(), 0);
                        assert_eq!(predicate.name_pos.line, 4);
                    } else if predicate.name == "pred_3" {
                        assert_eq!(predicate.name_pos.line, 5);
                        let items: Vec<(&str, Option<&str>)> = predicate
                            .variables
                            .iter()
                            .map(|x| (x.name, x.symbol_type))
                            .collect();
                        assert_eq!(items, vec![("a_1", None), ("a_2", None)]);
                    } else {
                        panic!("parsing error")
                    }
                }
            }
            _ => panic!("parsing errors"),
        }
    }

    #[test]
    pub fn method_parsing_test() {
        let program = String::from(
            "(define (domain bal)
                (:method m_1
                    :parameters (?p1 - p ?l1 ?l2 ?l3 - loc) 
                    :task (deliver_abs ?p1 ?l1 ?l2)
                    :subtasks (and
                        (pickup ?p1 ?l1)
                        (deliver_abs ?p1 ?l2 ?l3)
                    )
                )
             ) ",
        )
        .into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match Parser::new(lexer).parse() {
            Ok(AbstractSyntaxTree::Domain(ast)) => {
                assert_eq!(ast.methods.len(), 1);
                let method = &ast.methods[0];
                assert_eq!(method.name.name, "m_1");
                assert_eq!(method.name.name_pos.line, 2);
                assert_eq!(method.task.name, "deliver_abs");
                assert_eq!(method.task.name_pos.line, 4);
                assert_eq!(method.task_terms.len(), 3);
                assert_eq!(method.task_terms[0].name, "p1");
                assert_eq!(method.task_terms[1].name, "l1");
                assert_eq!(method.task_terms[2].name, "l2");
                assert_eq!(method.tn.subtasks[0].task.name, "pickup");
                assert_eq!(method.tn.subtasks[0].task.name_pos.line, 6);
                let term_pos_lines1: Vec<u32> = method.tn.subtasks[0]
                    .terms
                    .iter()
                    .map(|x| x.name_pos.line)
                    .collect();
                assert_eq!(term_pos_lines1, vec![6, 6]);
                assert_eq!(method.tn.subtasks[0].terms[0].name, "p1");
                assert_eq!(method.tn.subtasks[0].terms[1].name, "l1");
                assert_eq!(method.tn.subtasks[1].task.name, "deliver_abs");
                let term_pos_lines2: Vec<u32> = method.tn.subtasks[1]
                    .terms
                    .iter()
                    .map(|x| x.name_pos.line)
                    .collect();
                assert_eq!(term_pos_lines2, vec![7, 7, 7]);
                assert_eq!(method.tn.subtasks[1].task.name_pos.line, 7);
                assert_eq!(method.tn.subtasks[1].terms[0].name, "p1");
                assert_eq!(method.tn.subtasks[1].terms[1].name, "l2");
                assert_eq!(method.tn.subtasks[1].terms[2].name, "l3");
                assert_eq!(method.precondition.is_none(), true);
            }
            _ => panic!("AST not created"),
        }
    }

    #[test]
    pub fn method_precondition_test() {
        let program = String::from(
            "(define (domain bal)
                (:method m_1
                    :parameters (?p1 - p ?l1 ?l2 ?l3 - loc) 
                    :task (deliver_abs ?p1 ?l1 ?l2)
                    :precondition (and (at ?p1 ?l3)
                    (driver ?l1) (not (= ?l1 ?l2)))
                    :subtasks (and
                        (pickup ?p1 ?l1)
                        (deliver_abs ?p1 ?l2 ?l3)
                    )
                )
             ) ",
        )
        .into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match Parser::new(lexer).parse() {
            Ok(AbstractSyntaxTree::Domain(ast)) => {
                assert_eq!(ast.methods.len(), 1);
                let method = &ast.methods[0];
                assert_eq!(method.name.name, "m_1");
                assert_eq!(method.task.name, "deliver_abs");
                assert_eq!(method.task_terms.len(), 3);
                assert_eq!(method.task_terms[0].name, "p1");
                assert_eq!(method.task_terms[1].name, "l1");
                assert_eq!(method.task_terms[2].name, "l2");
                assert_eq!(method.tn.subtasks[0].task.name, "pickup");
                assert_eq!(method.tn.subtasks[0].terms[0].name, "p1");
                assert_eq!(method.tn.subtasks[0].terms[1].name, "l1");
                assert_eq!(method.tn.subtasks[1].task.name, "deliver_abs");
                assert_eq!(method.tn.subtasks[1].terms[0].name, "p1");
                assert_eq!(method.tn.subtasks[1].terms[1].name, "l2");
                assert_eq!(method.tn.subtasks[1].terms[2].name, "l3");
                match &method.precondition {
                    Some(formula) => match formula {
                        Formula::And(predicates) => {
                            assert_eq!(predicates.len(), 3);
                            let pred1 = &*predicates[0];
                            match pred1 {
                                Formula::Atom(pred) => {
                                    assert_eq!(pred.name, "at");
                                    assert_eq!(pred.name_pos.line, 5);
                                    assert_eq!(pred.variables.len(), 2);
                                }
                                _ => {
                                    panic!("wrong formula parsing")
                                }
                            }
                            let pred2 = &*predicates[1];
                            match pred2 {
                                Formula::Atom(pred) => {
                                    assert_eq!(pred.name, "driver");
                                    assert_eq!(pred.name_pos.line, 6);
                                    assert_eq!(pred.variables.len(), 1);
                                }
                                _ => {
                                    panic!("wrong formula parsing")
                                }
                            }

                            let neq = &*predicates[2];
                            match neq {
                                Formula::Not(equality) => match **equality {
                                    Formula::Equals(a, b) => {
                                        assert_eq!(a, "l1");
                                        assert_eq!(b, "l2");
                                    }
                                    _ => {
                                        panic!("equality constraint not parsed successfully")
                                    }
                                },
                                _ => {
                                    panic!("wrong formula parsing")
                                }
                            }
                        }
                        _ => {
                            panic!("wrong formula parsing")
                        }
                    },
                    _ => {
                        panic!("wrong formula parsing")
                    }
                }
            }
            _ => panic!("AST not created"),
        }
    }

    #[test]
    pub fn universal_quantification_test() {
        let program = String::from(
            "(define (domain bal)
                (:method m_1
                    :parameters (?p1 - p ?l1 ?l2 ?l3 - loc) 
                    :task (deliver_abs ?p1 ?l1 ?l2)
                    :precondition (forall (?l1 ?l2 - loc) (= ?l1 ?l2))
                    :subtasks (and
                        (pickup ?p1 ?l1)
                        (deliver_abs ?p1 ?l2 ?l3)
                    )
                )
             ) ",
        )
        .into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match Parser::new(lexer).parse() {
            Ok(AbstractSyntaxTree::Domain(ast)) => {
                assert_eq!(ast.methods.len(), 1);
                let method = &ast.methods[0];
                assert_eq!(method.name.name, "m_1");
                assert_eq!(method.task.name, "deliver_abs");
                assert_eq!(method.task_terms.len(), 3);
                assert_eq!(method.task_terms[0].name, "p1");
                assert_eq!(method.task_terms[1].name, "l1");
                assert_eq!(method.task_terms[2].name, "l2");
                match &method.precondition {
                    Some(formula) => match formula {
                        Formula::ForAll(params, exp) => {
                            assert_eq!(params.len(), 2);
                            assert_eq!(params[0].name, "l1");
                            match params[0].symbol_type {
                                Some(x) => {
                                    assert_eq!(x, "loc");
                                }
                                _ => {
                                    panic!("wrong parameter type")
                                }
                            }
                            assert_eq!(params[1].name, "l2");
                            match params[1].symbol_type {
                                Some(x) => {
                                    assert_eq!(x, "loc");
                                }
                                _ => {
                                    panic!("wrong parameter type")
                                }
                            }
                            match **exp {
                                Formula::Equals(a, b) => {
                                    assert_eq!(a, "l1");
                                    assert_eq!(b, "l2");
                                }
                                _ => {
                                    panic!("wrong expression parsing")
                                }
                            }
                        }
                        _ => {
                            panic!("wrong formula parsing")
                        }
                    },
                    _ => {
                        panic!("wrong formula parsing")
                    }
                }
            }
            _ => panic!("AST not created"),
        }
    }

    #[test]
    pub fn init_state_parsing_test() {
        let program = String::from(
            "(define (problem p1) (domain bal)
             (:init
                (pred1 arg1 arg2)
                (pred2 arg1 arg2 arg3)
            ) ",
        )
        .into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match Parser::new(lexer).parse() {
            Ok(AbstractSyntaxTree::Problem(ast)) => {
                assert_eq!(ast.init_state.len(), 2);
                let pred1 = &ast.init_state[0];
                let pred2 = &ast.init_state[1];
                assert_eq!(pred1.name, "pred1");
                assert_eq!(pred1.variables.len(), 2);
                assert_eq!(pred1.variables[0].name, "arg1");
                assert_eq!(pred1.variables[1].name, "arg2");
                assert_eq!(pred2.name, "pred2");
                assert_eq!(pred2.variables.len(), 3);
                assert_eq!(pred2.variables[0].name, "arg1");
                assert_eq!(pred2.variables[1].name, "arg2");
                assert_eq!(pred2.variables[2].name, "arg3");
            }
            _ => {
                panic!("wrong AST")
            }
        }
    }

    #[test]
    pub fn init_tn_parsing_test() {
        let program = String::from(
            "(define (problem p1) (domain bal)
             (:htn
                :parameters (?d)
                :subtasks (and
                    (task0 (deliver package_0 
                            city_loc_0))
                    (task1 (retrieve package_1
                            city_loc_2 truck3))
                )
                :ordering (and
                    (< task0 task1)
                )
                :constraints (not (= term1 term2))
            ) ",
        )
        .into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match Parser::new(lexer).parse() {
            Ok(AbstractSyntaxTree::Problem(ast)) => match ast.init_tn {
                Some(tn) => {
                    match tn.parameters {
                        Some(p) => {
                            assert_eq!(p.len(), 1);
                            assert_eq!(p[0].name, "d");
                            assert_eq!(p[0].symbol_type.is_none(), true);
                        }
                        _ => panic!("wrong set of params"),
                    }
                    match tn.tn.orderings {
                        TaskOrdering::Partial(o) => {
                            assert_eq!(o.contains(&("task0", "task1")), true);
                            assert_eq!(o.len(), 1);
                        }
                        _ => {
                            panic!("ordering is not total")
                        }
                    }
                    assert_eq!(tn.tn.subtasks.len(), 2);
                    assert_eq!(tn.tn.subtasks[0].id.as_ref().unwrap().name, "task0");
                    assert_eq!(tn.tn.subtasks[0].task.name, "deliver");
                    assert_eq!(tn.tn.subtasks[0].terms.len(), 2);
                    assert_eq!(tn.tn.subtasks[0].id.as_ref().unwrap().name_pos.line, 5);
                    let s0_term_lines: Vec<u32> = tn.tn.subtasks[0]
                        .terms
                        .iter()
                        .map(|x| x.name_pos.line)
                        .collect();
                    assert_eq!(s0_term_lines, vec![5, 6]);
                    assert_eq!(tn.tn.subtasks[0].terms[0].name, "package_0");
                    assert_eq!(tn.tn.subtasks[0].terms[1].name, "city_loc_0");
                    assert_eq!(tn.tn.subtasks[1].id.as_ref().unwrap().name, "task1");
                    assert_eq!(tn.tn.subtasks[1].task.name, "retrieve");
                    assert_eq!(tn.tn.subtasks[1].terms.len(), 3);
                    assert_eq!(tn.tn.subtasks[1].id.as_ref().unwrap().name_pos.line, 7);
                    let s1_term_lines: Vec<u32> = tn.tn.subtasks[1]
                        .terms
                        .iter()
                        .map(|x| x.name_pos.line)
                        .collect();
                    assert_eq!(s1_term_lines, vec![7, 8, 8]);
                    assert_eq!(tn.tn.subtasks[1].terms[0].name, "package_1");
                    assert_eq!(tn.tn.subtasks[1].terms[1].name, "city_loc_2");
                    assert_eq!(tn.tn.subtasks[1].terms[2].name, "truck3");

                    match tn.tn.constraints {
                        Some(constraint) => {
                            assert_eq!(constraint.len(), 1);
                            match constraint[0] {
                                Constraint::NotEqual("term1", "term2") => {}
                                _ => {
                                    panic!("constraint not parsed correctly")
                                }
                            }
                        }
                        _ => {
                            panic!("constraints are not parsed")
                        }
                    }
                }
                None => {
                    panic!("init tn not parsed")
                }
            },
            _ => {
                panic!("failed to create AST")
            }
        }
    }

    #[test]
    pub fn no_op_test() {
        let program = String::from(
            "(define (domain bal)
                (:action a1
                    :parameters (?p1 - p ?l1 ?l2 ?l3 - loc)
                )
             ) ",
        )
        .into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match Parser::new(lexer).parse() {
            Ok(AbstractSyntaxTree::Domain(ast)) => {
                assert_eq!(ast.actions.len(), 1);
            }
            _ => panic!(),
        }
    }

    #[test]
    pub fn init_total_order_tn_parsing_test() {
        let program = String::from(
            "(define (problem p1) (domain bal)
             (:htn
                :ordered-tasks (and
                    (deliver package_0 city_loc_0)
                    (retrieve package_1 city_loc_2 truck3)
                )
            ) ",
        )
        .into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match Parser::new(lexer).parse() {
            Ok(AbstractSyntaxTree::Problem(ast)) => match ast.init_tn {
                Some(tn) => {
                    assert_eq!(tn.parameters.is_none(), true);
                    match tn.tn.orderings {
                        TaskOrdering::Total => {}
                        _ => {
                            panic!("ordering is not partial")
                        }
                    }
                    assert_eq!(tn.tn.subtasks.len(), 2);
                    assert_eq!(tn.tn.subtasks[0].id, None);
                    assert_eq!(tn.tn.subtasks[0].task.name, "deliver");
                    assert_eq!(tn.tn.subtasks[0].terms.len(), 2);
                    assert_eq!(tn.tn.subtasks[0].terms[0].name, "package_0");
                    assert_eq!(tn.tn.subtasks[0].terms[1].name, "city_loc_0");
                    assert_eq!(tn.tn.subtasks[1].id, None);
                    assert_eq!(tn.tn.subtasks[1].task.name, "retrieve");
                    assert_eq!(tn.tn.subtasks[1].terms.len(), 3);
                    assert_eq!(tn.tn.subtasks[1].terms[0].name, "package_1");
                    assert_eq!(tn.tn.subtasks[1].terms[1].name, "city_loc_2");
                    assert_eq!(tn.tn.subtasks[1].terms[2].name, "truck3");
                }
                None => panic!("tn not found"),
            },
            _ => panic!("false parsing"),
        }
    }

    #[test]
    pub fn compound_task_parsing_test() {
        let program = String::from(
            "(define (domain bal)
                (:predicates )
                (:task c_1
                 :parameters (p_1 p_2 - t1 p_3 - t2)
                )
             ) ",
        )
        .into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match Parser::new(lexer).parse() {
            Ok(AbstractSyntaxTree::Domain(ast)) => {
                assert_eq!(ast.compound_tasks.len(), 1);
                let c_1 = &ast.compound_tasks[0];
                assert_eq!(c_1.name, "c_1");
                assert_eq!(c_1.name_pos.line, 3);
                let c1_term_names: Vec<&str> = c_1.parameters.iter().map(|x| x.name).collect();
                let c1_term_types: Vec<&str> = c_1
                    .parameters
                    .iter()
                    .map(|x| x.symbol_type.unwrap())
                    .collect();
                assert_eq!(c1_term_names, vec!["p_1", "p_2", "p_3"]);
                assert_eq!(c1_term_types, vec!["t1", "t1", "t2"]);
                assert_eq!(c_1.parameters[0].type_pos.unwrap().line, 4);
            }
            _ => panic!("parsing errors"),
        }
    }

    #[test]
    pub fn action_parsing_test() {
        let program = String::from(
            "(define (domain bal)
                (:action a_1
                 :parameters (p_1 p_2 - t1 p_3 - t2)
                 :precondition (not (at p_1))
                 :effect (and (not (hold p_2 p_3))
                 (at p_2))
                )
             ) ",
        )
        .into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match Parser::new(lexer).parse() {
            Ok(AbstractSyntaxTree::Domain(ast)) => {
                assert_eq!(ast.actions.len(), 1);
                let action = &ast.actions[0];
                assert_eq!(action.name, "a_1");
                let a1_vars: Vec<&str> = action.parameters.iter().map(|x| x.name).collect();
                let a1_var_types: Vec<&str> = action
                    .parameters
                    .iter()
                    .map(|x| x.symbol_type.unwrap())
                    .collect();
                assert_eq!(a1_vars, vec!["p_1", "p_2", "p_3"]);
                assert_eq!(a1_var_types, vec!["t1", "t1", "t2"]);
                match &action.preconditions.as_ref().unwrap() {
                    Formula::Not(formula) => match &**formula {
                        Formula::Atom(predicate) => {
                            assert_eq!(predicate.name, "at");
                            assert_eq!(predicate.variables.len(), 1);
                            assert_eq!(predicate.variables[0].name, "p_1");
                        }
                        _ => {
                            panic!("wrong formula")
                        }
                    },
                    _ => panic!("wrong formula"),
                }
                match &action.effects.as_ref().unwrap() {
                    Formula::And(formula) => {
                        assert_eq!(formula.len(), 2);
                        if let Formula::Not(exp) = formula[0].as_ref() {
                            if let Formula::Atom(pred) = exp.as_ref() {
                                assert_eq!(pred.name, "hold");
                                assert_eq!(pred.variables.len(), 2);
                                assert_eq!(pred.variables[0].name, "p_2");
                                assert_eq!(pred.variables[1].name, "p_3");
                            } else {
                                panic!("wrong formula")
                            }
                        } else {
                            panic!("wrong formula")
                        };
                        if let Formula::Atom(pred) = formula[1].as_ref() {
                            assert_eq!(pred.name, "at");
                            assert_eq!(pred.variables.len(), 1);
                            assert_eq!(pred.variables[0].name, "p_2");
                        } else {
                            panic!("wrong formula")
                        }
                    }
                    _ => panic!("wrong formula"),
                }
            }
            _ => panic!("parsing errors"),
        }
    }

    #[test]
    pub fn quantifier_parsing_test() {
        let program = String::from(
            "(define (domain bal)
                (:action a_1
                 :parameters (p_1 p_2 - t1 p_3 - t2)
                 :precondition (and
                    (not (at p_1))
                    (exists (?num1 ?num2 - number) (smaller ?num1 ?num2))
                 )
                 :effect (and
                    (not (hold p_2 p_3))
                    (at p_2)
                    (forall (?loc - s) (and (pred1 ?loc) (pred2 ?loc)))
                 )
                )
             ) ",
        )
        .into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match Parser::new(lexer).parse() {
            Ok(AbstractSyntaxTree::Domain(ast)) => {
                assert_eq!(ast.actions.len(), 1);
                let action = &ast.actions[0];
                assert_eq!(action.name, "a_1");
                let a1_vars: Vec<&str> = action.parameters.iter().map(|x| x.name).collect();
                let a1_var_types: Vec<&str> = action
                    .parameters
                    .iter()
                    .map(|x| x.symbol_type.unwrap())
                    .collect();
                assert_eq!(a1_vars, vec!["p_1", "p_2", "p_3"]);
                assert_eq!(a1_var_types, vec!["t1", "t1", "t2"]);
                match &action.preconditions.as_ref().unwrap() {
                    Formula::And(exps) => {
                        assert_eq!(exps.len(), 2);
                        match &*exps[0] {
                            Formula::Not(formula) => match &**formula {
                                Formula::Atom(predicate) => {
                                    assert_eq!(predicate.name, "at");
                                    assert_eq!(predicate.variables.len(), 1);
                                    assert_eq!(predicate.variables[0].name, "p_1");
                                }
                                _ => {
                                    panic!("wrong formula")
                                }
                            },
                            _ => panic!(),
                        }
                        match &*exps[1] {
                            Formula::Exists(q, qs) => {
                                assert_eq!(q.len(), 2);
                                assert_eq!(q[0].name, "num1");
                                assert_eq!(q[1].name, "num2");
                                let predicates = qs.get_propositional_predicates();
                                assert_eq!(predicates.len(), 1);
                            }
                            _ => panic!(),
                        }
                    }

                    _ => panic!("wrong formula"),
                }
                match &action.effects.as_ref().unwrap() {
                    Formula::And(formula) => {
                        assert_eq!(formula.len(), 3);
                        match formula[0].as_ref() {
                            Formula::Not(exp) => {
                                if let Formula::Atom(pred) = exp.as_ref() {
                                    assert_eq!(pred.name, "hold");
                                    assert_eq!(pred.variables.len(), 2);
                                    assert_eq!(pred.variables[0].name, "p_2");
                                    assert_eq!(pred.variables[1].name, "p_3");
                                } else {
                                    panic!("wrong formula")
                                }
                            }
                            _ => panic!(),
                        }
                        match formula[1].as_ref() {
                            Formula::Atom(pred) => {
                                assert_eq!(pred.name, "at");
                                assert_eq!(pred.variables.len(), 1);
                                assert_eq!(pred.variables[0].name, "p_2");
                            }
                            _ => panic!(),
                        }
                        match formula[2].as_ref() {
                            Formula::ForAll(q, e) => {
                                assert_eq!(q.len(), 1);
                                assert_eq!(q[0].name, "loc");
                                assert_eq!(e.get_propositional_predicates().len(), 2);
                            }
                            _ => panic!(),
                        }
                    }
                    _ => panic!("wrong formula"),
                }
            }
            _ => panic!("parsing errors"),
        }
    }

    #[test]
    pub fn non_deterministic_action_parsing_test() {
        let program = String::from(
            "(define (domain bal)
                (:action a_1
                 :parameters (p_1 p_2 - t1 p_3 - t2)
                 :precondition (at p1)
                 :effect (oneof (not (hold p_2 p_3)) (at p_2))
                )
             ) ",
        )
        .into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match Parser::new(lexer).parse() {
            Ok(AbstractSyntaxTree::Domain(ast)) => {
                assert_eq!(ast.actions.len(), 1);
                let action = &ast.actions[0];
                assert_eq!(action.name, "a_1");
                let a1_vars: Vec<&str> = action.parameters.iter().map(|x| x.name).collect();
                let a1_var_types: Vec<&str> = action
                    .parameters
                    .iter()
                    .map(|x| x.symbol_type.unwrap())
                    .collect();
                assert_eq!(a1_vars, vec!["p_1", "p_2", "p_3"]);
                assert_eq!(a1_var_types, vec!["t1", "t1", "t2"]);
                match &action.effects.as_ref().unwrap() {
                    Formula::Xor(formula) => {
                        assert_eq!(formula.len(), 2);
                        if let Formula::Not(exp) = formula[0].as_ref() {
                            if let Formula::Atom(pred) = exp.as_ref() {
                                assert_eq!(pred.name, "hold");
                                assert_eq!(pred.variables.len(), 2);
                                assert_eq!(pred.variables[0].name, "p_2");
                                assert_eq!(pred.variables[1].name, "p_3");
                            } else {
                                panic!("wrong formula")
                            }
                        } else {
                            panic!("wrong formula")
                        };
                        if let Formula::Atom(pred) = formula[1].as_ref() {
                            assert_eq!(pred.name, "at");
                            assert_eq!(pred.variables.len(), 1);
                            assert_eq!(pred.variables[0].name, "p_2");
                        } else {
                            panic!("wrong formula")
                        }
                    }
                    _ => panic!("wrong formula"),
                }
            }
            _ => panic!("parsing errors"),
        }
    }

    #[test]
    pub fn probabilistic_action_parsing_test() {
        let program = String::from(
            "(define (domain bal)
                (:action a_1
                :parameters (p_1 p_2 - t1 p_3 - t2)
                :precondition (at p1)
                :effect (probabilistic
                    0.8 (and (not (at p_2 p_3)) (at p_2))
                    0.2 (probabilistic
                            0.5 (hold p_2 p_3)
                            0.5 (not (at p_2))
                        )
                    )
                )
            )",
        )
        .into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match Parser::new(lexer).parse() {
            Ok(AbstractSyntaxTree::Domain(ast)) => {
                assert_eq!(ast.actions.len(), 1);
                let action = &ast.actions[0];
                assert_eq!(action.name, "a_1");
                // a probabilistic block parses to the And of its Weighted branches
                match &action.effects.as_ref().unwrap() {
                    Formula::And(branches) => {
                        assert_eq!(branches.len(), 2);
                        // branch 1: 0.8 (and (not (at p_2 p_3)) (at p_2))
                        if let Formula::Weighted(weight, effect) = branches[0].as_ref() {
                            match weight {
                                NumberType::Real(x) => assert_eq!(*x, 0.8),
                                _ => panic!("Parsing number failed."),
                            }
                            if let Formula::And(conj) = effect.as_ref() {
                                assert_eq!(conj.len(), 2);
                                if let Formula::Not(inner) = conj[0].as_ref() {
                                    if let Formula::Atom(pred) = inner.as_ref() {
                                        assert_eq!(pred.name, "at");
                                        assert_eq!(pred.variables.len(), 2);
                                        assert_eq!(pred.variables[0].name, "p_2");
                                        assert_eq!(pred.variables[1].name, "p_3");
                                    } else {
                                        panic!("wrong formula")
                                    }
                                } else {
                                    panic!("wrong formula")
                                }
                                if let Formula::Atom(pred) = conj[1].as_ref() {
                                    assert_eq!(pred.name, "at");
                                    assert_eq!(pred.variables.len(), 1);
                                    assert_eq!(pred.variables[0].name, "p_2");
                                } else {
                                    panic!("wrong formula")
                                }
                            } else {
                                panic!("wrong formula")
                            }
                        } else {
                            panic!("expected a weighted branch")
                        }

                        // branch 2: 0.2 (probabilistic 0.5 (hold p_2 p_3) 0.5 (not (at p_2)))
                        if let Formula::Weighted(weight, effect) = branches[1].as_ref() {
                            match weight {
                                NumberType::Real(x) => assert_eq!(*x, 0.2),
                                _ => panic!("Parsing number failed."),
                            }
                            // the nested probabilistic is itself an And of Weighted
                            if let Formula::And(nested) = effect.as_ref() {
                                assert_eq!(nested.len(), 2);
                                if let Formula::Weighted(w, e) = nested[0].as_ref() {
                                    match w {
                                        NumberType::Real(x) => assert_eq!(*x, 0.5),
                                        _ => panic!("Parsing number failed."),
                                    }
                                    if let Formula::Atom(pred) = e.as_ref() {
                                        assert_eq!(pred.name, "hold");
                                        assert_eq!(pred.variables.len(), 2);
                                        assert_eq!(pred.variables[0].name, "p_2");
                                        assert_eq!(pred.variables[1].name, "p_3");
                                    } else {
                                        panic!("wrong formula")
                                    }
                                } else {
                                    panic!("expected a weighted branch")
                                }
                                if let Formula::Weighted(w, e) = nested[1].as_ref() {
                                    match w {
                                        NumberType::Real(x) => assert_eq!(*x, 0.5),
                                        _ => panic!("Parsing number failed."),
                                    }
                                    if let Formula::Not(inner) = e.as_ref() {
                                        if let Formula::Atom(pred) = inner.as_ref() {
                                            assert_eq!(pred.name, "at");
                                            assert_eq!(pred.variables.len(), 1);
                                            assert_eq!(pred.variables[0].name, "p_2");
                                        } else {
                                            panic!("wrong formula")
                                        }
                                    } else {
                                        panic!("wrong formula")
                                    }
                                } else {
                                    panic!("expected a weighted branch")
                                }
                            } else {
                                panic!("expected nested probabilistic")
                            }
                        } else {
                            panic!("expected a weighted branch")
                        }
                    }
                    _ => panic!("wrong formula"),
                }
            }
            _ => panic!("parsing errors"),
        }
    }

    #[test]
    pub fn var_type_declaration_test() {
        let program = String::from(
            "(define (domain blahblah)
                (:types
                    Port AbstractDevice - Object
                    AbstractCable Device - AbstractDevice
                    PlugType PlugFace PlugDirection SignalType - Enum
                )
             ) ",
        )
        .into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match Parser::new(lexer).parse() {
            Ok(AbstractSyntaxTree::Domain(ast)) => {
                let types = ast.types.unwrap();
                assert_eq!(types.len(), 8);
                assert_eq!(types[0].name, "Port");
                assert_eq!(types[0].symbol_type.unwrap(), "Object");
                assert_eq!(types[1].name, "AbstractDevice");
                assert_eq!(types[1].symbol_type.unwrap(), "Object");
                assert_eq!(types[2].name, "AbstractCable");
                assert_eq!(types[2].symbol_type.unwrap(), "AbstractDevice");
                assert_eq!(types[3].name, "Device");
                assert_eq!(types[3].symbol_type.unwrap(), "AbstractDevice");
                assert_eq!(types[4].name, "PlugType");
                assert_eq!(types[4].symbol_type.unwrap(), "Enum");
                assert_eq!(types[5].name, "PlugFace");
                assert_eq!(types[5].symbol_type.unwrap(), "Enum");
                assert_eq!(types[6].name, "PlugDirection");
                assert_eq!(types[6].symbol_type.unwrap(), "Enum");
                assert_eq!(types[7].name, "SignalType");
                assert_eq!(types[7].symbol_type.unwrap(), "Enum");
            }
            _ => panic!("parsing erro"),
        }
    }

    #[test]
    pub fn function_parsing_test() {
        let program = String::from(
            "(define (domain rover-domain)
            (:functions
                (battery-amount ?r - rover)
                (recharge-rate ?r - rover)
                (battery-capacity)
                (distance ?wp1 - waypoint ?wp2 - waypoint)
            )
        )",
        ).into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match Parser::new(lexer).parse() {
            Ok(AbstractSyntaxTree::Domain(ast)) => {
                assert_eq!(ast.functions.len(), 4);

                // (battery-amount ?r - rover)
                let battery_amount = &ast.functions[0];
                assert_eq!(battery_amount.name, "battery-amount");
                assert_eq!(battery_amount.variables.len(), 1);
                assert_eq!(battery_amount.variables[0].name, "r");
                if let Some(val) = battery_amount.variables[0].symbol_type {
                    assert_eq!(val, "rover")
                }

                // (recharge-rate ?r - rover)
                let recharge_rate = &ast.functions[1];
                assert_eq!(recharge_rate.name, "recharge-rate");
                assert_eq!(recharge_rate.variables.len(), 1);
                assert_eq!(recharge_rate.variables[0].name, "r");
                if let Some(val) = recharge_rate.variables[0].symbol_type {
                    assert_eq!(val, "rover")
                }

                // (battery-capacity)  -- zero-arity
                let battery_capacity = &ast.functions[2];
                assert_eq!(battery_capacity.name, "battery-capacity");
                assert_eq!(battery_capacity.variables.len(), 0);

                // (distance ?wp1 - waypoint ?wp2 - waypoint)
                let distance = &ast.functions[3];
                assert_eq!(distance.name, "distance");
                assert_eq!(distance.variables.len(), 2);
                assert_eq!(distance.variables[0].name, "wp1");
                if let Some(val) = distance.variables[0].symbol_type {
                    assert_eq!(val, "waypoint")
                }
                assert_eq!(distance.variables[1].name, "wp2");
                if let Some(val) = distance.variables[1].symbol_type {
                    assert_eq!(val, "waypoint")
                }
            }
            _ => panic!("parsing errors"),
        }
    }

    #[test]
    pub fn comment_test() {
        let program = String::from(
            ";author: me
            ; domain bal
            (define (domain bal)
                ;task c_1
                (:task c_1
                 :parameters (p_1 p_2 - t1 p_3 - t2) ;task parameters are defined here
                )
             ) ",
        )
        .into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match Parser::new(lexer).parse() {
            Ok(AbstractSyntaxTree::Domain(ast)) => {
                assert_eq!(ast.compound_tasks.len(), 1);
                let c_1 = &ast.compound_tasks[0];
                assert_eq!(c_1.name, "c_1");
                let c1_term_names: Vec<&str> = c_1.parameters.iter().map(|x| x.name).collect();
                let c1_term_types: Vec<&str> = c_1
                    .parameters
                    .iter()
                    .map(|x| x.symbol_type.unwrap())
                    .collect();
                assert_eq!(c1_term_names, vec!["p_1", "p_2", "p_3"]);
                assert_eq!(c1_term_types, vec!["t1", "t1", "t2"]);
            }
            _ => panic!("parsing errors"),
        }
    }

    #[test]
    pub fn constants_declaration_test() {
        let program = String::from(
            "(define (domain blahblah)
                (:constants
                    Port AbstractDevice - Object
                    AbstractCable Device - AbstractDevice
                    PlugType PlugFace PlugDirection SignalType - Enum
                )
             ) ",
        )
        .into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match Parser::new(lexer).parse() {
            Ok(AbstractSyntaxTree::Domain(ast)) => {
                let constants = ast.constants.unwrap();
                assert_eq!(constants.len(), 8);
                assert_eq!(constants[0].name, "Port");
                assert_eq!(constants[0].symbol_type.unwrap(), "Object");
                assert_eq!(constants[1].name, "AbstractDevice");
                assert_eq!(constants[1].symbol_type.unwrap(), "Object");
                assert_eq!(constants[2].name, "AbstractCable");
                assert_eq!(constants[2].symbol_type.unwrap(), "AbstractDevice");
                assert_eq!(constants[3].name, "Device");
                assert_eq!(constants[3].symbol_type.unwrap(), "AbstractDevice");
                assert_eq!(constants[4].name, "PlugType");
                assert_eq!(constants[4].symbol_type.unwrap(), "Enum");
                assert_eq!(constants[5].name, "PlugFace");
                assert_eq!(constants[5].symbol_type.unwrap(), "Enum");
                assert_eq!(constants[6].name, "PlugDirection");
                assert_eq!(constants[6].symbol_type.unwrap(), "Enum");
                assert_eq!(constants[7].name, "SignalType");
                assert_eq!(constants[7].symbol_type.unwrap(), "Enum");
            }
            _ => panic!("parsing erro"),
        }
    }

    #[test]
    pub fn last_character_bug_test() {
        let program = String::from(
            "(define (domain bal)
                (:action a_1
                 :parameters (p_1 p_2 - t1 p_3 - t2)
                 :precondition (not (at p_1))
                 :effect (and (not (hold p_2 p_3))
                 (at p_2))
                )
             )",
        )
        .into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match Parser::new(lexer).parse() {
            Ok(AbstractSyntaxTree::Domain(ast)) => {}
            Ok(_) => panic!(),
            Err(token) => panic!("{:?}", token),
        }
    }

    #[test]
    pub fn last_character_bug_2_test() {
        let program = String::from(
            "(define (domain bal)
                (:action a_1
                 :parameters (p_1 p_2 - t1 p_3 - t2)
                 :precondition (not (at p_1))
                 :effect (and (not (hold p_2 p_3))
                 (at p_2))
                )
             )             ",
        )
        .into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match Parser::new(lexer).parse() {
            Ok(AbstractSyntaxTree::Domain(ast)) => {}
            Ok(_) => panic!(),
            Err(token) => panic!("{:?}", token),
        }
    }

    #[test]
    pub fn empty_tn_test() {
        let program = String::from(
            "(define (problem p1) (domain bal)
             (:htn ) ",
        )
        .into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match Parser::new(lexer).parse() {
            Ok(AbstractSyntaxTree::Problem(ast)) => match ast.init_tn {
                Some(tn) => {
                    assert_eq!(tn.parameters.is_none(), true);
                    assert_eq!(tn.tn.subtasks.len(), 0);
                }
                _ => panic!(),
            },
            Err(token) => panic!("{:?}", token),
            _ => panic!(),
        }
    }

    #[test]
    pub fn file_classification_test() {
        let program = String::from(
            "(define (domain blahblah)
                (:constants
                    Port AbstractDevice - Object
                    AbstractCable Device - AbstractDevice
                    PlugType PlugFace PlugDirection SignalType - Enum
                )
             ) ",
        )
        .into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match Parser::new(lexer).classify() {
            FileVariant::Domain => {}
            _ => panic!(),
        }
    }
}
