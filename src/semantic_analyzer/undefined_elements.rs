use super::*;

pub fn check_predicate_declarations<'a>(
    formula: &Formula<'a>,
    declared_predicates: &Vec<Predicate<'a>>,
) -> Result<(), SemanticErrorType> {
    match &*formula {
        Formula::Empty => {}
        Formula::Atom(predicate) => {
            for declared_predicate in declared_predicates {
                // Assert same name
                if predicate.name == declared_predicate.name {
                    // Assert same arity
                    if predicate.variables.len() == declared_predicate.variables.len() {
                        return Ok(());
                    } else {
                        return Err(SemanticErrorType::InconsistentPredicateArity(ArityError {
                            symbol: predicate.name.to_string(),
                            expected_arity: declared_predicate.variables.len() as u32,
                            found_arity: predicate.variables.len() as u32,
                            position: predicate.name_pos
                        }));
                    }
                }
            }
            return Err(SemanticErrorType::UndefinedPredicate(
                UndefinedSymbolError {
                    symbol: predicate.name.to_string(),
                    position: predicate.name_pos,
                },
            ));
        }
        Formula::Not(new_formula) => {
            return check_predicate_declarations(&*new_formula, declared_predicates);
        }
        Formula::And(new_formula) | Formula::Or(new_formula) | Formula::Xor(new_formula) => {
            for f in new_formula {
                check_predicate_declarations(&*f, declared_predicates)?;
            }
        }
        Formula::ForAll(_, new_formula) => {
            return check_predicate_declarations(&*new_formula, declared_predicates);
        }
        Formula::Equals(_, _) => {}
        Formula::Weighted(_, new_formula) => {
            return check_predicate_declarations(&*new_formula, declared_predicates);
        }
        // TODO: add support for imply, and exists
        _ => {
            panic!()
        }
    }
    return Ok(());
}
