use super::*;

// every predicate or function must be declared, with the arity it is used at
pub fn check_declarations<'a, C: Call<'a>>(
    calls: &[&C],
    declared: &[C],
) -> Result<(), SemanticErrorType> {
    for call in calls {
        let same_name: Vec<&C> = declared
            .iter()
            .filter(|declaration| declaration.name() == call.name())
            .collect();
        let Some(first) = same_name.first() else {
            return Err(C::undefined(UndefinedSymbolError {
                symbol: call.name().to_string(),
                position: call.name_pos(),
            }));
        };
        let arity = call.symbols().len();
        if !same_name
            .iter()
            .any(|declaration| declaration.symbols().len() == arity)
        {
            return Err(C::inconsistent_arity(ArityError {
                symbol: call.name().to_string(),
                expected_arity: first.symbols().len() as u32,
                found_arity: arity as u32,
                position: call.name_pos(),
            }));
        }
    }
    Ok(())
}
