extern crate hddl_analyzer;

use hddl_analyzer::{HDDLAnalyzer, ParsingError, SemanticErrorType, WarningType};
use std::fs;

#[test]
pub fn cyclic_ordering_validation_test() {
    let domain = fs::read(
        "tests/flawed_domains/cyclic-ordering-for-subtasks-domain.hddl"
    ).unwrap();
    match HDDLAnalyzer::verify(&domain, None) {
        Ok(_) => panic!("error not found"),
        Err(err) => {
            if let hddl_analyzer::ParsingError::Semantic(x) = err {
                if let SemanticErrorType::CyclicOrderingDeclaration(t) = x {
                    assert_eq!(t.line, 56);
                } else {
                    panic!("wrong error {:?}", x)
                }
            } else {
                panic!()
            }
        }
    }
}

#[test]
pub fn cyclic_type_validation_test() {
    let domain = fs::read(
        "tests/flawed_domains/directly-cyclic-subtypes-domain.hddl"
    ).unwrap();
    match HDDLAnalyzer::verify(&domain, None) {
        Ok(_) => panic!("error not found"),
        Err(err) => {
            if let hddl_analyzer::ParsingError::Semantic(x) = err {
                if let SemanticErrorType::CyclicTypeDeclaration = x {
                    
                } else {
                    panic!("wrong error {:?}", x)
                }
            } else {
                panic!()
            }
        }
    }

    let domain = fs::read(
        "tests/flawed_domains/indirectly-cyclic-subtypes-domain.hddl"
    ).unwrap();
    match HDDLAnalyzer::verify(&domain, None) {
        Ok(_) => panic!("error not found"),
        Err(err) => {
            if let hddl_analyzer::ParsingError::Semantic(x) = err {
                if let SemanticErrorType::CyclicTypeDeclaration = x {
                    
                } else {
                    panic!("wrong error {:?}", x)
                }
            } else {
                panic!()
            }
        }
    }
}

#[test]
pub fn duplicate_action_validation_test() {
    let domain = fs::read(
        "tests/flawed_domains/duplicate-action-domain.hddl"
    ).unwrap();
    match HDDLAnalyzer::verify(&domain, None) {
        Ok(_) => panic!("error not found"),
        Err(err) => {
            if let hddl_analyzer::ParsingError::Semantic(x) = err {
                if let SemanticErrorType::DuplicateActionDeclaration(t) = x {
                    assert_eq!(t.symbol, "move_seg_twe1_0_200_seg_twe2_0_50_south_south_medium");
                } else {
                    panic!("wrong error {:?}", x)
                }
            } else {
                panic!()
            }
        }
    }
}

#[test]
pub fn duplicate_task_validation_test() {
    let domain = fs::read(
        "tests/flawed_domains/duplicate-compound-task-domain.hddl"
    ).unwrap();
    match HDDLAnalyzer::verify(&domain, None) {
        Ok(_) => panic!("error not found"),
        Err(err) => {
            if let hddl_analyzer::ParsingError::Semantic(x) = err {
                if let SemanticErrorType::DuplicateCompoundTaskDeclaration(t) = x {
                    assert_eq!(t.symbol, "AchieveSomeGoal");
                } else {
                    panic!("wrong error {:?}", x)
                }
            } else {
                panic!()
            }
        }
    }
}

#[test]
pub fn duplicate_method_validation_test() {
    let domain = fs::read(
        "tests/flawed_domains/duplicate-decomposition-method-domain.hddl"
    ).unwrap();
    match HDDLAnalyzer::verify(&domain, None) {
        Ok(_) => panic!("error not found"),
        Err(err) => {
            if let hddl_analyzer::ParsingError::Semantic(x) = err {
                if let SemanticErrorType::DuplicateMethodDeclaration(t) = x {
                    assert_eq!(t.symbol, "ParkAirplane");
                } else {
                    panic!("wrong error {:?}", x)
                }
            } else {
                panic!()
            }
        }
    }
}

#[test]
pub fn duplicate_predicate_validation_test() {
    let domain = fs::read(
        "tests/flawed_domains/duplicate-predicate-domain.hddl"
    ).unwrap();
    match HDDLAnalyzer::verify(&domain, None) {
        Ok(_) => panic!("error not found"),
        Err(err) => {
            if let hddl_analyzer::ParsingError::Semantic(x) = err {
                if let SemanticErrorType::DuplicatePredicateDeclaration(t) = x {
                    assert_eq!(t.symbol, "at-segment");
                } else {
                    panic!("wrong error {:?}", x)
                }
            } else {
                panic!()
            }
        }
    }
}

