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
