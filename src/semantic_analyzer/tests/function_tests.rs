use super::*;

// parses and verifies a domain, and a problem against it when given,
// returning the semantic error if any
fn verify(domain_text: &str, problem_text: Option<&str>) -> Result<(), SemanticErrorType> {
    let domain_text = domain_text.as_bytes().to_vec();
    let AbstractSyntaxTree::Domain(domain) = Parser::new(LexicalAnalyzer::new(&domain_text))
        .parse()
        .unwrap()
    else {
        panic!("expected a domain")
    };
    let analyzer = DomainSemanticAnalyzer::new(&domain);
    let symbols = analyzer.verify_domain()?;
    let Some(problem_text) = problem_text else {
        return Ok(());
    };
    let problem_text = problem_text.as_bytes().to_vec();
    let AbstractSyntaxTree::Problem(problem) = Parser::new(LexicalAnalyzer::new(&problem_text))
        .parse()
        .unwrap()
    else {
        panic!("expected a problem")
    };
    ProblemSemanticAnalyzer::new(&problem, symbols)
        .verify_problem()
        .map(|_| ())
}

#[test]
pub fn numeric_effects_verify_test() {
    let result = verify(
        "(define (domain transport)
            (:types truck - vehicle vehicle location)
            (:constants depot - location)
            (:predicates (at ?v - vehicle ?l - location))
            (:functions
                (road-length ?l1 ?l2 - location) - number
                (visits ?l - location)
                (fuel ?v - vehicle)
                (total-cost)
            )
            (:action drive
             :parameters (?t - truck ?l1 ?l2 - location)
             :precondition (at ?t ?l1)
             :effect (and
                (at ?t ?l2)
                (increase (total-cost) (road-length ?l1 ?l2))
                (decrease (fuel ?t) (road-length ?l1 depot))
                (forall (?l - location) (when (at ?t ?l) (increase (visits ?l) 1)))
             )
            )
        )",
        None,
    );
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
pub fn undefined_function_test() {
    let error = verify(
        "(define (domain transport)
            (:functions (total-cost))
            (:action drive
             :parameters (?l1 ?l2)
             :effect (increase (total-cost) (road-length ?l1 ?l2))
            )
        )",
        None,
    )
    .unwrap_err();
    match error {
        SemanticErrorType::UndefinedFunction(undefined) => {
            assert_eq!(undefined.symbol, "road-length");
            assert_eq!(undefined.position.line, 5);
        }
        other => panic!("{:?}", other),
    }
}

#[test]
pub fn inconsistent_function_arity_test() {
    let error = verify(
        "(define (domain transport)
            (:functions (road-length ?l1 ?l2))
            (:action drive
             :parameters (?l1 ?l2)
             :effect (and
                (increase (road-length ?l1 ?l2) 1)
                (increase (road-length ?l1) 1))
            )
        )",
        None,
    )
    .unwrap_err();
    match error {
        SemanticErrorType::InconsistentFunctionArity(arity) => {
            assert_eq!(arity.symbol, "road-length");
            assert_eq!(arity.expected_arity, 2);
            assert_eq!(arity.found_arity, 1);
            assert_eq!(arity.position.line, 7);
        }
        other => panic!("{:?}", other),
    }
}

#[test]
pub fn inconsistent_function_arg_type_test() {
    let error = verify(
        "(define (domain transport)
            (:types vehicle location)
            (:functions (fuel ?v - vehicle))
            (:action drive
             :parameters (?v - vehicle ?l - location)
             :effect (assign (fuel ?l) 0)
            )
        )",
        None,
    )
    .unwrap_err();
    match error {
        SemanticErrorType::InconsistentFunctionArgType(type_error) => {
            assert_eq!(type_error.var_name, "?l");
            assert_eq!(type_error.expected.as_deref(), Some("vehicle"));
            assert_eq!(type_error.found.as_deref(), Some("location"));
            assert_eq!(type_error.position.line, 6);
        }
        other => panic!("{:?}", other),
    }
}

#[test]
pub fn undefined_fluent_argument_test() {
    let error = verify(
        "(define (domain transport)
            (:functions (fuel ?v) (total-cost))
            (:action drive
             :parameters (?v)
             :effect (increase (total-cost) (fuel ?w))
            )
        )",
        None,
    )
    .unwrap_err();
    match error {
        SemanticErrorType::UndefinedParameter(undefined) => {
            assert_eq!(undefined.symbol, "?w");
            assert_eq!(undefined.position.line, 5);
        }
        other => panic!("{:?}", other),
    }
}

#[test]
pub fn duplicate_function_test() {
    // like predicates, functions are identified by name and arity
    assert!(
        verify(
            "(define (domain transport)
                (:functions (fuel ?v) (fuel ?v ?l))
            )",
            None,
        )
        .is_ok()
    );
    let error = verify(
        "(define (domain transport)
            (:functions
                (fuel ?v)
                (total-cost)
                (fuel ?x)
            )
        )",
        None,
    )
    .unwrap_err();
    match error {
        SemanticErrorType::DuplicateFunctionDeclaration(duplicate) => {
            assert_eq!(duplicate.symbol, "fuel");
            assert_eq!(duplicate.first_pos.line, 3);
            assert_eq!(duplicate.second_pos.line, 5);
        }
        other => panic!("{:?}", other),
    }
}