#[test]
pub fn duplicate_parameter_validation_test() {
    let domain = fs::read(
        "tests/flawed_domains/duplicate-parameters-domain.hddl"
    ).unwrap();
    match HDDLAnalyzer::verify(&domain, None) {
        Ok(_) => panic!("error not found"),
        Err(err) => {
            if let hddl_analyzer::ParsingError::Semantic(x) = err {
                if let SemanticErrorType::DuplicateParameterDeclaration(t) = x {
                    assert_eq!(t.symbol, "a");
                } else {
                    panic!("wrong error {:?}", x)
                }
            } else {
                panic!()
            }
        }
    }
}

#[test]
pub fn extra_parantheses_validation_test() {
    let domain = fs::read(
        "tests/flawed_domains/extra-parentheses-domain.hddl"
    ).unwrap();
    match HDDLAnalyzer::verify(&domain, None) {
        Ok(_) => panic!("error not found"),
        Err(err) => {
            if let hddl_analyzer::ParsingError::Syntactic(x) = err {
                assert_eq!(x.found, "Keyword :effect");
            } else {
                panic!()
            }
        }
    }
}

#[test]
#[ignore="fix"]
pub fn forgotten_dash_validation_test() {
    let domain = fs::read(
        "tests/flawed_domains/forgotten-dash-domain.hddl"
    ).unwrap();
    match HDDLAnalyzer::verify(&domain, None) {
        Ok(_) => panic!("error not found"),
        Err(err) => {
            if let hddl_analyzer::ParsingError::Syntactic(x) = err {
                assert_eq!(x.found, "Keyword :effect");
            } else {
                panic!()
            }
        }
    }
}

#[test]
pub fn forgotten_entry_validation_test() {
    let domain = fs::read(
        "tests/flawed_domains/forgotten-entries-domain.hddl"
    ).unwrap();
    match HDDLAnalyzer::verify(&domain, None) {
        Ok(_) => panic!("error not found"),
        Err(err) => {
            if let hddl_analyzer::ParsingError::Syntactic(x) = err {
                assert_eq!(x.position.line, 63);
            } else {
                panic!()
            }
        }
    }
}

#[test]
#[ignore = "fix"]
pub fn forgotten_question_mark_validation_test() {
    let domain = fs::read(
        "tests/flawed_domains/forgotten-question-mark-domain.hddl"
    ).unwrap();
    match HDDLAnalyzer::verify(&domain, None) {
        Ok(_) => panic!("error not found"),
        Err(err) => {
            if let hddl_analyzer::ParsingError::Syntactic(x) = err {
                assert_eq!(x.position.line, 63);
            } else {
                panic!()
            }
        }
    }
}

#[test]
pub fn inconsistent_arity_predicate_validation_test() {
    let domain = fs::read(
        "tests/flawed_domains/inconsistent-num-parameters-predicate-domain.hddl"
    ).unwrap();
    match HDDLAnalyzer::verify(&domain, None) {
        Ok(_) => panic!("error not found"),
        Err(err) => {
            if let hddl_analyzer::ParsingError::Semantic(x) = err {
                if let SemanticErrorType::InconsistentPredicateArity(t) = x {
                    assert_eq!(t.symbol, "at-segment");
                } else {
                    panic!()
                }
            } else {
                panic!()
            }
        }
    }
}

#[test]
pub fn inconsistent_type_predicate_validation_test() {
    let domain = fs::read(
        "tests/flawed_domains/inconsistent-type-parameters-predicate-domain.hddl"
    ).unwrap();
    match HDDLAnalyzer::verify(&domain, None) {
        Ok(_) => panic!("error not found"),
        Err(err) => {
            if let hddl_analyzer::ParsingError::Semantic(x) = err {
                if let SemanticErrorType::InconsistentPredicateArgType(t) = x {
                    assert_eq!(t.var_name, "seg_pp_0_60");
                } else {
                    panic!()
                }
            } else {
                panic!()
            }
        }
    }
}

#[test]
pub fn inconsistent_arity_task_validation_test() {
    let domain = fs::read(
        "tests/flawed_domains/inconsistent-num-parameters-task-domain.hddl"
    ).unwrap();
    match HDDLAnalyzer::verify(&domain, None) {
        Ok(_) => panic!("error not found"),
        Err(err) => {
            if let hddl_analyzer::ParsingError::Semantic(x) = err {
                if let SemanticErrorType::InconsistentTaskArity(t) = x {
                    assert_eq!(t.symbol, "move_seg_ppdoor_0_40_seg_tww1_0_200_north_south_medium");
                } else {
                    panic!()
                }
            } else {
                panic!()
            }
        }
    }
}

#[test]
pub fn inconsistent_type_task_validation_test() {
    let domain = fs::read(
        "tests/flawed_domains/inconsistent-type-parameters-task-domain.hddl"
    ).unwrap();
    match HDDLAnalyzer::verify(&domain, None) {
        Ok(_) => panic!("error not found"),
        Err(err) => {
            if let hddl_analyzer::ParsingError::Semantic(x) = err {
                if let SemanticErrorType::InconsistentTaskArgType(t) = x {
                    assert_eq!(t.var_name, "a_0");
                } else {
                    panic!()
                }
            } else {
                panic!()
            }
        }
    }
}

