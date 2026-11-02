use std::collections::HashSet;

use tdg::TDG;

use super::*;

#[test]
pub fn tdg_correctness_test() {
    let program = String::from(
        "(define (domain bal)
            (:predicates 
                (at ?l)
            )
            (:action p_1
            :parameters(?l1)
            :precondition (at ?l1)
            )
            (:action p_2
            :parameters(?l1)
            :precondition (at ?l1)
            )
            (:task abs_1 :parameters(?a))
            (:task abs_2 :parameters(?a))
            (:task abs_3 :parameters(?a))

            (:method m_1
                :parameters (?p1) 
                :task (abs_1 ?p1)
                :ordered-subtasks (and
                    (t1 (abs_1 ?p1))
                    (t2 (abs_2 ?p1))
                    (t3 (abs_3 ?p1))
                    (t4 (p_1 ?p1))
                )
            )
            (:method m_2
                :parameters (?p1) 
                :task (abs_1 ?p1)
                :ordered-subtasks ()
            )
        ) ",
    )
    .into_bytes();
    let problem = String::from(
        "
        (define (problem p-1-2-2)
            (:domain barman_htn)
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
    let ast = parser.parse().unwrap();
    match ast {
        AbstractSyntaxTree::Domain(d) => {
            let p_lexer = LexicalAnalyzer::new(&problem);
            let p_parser = Parser::new(p_lexer);
            let p_ast = p_parser.parse().unwrap();
            match p_ast {
                AbstractSyntaxTree::Problem(_) => {
                    let tdg = TDG::new(&d);
                    let reachable_abs_1 = tdg.reachable("abs_1");
                    assert_eq!(reachable_abs_1.compounds.len(), 3);
                    assert_eq!(reachable_abs_1.primitives.len(), 1);
                    assert_eq!(reachable_abs_1.compounds.contains("abs_1"), true);
                    assert_eq!(reachable_abs_1.compounds.contains("abs_2"), true);
                    assert_eq!(reachable_abs_1.compounds.contains("abs_3"), true);
                    assert_eq!(reachable_abs_1.primitives.contains("p_1"), true);
                    assert_eq!(reachable_abs_1.nullable, true);

                    let reachable_p_2 = tdg.reachable("p_2");
                    assert_eq!(reachable_p_2.primitives.len(), 1);
                    assert_eq!(reachable_p_2.primitives.contains("p_2"), true);
                    assert_eq!(reachable_p_2.nullable, false);
                }
                _ => panic!(),
            }
        }
        AbstractSyntaxTree::Problem(_) => panic!(),
    }
}

#[test]
pub fn tdg_non_recursive_test() {
    let program = String::from(
        "(define (domain bal)
            (:predicates 
                (at ?l)
            )
            (:action p_1
            :parameters(?l1)
            :precondition (at ?l1)
            )
            (:action p_2
            :parameters(?l1)
            :precondition (at ?l1)
            )
            (:task abs_1 :parameters(?a))
            (:task abs_2 :parameters(?a))
            (:task abs_3 :parameters(?a))

            (:method m_1
                :parameters (?p1) 
                :task (abs_1 ?p1)
                :ordered-subtasks (and
                    (t2 (abs_2 ?p1))
                    (t3 (abs_3 ?p1))
                    (t4 (p_1 ?p1))
                )
            )
        ) ",
    )
    .into_bytes();
    let problem = String::from(
        "
        (define (problem p-1-2-2)
            (:domain barman_htn)
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
    let ast = parser.parse().unwrap();
    match ast {
        AbstractSyntaxTree::Domain(d) => {
            let p_lexer = LexicalAnalyzer::new(&problem);
            let p_parser = Parser::new(p_lexer);
            let p_ast = p_parser.parse().unwrap();
            match p_ast {
                AbstractSyntaxTree::Problem(_) => {
                    let tdg = TDG::new(&d);
                    let nullables = tdg.compute_nullables();
                    let recursion_info = tdg.classify_cycles(&nullables);
                    assert!(recursion_info.recursive_tasks.len() == 0);
                    assert!(recursion_info.eps_prefix_tasks.len() == 0);
                    assert!(recursion_info.empty_recursive_tasks.len() == 0);
                    assert!(recursion_info.growing_empty_recursive_tasks.len() == 0);
                    assert!(recursion_info.grow_and_shrink_tasks.len() == 0);
                    assert!(recursion_info.acyclic_tasks.len() > 0);
                }
                _ => panic!(),
            }
        }
        AbstractSyntaxTree::Problem(_) => panic!(),
    }
}

#[test]
pub fn tdg_recursive_test() {
    let program = String::from(
        "(define (domain bal)
            (:predicates 
                (at ?l)
            )
            (:action p_1
            :parameters(?l1)
            :precondition (at ?l1)
            )
            (:action p_2
            :parameters(?l1)
            :precondition (at ?l1)
            )
            (:task abs_1 :parameters(?a))
            (:task abs_2 :parameters(?a))
            (:task abs_3 :parameters(?a))

            (:method m_1
                :parameters (?p1) 
                :task (abs_1 ?p1)
                :ordered-subtasks (and
                    (t0 (p_2 ?p1))
                    (t1 (abs_2 ?p1))
                    (t2 (abs_3 ?p1))
                    (t3 (abs_1 ?p1))
                    (t4 (p_1 ?p1))
                )
            )
        ) ",
    )
    .into_bytes();
    let problem = String::from(
        "
        (define (problem p-1-2-2)
            (:domain barman_htn)
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
    let ast = parser.parse().unwrap();
    match ast {
        AbstractSyntaxTree::Domain(d) => {
            let p_lexer = LexicalAnalyzer::new(&problem);
            let p_parser = Parser::new(p_lexer);
            let p_ast = p_parser.parse().unwrap();
            match p_ast {
                AbstractSyntaxTree::Problem(_) => {
                    let tdg = TDG::new(&d);
                    let nullables = tdg.compute_nullables();
                    let recursion_info = tdg.classify_cycles(&nullables);
                    assert!(recursion_info.recursive_tasks.len() > 0);
                }
                _ => panic!(),
            }
        }
        AbstractSyntaxTree::Problem(_) => panic!(),
    }
}

#[test]
pub fn tdg_grow_and_shrink_cycle_test() {
    let program = String::from(
        "(define (domain bal)
            (:predicates 
                (at ?l)
            )
            (:action p_1
            :parameters(?l1)
            :precondition (at ?l1)
            )
            (:action p_2
            :parameters(?l1)
            :precondition (at ?l1)
            )
            (:task abs_1 :parameters(?a))
            (:task abs_2 :parameters(?a))
            (:task abs_3 :parameters(?a))

            (:method m_1
                :parameters (?p1) 
                :task (abs_1 ?p1)
                :ordered-subtasks (and
                    (t0 (abs_3 ?p1))
                    (t1 (abs_1 ?p1))
                    (t2 (abs_3 ?p1))
                    (t3 (abs_3 ?p1))
                )
            )

            (:method m_2
                :parameters (?p1) 
                :task (abs_3 ?p1)
                :ordered-subtasks ()
            )
        ) ",
    )
    .into_bytes();
    let problem = String::from(
        "
        (define (problem p-1-2-2)
            (:domain barman_htn)
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
    let ast = parser.parse().unwrap();
    match ast {
        AbstractSyntaxTree::Domain(d) => {
            let p_lexer = LexicalAnalyzer::new(&problem);
            let p_parser = Parser::new(p_lexer);
            let p_ast = p_parser.parse().unwrap();
            match p_ast {
                AbstractSyntaxTree::Problem(_) => {
                    let tdg = TDG::new(&d);
                    let nullables = tdg.compute_nullables();
                    let recursion_info = tdg.classify_cycles(&nullables);
                    assert!(recursion_info.grow_and_shrink_tasks.len() > 0);
                }
                _ => panic!(),
            }
        }
        AbstractSyntaxTree::Problem(_) => panic!(),
    }
}

#[test]
pub fn tdg_grow_and_shrink_cycle_partial_order_1_test() {
    let program = String::from(
        "(define (domain bal)
            (:predicates 
                (at ?l)
            )
            (:action p_1
            :parameters(?l1)
            :precondition (at ?l1)
            )
            (:action p_2
            :parameters(?l1)
            :precondition (at ?l1)
            )
            (:task abs_1 :parameters(?a))
            (:task abs_2 :parameters(?a))
            (:task abs_3 :parameters(?a))

            (:method m_1
                :parameters (?p1) 
                :task (abs_1 ?p1)
                :subtasks (and
                    (t0 (abs_3 ?p1))
                    (t1 (abs_1 ?p1))
                    (t2 (abs_3 ?p1))
                    (t3 (abs_3 ?p1))
                )
                :ordering (and
                    (< t0 t1)
                    (< t1 t2)
                    (< t2 t3)
                )
            )

            (:method m_2
                :parameters (?p1) 
                :task (abs_3 ?p1)
                :subtasks ()
            )
        ) ",
    )
    .into_bytes();
    let problem = String::from(
        "
        (define (problem p-1-2-2)
            (:domain barman_htn)
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
    let ast = parser.parse().unwrap();
    match ast {
        AbstractSyntaxTree::Domain(d) => {
            let p_lexer = LexicalAnalyzer::new(&problem);
            let p_parser = Parser::new(p_lexer);
            let p_ast = p_parser.parse().unwrap();
            match p_ast {
                AbstractSyntaxTree::Problem(_) => {
                    let tdg = TDG::new(&d);
                    let nullables = tdg.compute_nullables();
                    let recursion_info = tdg.classify_cycles(&nullables);
                    assert!(recursion_info.grow_and_shrink_tasks.len() > 0);
                }
                _ => panic!(),
            }
        }
        AbstractSyntaxTree::Problem(_) => panic!(),
    }
}

#[test]
pub fn tdg_growing_cycle_test() {
    let program = String::from(
        "(define (domain bal)
            (:predicates 
                (at ?l)
            )
            (:action p_1
            :parameters(?l1)
            :precondition (at ?l1)
            )
            (:action p_2
            :parameters(?l1)
            :precondition (at ?l1)
            )
            (:task abs_1 :parameters(?a))
            (:task abs_2 :parameters(?a))
            (:task abs_3 :parameters(?a))

            (:method m_1
                :parameters (?p1) 
                :task (abs_1 ?p1)
                :ordered-subtasks (and
                    (t1 (abs_1 ?p1))
                    (t2 (abs_3 ?p1))
                    (t3 (abs_1 ?p1))
                    (t4 (p_1 ?p1))
                )
            )

            (:method m_2
                :parameters (?p1) 
                :task (abs_3 ?p1)
                :ordered-subtasks (and
                    (t1 (p_1 ?p2))
                    (t2 (p_1 ?p2))
                )
            )
        ) ",
    )
    .into_bytes();
    let problem = String::from(
        "
        (define (problem p-1-2-2)
            (:domain barman_htn)
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
    let ast = parser.parse().unwrap();
    match ast {
        AbstractSyntaxTree::Domain(d) => {
            let p_lexer = LexicalAnalyzer::new(&problem);
            let p_parser = Parser::new(p_lexer);
            let p_ast = p_parser.parse().unwrap();
            match p_ast {
                AbstractSyntaxTree::Problem(_) => {
                    let tdg = TDG::new(&d);
                    let nullables = tdg.compute_nullables();
                    let recursion_info = tdg.classify_cycles(&nullables);
                    assert!(recursion_info.growing_empty_recursive_tasks.len() > 0);
                }
                _ => panic!(),
            }
        }
        AbstractSyntaxTree::Problem(_) => panic!(),
    }
}

#[test]
pub fn satelite_domain_cycle_test() {
    let program = String::from(
        "(define (domain bal)
            (:predicates 
                (at ?l)
            )
            (:action p_1
            :parameters(?l1)
            :precondition (at ?l1)
            )
            (:action p_2
            :parameters(?l1)
            :precondition (at ?l1)
            )

            (:task do_prepare :parameters (?s - satellite ?i - instrument ?d - direction) )
            (:task do_switching :parameters (?s - satellite ?i - instrument) )
            (:task do_calibration :parameters (?s - satellite ?i - instrument ?d - direction) )
            (:task do_turning :parameters (?s - satellite ?d - direction) )

            (:method m1_do_prepare
                :parameters ( ?s - satellite  ?i - instrument  ?d - direction )
                :task (do_prepare ?s ?i ?d)
                :precondition ()
                :ordered-subtasks(and (t1 (do_switching ?s ?i)) (t2 (do_turning ?s ?d)))
            )
            (:method m3_do_switching
                :parameters ( ?s - satellite  ?i - instrument ?d - direction )
                :task (do_switching ?s ?i)
                :precondition (and (on_board ?i ?s) (power_avail ?s))
                :ordered-subtasks(and (t1 (switch_on ?i ?s)) (t2 (do_calibration ?s ?i ?d)))
            )
            (:method m5_do_calibration
                :parameters ( ?s - satellite  ?i - instrument  ?d - direction )
                :task (do_calibration ?s ?i ?d)
                :precondition (and (not(calibrated ?i)))
                :ordered-subtasks(and (t1 (do_prepare ?s ?i ?d)) (t2 (calibrate ?s ?i ?d)))
            )

            (:action switch_on
                :parameters (?i - instrument ?s - satellite)
                :precondition (and (on_board ?i ?s) (power_avail ?s))
                :effect (and (power_on ?i) (not (calibrated ?i)) (not (power_avail ?s)))
            )
            (:action calibrate
                :parameters (?s - satellite ?i - instrument ?d - direction)
                :precondition (and (on_board ?i ?s) (calibration_target ?i ?d) (pointing ?s ?d) (power_on ?i))
                :effect (calibrated ?i)
            )
        ) ",
    )
    .into_bytes();
    let problem = String::from(
        "
        (define (problem p-1-2-2)
            (:domain barman_htn)
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
    let ast = parser.parse().unwrap();
    match ast {
        AbstractSyntaxTree::Domain(d) => {
            let p_lexer = LexicalAnalyzer::new(&problem);
            let p_parser = Parser::new(p_lexer);
            let p_ast = p_parser.parse().unwrap();
            match p_ast {
                AbstractSyntaxTree::Problem(_) => {
                    let tdg = TDG::new(&d);
                    let nullables = tdg.compute_nullables();
                    let recursion_info = tdg.classify_cycles(&nullables);
                    assert!(recursion_info.recursive_tasks.len() > 0);
                }
                _ => panic!(),
            }
        }
        AbstractSyntaxTree::Problem(_) => panic!(),
    }
}

#[test]
pub fn tdg_nullables_test() {
    let program = String::from(
        "(define (domain bal)
            (:predicates 
                (at ?l)
            )
            (:action p_1
            :parameters(?l1)
            :precondition (at ?l1)
            )
            (:action p_2
            :parameters(?l1)
            :precondition (at ?l1)
            )
            (:task abs_1 :parameters(?a))
            (:task abs_2 :parameters(?a))
            (:task abs_3 :parameters(?a))

            (:method m_1
                :parameters (?p1) 
                :task (abs_1 ?p1)
                :ordered-subtasks (and
                    (t0 (abs_3 ?p1))
                    (t1 (abs_2 ?p1))
                    (t2 (abs_3 ?p1))
                    (t3 (abs_3 ?p1))
                )
            )

            (:method m_2
                :parameters (?p1) 
                :task (abs_2 ?p1)
                :ordered-subtasks ()
            )

            (:method m_3
                :parameters (?p1) 
                :task (abs_3 ?p1)
                :ordered-subtasks (
                    abs_2 ?p1
                )
            )

            (:method m_3
                :parameters (?1) 
                :task (abs_3 ?p1)
                :ordered-subtasks (and
                    (p_1 ?l1)
                    (p_2 ?l1)
                )
            )
        ) ",
    )
    .into_bytes();
    let problem = String::from(
        "
        (define (problem p-1-2-2)
            (:domain barman_htn)
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
    let ast = parser.parse().unwrap();
    match ast {
        AbstractSyntaxTree::Domain(d) => {
            let p_lexer = LexicalAnalyzer::new(&problem);
            let p_parser = Parser::new(p_lexer);
            let p_ast = p_parser.parse().unwrap();
            match p_ast {
                AbstractSyntaxTree::Problem(_) => {
                    let tdg = TDG::new(&d);
                    assert_eq!(
                        tdg.compute_nullables(),
                        HashSet::from(["abs_1", "abs_2", "abs_3"])
                    )
                }
                _ => panic!(),
            }
        }
        AbstractSyntaxTree::Problem(_) => panic!(),
    }
}
