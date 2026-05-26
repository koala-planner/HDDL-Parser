extern crate hddl_analyzer;

use hddl_analyzer::{HDDLAnalyzer, SemanticErrorType};
use std::fs;

#[test]
// #[ignore = "takes too long to run"]
pub fn ipc_validation_test() {
    for folder in fs::read_dir("tests/ipc").unwrap() {
        let path = folder.as_ref().unwrap().path();
        let domain_path = fs::read_dir(path.clone()).unwrap().filter(|x| {
            x.as_ref().unwrap().file_name() == "domain.hddl"
        }).next().as_ref().unwrap().as_ref().unwrap().path();
        let domain = fs::read(&domain_path).unwrap();
        for file in fs::read_dir(path).unwrap() {
            if file.as_ref().unwrap().file_name() == "domain.hddl" {
                continue;
            } else {
                let problem_path = file.as_ref().unwrap().path();
                let problem = fs::read(&problem_path).unwrap();
                match HDDLAnalyzer::verify(&domain, Some(&problem)) {
                    Err(token) => {
                        let error = format!("Domain: {:?} \nProblem:{:?}\nError: {:?}", domain_path, problem_path, token);
                        panic!("{}",  error)
                    }
                    Ok(_) => {}
                }
            }
        }
    }    
}