#[test]
pub fn undeclared_method_param_validation_test() {
    let domain = fs::read(
        "tests/flawed_domains/undeclared-method-parameter-domain.hddl"
    ).unwrap();
    match HDDLAnalyzer::verify(&domain, None) {
        Ok(_) => panic!("error not found"),
        Err(err) => {
            if let hddl_analyzer::ParsingError::Semantic(x) = err {
                if let SemanticErrorType::UndefinedParameter(t) = x {
                    assert_eq!(t.symbol, "d");
                } else {
                    panic!()
                }
            } else {
                panic!()
            }
        }
    }
}

#[test]
pub fn undeclared_task_param_validation_test() {
    let domain = fs::read(
        "tests/flawed_domains/undeclared-task-parameter-domain.hddl"
    ).unwrap();
    match HDDLAnalyzer::verify(&domain, None) {
        Ok(_) => panic!("error not found"),
        Err(err) => {
            if let hddl_analyzer::ParsingError::Semantic(x) = err {
                if let SemanticErrorType::UndefinedParameter(t) = x {
                    assert_eq!(t.symbol, "s");
                } else {
                    panic!()
                }
            } else {
                panic!()
            }
        }
    }
}

#[test]
pub fn undeclared_predicate_validation_test() {
    let domain = fs::read(
        "tests/flawed_domains/undefined-predicate-domain.hddl"
    ).unwrap();
    match HDDLAnalyzer::verify(&domain, None) {
        Ok(_) => panic!("error not found"),
        Err(err) => {
            if let hddl_analyzer::ParsingError::Semantic(x) = err {
                if let SemanticErrorType::UndefinedPredicate(t) = x {
                    assert_eq!(t.symbol, "occupied");
                } else {
                    panic!()
                }
            } else {
                panic!()
            }
        }
    }
}

#[test]
pub fn undeclared_task_validation_test() {
    let domain = fs::read(
        "tests/flawed_domains/undefined-task-domain.hddl"
    ).unwrap();
    match HDDLAnalyzer::verify(&domain, None) {
        Ok(_) => panic!("error not found"),
        Err(err) => {
            if let hddl_analyzer::ParsingError::Semantic(x) = err {
                if let SemanticErrorType::UndefinedSubtask(t) = x {
                    assert_eq!(t.symbol, "undefined_task");
                } else {
                    panic!()
                }
            } else {
                panic!()
            }
        }
    }
}

#[test]
pub fn undeclared_type_validation_test() {
    let domain = fs::read(
        "tests/flawed_domains/undefined-type-domain.hddl"
    ).unwrap();
    match HDDLAnalyzer::verify(&domain, None) {
        Ok(_) => panic!("error not found"),
        Err(err) => {
            if let hddl_analyzer::ParsingError::Semantic(x) = err {
                if let SemanticErrorType::UndefinedType(t) = x {
                    assert_eq!(t.symbol, "airplane");
                } else {
                    panic!()
                }
            } else {
                panic!()
            }
        }
    }
}

#[test]
pub fn no_primitive_refinement_validation_test() {
    let domain = fs::read(
        "tests/flawed_domains/abstract-task-without-refinement-domain.hddl"
    ).unwrap();
    match HDDLAnalyzer::verify(&domain, None) {
        Ok(warnings) => {
            assert_eq!(warnings.len(), 1);
            match &warnings[0] {
                WarningType::NoPrimitiveRefinement(x) => {
                    assert_eq!(x.symbol, "AchieveSomeGoal")
                }
                _ => panic!()
            }
        },
        Err(err) => {
            panic!()
        }
    }
}

#[test]
pub fn no_method_validation_test() {
    let domain = fs::read(
        "tests/flawed_domains/abstract-task-without-decomposition-domain.hddl"
    ).unwrap();
    match HDDLAnalyzer::verify(&domain, None) {
        Ok(warnings) => {
            assert_eq!(warnings.len(), 1);
            match &warnings[0] {
                WarningType::NoPrimitiveRefinement(x) => {
                    assert_eq!(x.symbol, "AchieveSomeGoal")
                }
                _ => panic!()
            }
        },
        Err(err) => {
            panic!()
        }
    }
}

#[test]
pub fn ignore_possibly_complementary_effects_validation_test() {
    let domain = fs::read(
        "tests/flawed_domains/possible-complementary-effects-domain.hddl"
    ).unwrap();
    match HDDLAnalyzer::verify(&domain, None) {
        Ok(_) => {},
        Err(err) => {
            panic!()
        }
    }
}