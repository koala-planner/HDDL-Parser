use hddl_analyzer::{HDDLProgram, Transformation, Transpiler};
use std::fs;
use std::process::{Command, Output};

fn run_hddl_analyzer(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_hddl_analyzer"))
        .args(args)
        .output()
        .expect("hddl_analyzer subprocess should run")
}

fn output_text(output: &Output) -> (String, String) {
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn cli_missing_input_current_behavior_exits_zero_and_writes_error() {
    let output = run_hddl_analyzer(&["verify", "/tmp/definitely-missing-hddl-parser-input.hddl"]);
    let (_, stderr) = output_text(&output);

    assert_eq!(output.status.code(), Some(0));
    assert!(stderr.contains("[Error]"));
    assert!(stderr.contains("No such file or directory"));
}

#[test]
fn cli_unsupported_extension_current_behavior_exits_zero_and_writes_error() {
    let output = run_hddl_analyzer(&["verify", "Cargo.toml"]);
    let (_, stderr) = output_text(&output);

    assert_eq!(output.status.code(), Some(0));
    assert!(stderr.contains("[Error]"));
    assert!(stderr.contains("unrecognized input extension '.toml'"));
}

#[test]
fn cli_semantic_failure_current_behavior_exits_zero_and_writes_error() {
    let output = run_hddl_analyzer(&["verify", "tests/flawed_domains/undefined-task-domain.hddl"]);
    let (_, stderr) = output_text(&output);

    assert_eq!(output.status.code(), Some(0));
    assert!(stderr.contains("[Error]"));
    assert!(stderr.contains("subtask undefined_task is not defined"));
}

#[test]
fn cli_output_write_failure_current_behavior_exits_zero_and_writes_error() {
    let output = run_hddl_analyzer(&[
        "convert",
        "tests/ipc/Blocksworld-GTOHP/domain.hddl",
        "--to",
        "json",
        "-o",
        "tests",
    ]);
    let (_, stderr) = output_text(&output);

    assert_eq!(output.status.code(), Some(0));
    assert!(stderr.contains("[Error]"));
    assert!(stderr.contains("Is a directory"));
}

#[test]
fn cli_known_good_verification_current_behavior_exits_zero_and_prints_success() {
    let output = run_hddl_analyzer(&["verify", "tests/ipc/Blocksworld-GTOHP/domain.hddl"]);
    let (stdout, stderr) = output_text(&output);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout.contains("[Ok]"));
    assert!(stderr.is_empty());
}

#[test]
fn hddl_program_domain_as_problem_current_behavior_panics() {
    let domain = fs::read("tests/ipc/Blocksworld-GTOHP/domain.hddl").unwrap();

    let result = std::panic::catch_unwind(|| {
        let _ = HDDLProgram::from_hddl(&domain, Some(&domain));
    });

    assert!(result.is_err());
}

#[test]
fn hddl_program_problem_as_domain_current_behavior_panics() {
    let problem = fs::read("tests/ipc/Blocksworld-GTOHP/p01.hddl").unwrap();

    let result = std::panic::catch_unwind(|| {
        let _ = HDDLProgram::from_hddl(&problem, None);
    });

    assert!(result.is_err());
}

#[test]
fn transpiler_domain_as_problem_current_behavior_panics() {
    let domain = fs::read("tests/ipc/Blocksworld-GTOHP/domain.hddl").unwrap();

    let result = std::panic::catch_unwind(|| {
        let _ = Transpiler::from_hddl(&domain, Some(&domain));
    });

    assert!(result.is_err());
}

#[test]
fn transpiler_problem_as_domain_current_behavior_panics() {
    let problem = fs::read("tests/ipc/Blocksworld-GTOHP/p01.hddl").unwrap();

    let result = std::panic::catch_unwind(|| {
        let _ = Transpiler::from_hddl(&problem, None);
    });

    assert!(result.is_err());
}

#[test]
fn remove_equality_constraints_domain_only_current_behavior_panics() {
    let domain = fs::read("tests/ipc/Blocksworld-GTOHP/domain.hddl").unwrap();

    let result = std::panic::catch_unwind(|| {
        let _ = Transpiler::from_hddl(&domain, None)
            .unwrap()
            .transform(Transformation::RemoveEqualityConstraints);
    });

    assert!(result.is_err());
}
