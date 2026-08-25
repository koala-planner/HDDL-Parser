extern crate hddl_analyzer;

use hddl_analyzer::{HDDLProgram, ParsingError, SemanticErrorType, Transpiler};
use std::fs;

#[test]
pub fn json_round_trip_single_domain() {
    let domain = fs::read("tests/ipc/Blocksworld-GTOHP/domain.hddl").unwrap();
    let program = HDDLProgram::from_hddl(&domain, None).unwrap();
    let exported = Transpiler::new(program).to_json();
    let reimported = HDDLProgram::from_json(&exported).unwrap();
    let re_exported = Transpiler::new(reimported).to_json();
    assert_eq!(exported, re_exported);
}

#[test]
#[ignore = "takes a long time"]
pub fn json_round_trip_ipc() {
    for folder in fs::read_dir("tests/ipc").unwrap() {
        let path = folder.as_ref().unwrap().path();
        let domain_path = fs::read_dir(path.clone())
            .unwrap()
            .find(|x| x.as_ref().unwrap().file_name() == "domain.hddl")
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .path();
        let domain = fs::read(&domain_path).unwrap();
        for file in fs::read_dir(path).unwrap() {
            if file.as_ref().unwrap().file_name() == "domain.hddl" {
                continue;
            }
            let problem_path = file.as_ref().unwrap().path();
            let problem = fs::read(&problem_path).unwrap();
            let program = HDDLProgram::from_hddl(&domain, Some(&problem)).unwrap();
            let exported = Transpiler::new(program).to_json();
            let reimported = match HDDLProgram::from_json(&exported) {
                Ok(program) => program,
                Err(error) => panic!(
                    "Domain: {:?} \nProblem:{:?}\nImport error: {:?}",
                    domain_path, problem_path, error
                ),
            };
            if let Err(error) = reimported.verify() {
                panic!(
                    "Domain: {:?} \nProblem:{:?}\nVerification error: {:?}",
                    domain_path, problem_path, error
                );
            }
            let re_exported = Transpiler::new(reimported).to_json();
            assert_eq!(
                exported, re_exported,
                "round trip mismatch for {:?} / {:?}",
                domain_path, problem_path
            );
        }
    }
}

#[test]
pub fn json_round_trip_metadata() {
    let domain = fs::read("tests/ipc/Blocksworld-GTOHP/domain.hddl").unwrap();
    let program = HDDLProgram::from_hddl(&domain, None).unwrap();
    let original = program.metadata().unwrap();
    let exported = Transpiler::new(program).to_json();
    let reimported = HDDLProgram::from_json(&exported).unwrap();
    let recovered = reimported.metadata().unwrap();
    assert_eq!(original.domain_name, recovered.domain_name);
    assert_eq!(original.n_actions, recovered.n_actions);
    assert_eq!(original.n_tasks, recovered.n_tasks);
    assert_eq!(original.n_methods, recovered.n_methods);
    assert_eq!(original.recursion, recovered.recursion);
    // nullables are collected from a set, so their order is nondeterministic
    let mut original_nullables = original.nullables.clone();
    let mut recovered_nullables = recovered.nullables.clone();
    original_nullables.sort();
    recovered_nullables.sort();
    assert_eq!(original_nullables, recovered_nullables);
}

#[test]
pub fn json_minimal_handwritten() {
    let json = r#"{
        "domain": {
            "name": "minimal",
            "requirements": [],
            "predicates": [],
            "compound_tasks": [],
            "methods": [],
            "actions": [{"name": "noop", "parameters": []}],
            "functions": []
        }
    }"#;
    let program = HDDLProgram::from_json(json).unwrap();
    assert_eq!(program.domain.actions.len(), 1);
    assert_eq!(program.domain.actions[0].name, "noop");
    // omitted positions default to the "synthesized" sentinel, line 0
    assert_eq!(program.domain.actions[0].name_pos.line, 0);
    assert!(program.problem.is_none());
    program.verify().unwrap();
}

