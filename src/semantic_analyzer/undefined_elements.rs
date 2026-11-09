use super::*;

// the declaration a call refers to: one with its name, at the arity it is used at
pub fn find_declaration<'a, 'd, C: Call<'a>>(
    call: &C,
    declared: &'d [C],
) -> Result<&'d C, SemanticErrorType> {
    let same_name: Vec<&'d C> = declared
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
    match same_name
        .iter()
        .find(|declaration| declaration.symbols().len() == arity)
    {
        Some(declaration) => Ok(declaration),
        None => Err(C::inconsistent_arity(ArityError {
            symbol: call.name().to_string(),
            expected_arity: first.symbols().len() as u32,
            found_arity: arity as u32,
            position: call.name_pos(),
        })),
    }
}

// every predicate or function must be declared, with the arity it is used at
pub fn check_declarations<'a, C: Call<'a>>(
    calls: &[&C],
    declared: &[C],
) -> Result<(), SemanticErrorType> {
    for call in calls {
        find_declaration(*call, declared)?;
    }
    Ok(())
}
