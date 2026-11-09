use super::*;

// parses and verifies a domain, returning the semantic error if any
fn verify(program: &str) -> Result<(), SemanticErrorType> {
    let program = program.as_bytes().to_vec();
    let parser = Parser::new(LexicalAnalyzer::new(&program));
    match parser.parse().unwrap() {
        AbstractSyntaxTree::Domain(d) => {
            DomainSemanticAnalyzer::new(&d).verify_domain().map(|_| ())
        }
        _ => panic!("expected a domain"),
    }
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
