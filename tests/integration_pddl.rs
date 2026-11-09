extern crate hddl_analyzer;

use hddl_analyzer::{HDDLProgram, Transpiler};
use std::fs;

// the IPC 2011 transport domain: action costs as numeric effects over
// `(total-cost)`, and `- number` typed function declarations
#[test]
pub fn transport_domain_verifies_and_round_trips() {
    let domain = fs::read("tests/pddl/transport-sequential-optimal/domain.pddl").unwrap();
    let program = HDDLProgram::from_hddl(&domain, None).unwrap();
    program.verify().unwrap();

    // the printed HDDL parses and verifies again, with the numeric effects intact
    let transpiler = Transpiler::new(program);
    let (hddl, _) = transpiler.to_hddl();
    assert!(hddl.contains("(increase (total-cost) (road-length ?l1 ?l2))"));
    assert!(hddl.contains("(increase (total-cost) 1)"));
    let reprinted = hddl.clone().into_bytes();
    HDDLProgram::from_hddl(&reprinted, None)
        .unwrap()
        .verify()
        .unwrap();

    // the JSON export is stable under re-import
    let exported = transpiler.to_json();
    let reimported = HDDLProgram::from_json(&exported).unwrap();
    assert_eq!(exported, Transpiler::new(reimported).to_json());
}

// an IPC 2011 transport instance: fluent values in :init and a :metric after the goal
#[test]
pub fn transport_instance_verifies_and_round_trips() {
    let domain = fs::read("tests/pddl/transport-sequential-optimal/domain.pddl").unwrap();
    let problem = fs::read("tests/pddl/transport-sequential-optimal/instance-1.pddl").unwrap();
    let program = HDDLProgram::from_hddl(&domain, Some(&problem)).unwrap();
    program.verify().unwrap();

    let transpiler = Transpiler::new(program);
    let (domain_hddl, problem_hddl) = transpiler.to_hddl();
    let problem_hddl = problem_hddl.unwrap();
    assert!(problem_hddl.contains("(= (total-cost) 0)"));
    assert!(problem_hddl.contains("(= (road-length city-1-loc-3 city-1-loc-1) 22)"));
    assert!(problem_hddl.contains("(:metric minimize (total-cost))"));

    // the printed pair parses, verifies, and prints identically
    let domain_bytes = domain_hddl.clone().into_bytes();
    let problem_bytes = problem_hddl.clone().into_bytes();
    let reparsed = HDDLProgram::from_hddl(&domain_bytes, Some(&problem_bytes)).unwrap();
    reparsed.verify().unwrap();
    assert_eq!(
        Transpiler::new(reparsed).to_hddl(),
        (domain_hddl, Some(problem_hddl))
    );

    // the JSON export is stable under re-import and still verifies
    let exported = transpiler.to_json();
    let reimported = HDDLProgram::from_json(&exported).unwrap();
    reimported.verify().unwrap();
    assert_eq!(exported, Transpiler::new(reimported).to_json());
}