#[test]
pub fn json_symbol_type_without_type_pos_no_panic() {
    // "vehicle" is not a declared type, and the variable carries no type_pos;
    let json = r#"{
        "domain": {
            "name": "d",
            "requirements": [],
            "predicates": [{
                "name": "at",
                "variables": [{"name": "v", "symbol_type": "vehicle"}]
            }],
            "compound_tasks": [],
            "methods": [],
            "actions": [],
            "functions": []
        }
    }"#;
    let program = HDDLProgram::from_json(json).unwrap();
    match program.verify() {
        Err(ParsingError::Semantic(SemanticErrorType::UndefinedType(undefined))) => {
            assert_eq!(undefined.symbol, "vehicle");
            assert_eq!(undefined.position.line, 0);
        }
        other => panic!("expected an UndefinedType error, got {:?}", other),
    }
}

#[test]
pub fn json_cyclic_ordering_no_pos_domain_no_panic() {
    let json = r#"{
        "domain": {
            "name": "d",
            "requirements": [],
            "predicates": [],
            "compound_tasks": [{"name": "ct", "parameters": []}],
            "methods": [{
                "name": {"name": "m"},
                "task": {"name": "ct"},
                "task_terms": [],
                "params": [],
                "tn": {
                    "subtasks": [
                        {"id": {"name": "t1"}, "task": {"name": "a1"}, "terms": []},
                        {"id": {"name": "t2"}, "task": {"name": "a1"}, "terms": []}
                    ],
                    "orderings": {"Partial": [["t1", "t2"], ["t2", "t1"]]}
                }
            }],
            "actions": [{"name": "a1", "parameters": []}],
            "functions": []
        }
    }"#;
    let program = HDDLProgram::from_json(json).unwrap();
    match program.verify() {
        Err(ParsingError::Semantic(SemanticErrorType::CyclicOrderingDeclaration(pos))) => {
            assert_eq!(pos.line, 0);
        }
        other => panic!(
            "expected a CyclicOrderingDeclaration error, got {:?}",
            other
        ),
    }
}

#[test]
pub fn json_cyclic_ordering_no_pos_problem_no_panic() {
    let json = r#"{
        "domain": {
            "name": "d",
            "requirements": [],
            "predicates": [],
            "compound_tasks": [],
            "methods": [],
            "actions": [{"name": "a1", "parameters": []}],
            "functions": []
        },
        "problem": {
            "requirements": [],
            "init_tn": {
                "tn": {
                    "subtasks": [
                        {"id": {"name": "t1"}, "task": {"name": "a1"}, "terms": []},
                        {"id": {"name": "t2"}, "task": {"name": "a1"}, "terms": []}
                    ],
                    "orderings": {"Partial": [["t1", "t2"], ["t2", "t1"]]}
                }
            },
            "init_state": [],
            "objects": []
        }
    }"#;
    let program = HDDLProgram::from_json(json).unwrap();
    match program.verify() {
        Err(ParsingError::Semantic(SemanticErrorType::CyclicOrderingDeclaration(pos))) => {
            assert_eq!(pos.line, 0);
        }
        other => panic!(
            "expected a CyclicOrderingDeclaration error, got {:?}",
            other
        ),
    }
}

#[test]
pub fn json_malformed_input() {
    match HDDLProgram::from_json("{ not json") {
        Err(ParsingError::JSON(error)) => {
            assert!(!error.to_string().is_empty());
            assert!(error.line > 0);
        }
        other => panic!("expected a Json error, got {:?}", other.map(|_| ())),
    }
}

#[test]
pub fn json_wrong_shape() {
    assert!(matches!(
        HDDLProgram::from_json(r#"{"domain": []}"#),
        Err(ParsingError::JSON(_))
    ));
    let unknown_formula_variant = r#"{
        "domain": {
            "name": "d",
            "requirements": [],
            "predicates": [],
            "compound_tasks": [],
            "methods": [],
            "actions": [{"name": "a", "parameters": [], "preconditions": {"Nand": []}}],
            "functions": []
        }
    }"#;
    assert!(matches!(
        HDDLProgram::from_json(unknown_formula_variant),
        Err(ParsingError::JSON(_))
    ));
}