#[test]
pub fn undefined_function_parameter_type_test() {
    let error = verify(
        "(define (domain transport)
            (:types vehicle)
            (:functions (fuel ?v - truck))
        )",
        None,
    )
    .unwrap_err();
    match error {
        SemanticErrorType::UndefinedType(undefined) => {
            assert_eq!(undefined.symbol, "truck");
            assert_eq!(undefined.position.line, 3);
        }
        other => panic!("{:?}", other),
    }
}

const NUMERIC_DOMAIN: &str = "(define (domain transport)
    (:types vehicle location)
    (:constants depot - location)
    (:predicates (at ?v - vehicle ?l - location))
    (:functions (road-length ?l1 ?l2 - location) (total-cost))
)";

// a problem over NUMERIC_DOMAIN with the given :init entries and trailing blocks
fn numeric_problem(init: &str, trailing: &str) -> String {
    format!(
        "(define (problem p) (:domain transport)
    (:objects t - vehicle a - location)
    (:init
        (at t a)
        {init}
    )
    {trailing}
)"
    )
}

#[test]
pub fn problem_numerics_verify_test() {
    let problem = numeric_problem(
        "(= (total-cost) 0) (= (road-length a depot) 5)",
        "(:goal (at t depot)) (:metric minimize (+ (total-cost) (* 2 (road-length a a))))",
    );
    let result = verify(NUMERIC_DOMAIN, Some(&problem));
    assert!(result.is_ok(), "{:?}", result);
    let problem = numeric_problem("", "(:metric maximize 5)");
    assert!(verify(NUMERIC_DOMAIN, Some(&problem)).is_ok());
}

#[test]
pub fn undefined_fluent_in_problem_test() {
    for (init, trailing, symbol) in [
        ("(= (fuel t) 3)", "", "fuel"),
        ("", "(:metric minimize (total-time))", "total-time"),
        ("", "(:metric minimize total-time)", "total-time"),
        (
            "",
            "(:metric minimize (+ (total-cost) (* 2 (fuel t))))",
            "fuel",
        ),
    ] {
        let problem = numeric_problem(init, trailing);
        match verify(NUMERIC_DOMAIN, Some(&problem)) {
            Err(SemanticErrorType::UndefinedFunction(undefined)) => {
                assert_eq!(undefined.symbol, symbol);
            }
            other => panic!("{init} {trailing}: {:?}", other),
        }
    }
    // the same check covers sums inside domain effects
    let error = verify(
        "(define (domain transport)
            (:functions (total-cost))
            (:action drive
             :parameters ()
             :effect (increase (total-cost) (+ (undeclared) 1))
            )
        )",
        None,
    )
    .unwrap_err();
    match error {
        SemanticErrorType::UndefinedFunction(undefined) => {
            assert_eq!(undefined.symbol, "undeclared");
            assert_eq!(undefined.position.line, 5);
        }
        other => panic!("{:?}", other),
    }
}

#[test]
pub fn inconsistent_fluent_arity_in_problem_test() {
    let problem = numeric_problem("(= (road-length a) 3)", "");
    match verify(NUMERIC_DOMAIN, Some(&problem)) {
        Err(SemanticErrorType::InconsistentFunctionArity(arity)) => {
            assert_eq!(arity.symbol, "road-length");
            assert_eq!(arity.expected_arity, 2);
            assert_eq!(arity.found_arity, 1);
            assert_eq!(arity.position.line, 5);
        }
        other => panic!("{:?}", other),
    }
}

#[test]
pub fn undefined_object_in_fluent_test() {
    let problem = numeric_problem("(= (road-length a c) 3)", "");
    match verify(NUMERIC_DOMAIN, Some(&problem)) {
        Err(SemanticErrorType::UndefinedObject(undefined)) => {
            assert_eq!(undefined.symbol, "c");
            assert_eq!(undefined.position.line, 5);
        }
        other => panic!("{:?}", other),
    }
}

#[test]
pub fn inconsistent_fluent_arg_type_in_problem_test() {
    for (init, trailing) in [
        ("(= (road-length t a) 3)", ""),
        ("", "(:metric minimize (road-length a t))"),
    ] {
        let problem = numeric_problem(init, trailing);
        match verify(NUMERIC_DOMAIN, Some(&problem)) {
            Err(SemanticErrorType::InconsistentFunctionArgType(type_error)) => {
                assert_eq!(type_error.var_name, "t");
                assert_eq!(type_error.expected.as_deref(), Some("location"));
                assert_eq!(type_error.found.as_deref(), Some("vehicle"));
            }
            other => panic!("{init} {trailing}: {:?}", other),
        }
    }
}
