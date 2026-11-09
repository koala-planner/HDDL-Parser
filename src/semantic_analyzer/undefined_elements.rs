use super::*;

// every predicate must be declared, with the arity it is used at
pub fn check_predicate_declarations<'a>(
    predicates: &[&Predicate<'a>],
    declared_predicates: &[Predicate<'a>],
) -> Result<(), SemanticErrorType> {
    for predicate in predicates {
        let declaration = declared_predicates
            .iter()
            .find(|declared| declared.name == predicate.name);
        match declaration {
            None => {
                return Err(SemanticErrorType::UndefinedPredicate(
                    UndefinedSymbolError {
                        symbol: predicate.name.to_string(),
                        position: predicate.name_pos,
                    },
                ));
            }
            Some(declared) if declared.variables.len() != predicate.variables.len() => {
                return Err(SemanticErrorType::InconsistentPredicateArity(ArityError {
                    symbol: predicate.name.to_string(),
                    expected_arity: declared.variables.len() as u32,
                    found_arity: predicate.variables.len() as u32,
                    position: predicate.name_pos,
                }));
            }
            Some(_) => {}
        }
    }
    Ok(())
}
