use std::collections::{HashMap, HashSet};

use super::*;

#[derive(Clone)]
pub struct DomainTypeChecker<'a> {
    pub(super) generic_type_checker: TypeChecker<'a>,
}

impl<'a> DomainTypeChecker<'a> {
    pub fn new(types: &Option<Vec<Symbol<'a>>>) -> DomainTypeChecker<'a> {
        DomainTypeChecker {
            generic_type_checker: TypeChecker::new(types),
        }
    }

    pub fn get_type_hierarchy(&'a self) -> GraphMap<&'a str, (), Directed> {
        self.generic_type_checker.type_hierarchy.clone()
    }

    pub fn check_type_declarations(
        &self,
        parameters: &Vec<Symbol<'a>>,
    ) -> Option<SemanticErrorType> {
        self.generic_type_checker
            .check_type_declarations(parameters)
    }

    pub fn verify_type_hierarchy(&self) -> Result<(), SemanticErrorType> {
        self.generic_type_checker.verify_type_hierarchy()
    }

    // every argument of a call must be a parameter or a constant whose type
    // fits the declaration; calls are resolved by name and arity
    // TODO: Add support for "universal qunatification" parameters
    pub fn check_calls<C: Call<'a>>(
        &self,
        calls: &[&C],
        parameters: &Vec<Symbol<'a>>,
        declared_constants: &HashSet<&Symbol<'a>>,
        declared: &[C],
    ) -> Result<(), SemanticErrorType> {
        // Assert all types are declared
        if let Some(undeclared_type) = self
            .generic_type_checker
            .check_type_declarations(parameters)
        {
            return Err(undeclared_type);
        }
        // Store parameter types
        let par_types: HashMap<&str, Option<&str>> =
            HashMap::from_iter(parameters.iter().map(|par| (par.name, par.symbol_type)));
        for call in calls {
            let declaration = declared.iter().find(|declaration| {
                declaration.name() == call.name()
                    && declaration.symbols().len() == call.symbols().len()
            });
            let Some(declaration) = declaration else {
                return Err(C::undefined(UndefinedSymbolError {
                    symbol: call.name().to_string(),
                    position: call.name_pos(),
                }));
            };
            // resolve every argument to its type before comparing any
            let mut arguments = vec![];
            for argument in call.symbols() {
                let found = match par_types.get(argument.name) {
                    Some(par_type) => *par_type,
                    None => match declared_constants.get(argument) {
                        Some(constant) => constant.symbol_type,
                        None => {
                            return Err(SemanticErrorType::UndefinedParameter(
                                UndefinedSymbolError {
                                    symbol: argument.name.to_string(),
                                    position: argument.name_pos,
                                },
                            ));
                        }
                    },
                };
                arguments.push((argument, found));
            }
            for ((argument, found), parameter) in arguments.into_iter().zip(declaration.symbols()) {
                let expected = parameter.symbol_type;
                if !self
                    .generic_type_checker
                    .is_var_type_consistent(found, expected)
                {
                    return Err(C::inconsistent_arg_type(TypeError {
                        expected: expected.map(|inner| inner.to_string()),
                        found: found.map(|inner| inner.to_string()),
                        var_name: argument.name.to_string(),
                        position: argument.name_pos,
                    }));
                }
            }
        }
        Ok(())
    }

    pub fn is_task_consistent(
        &self,
        task: &Symbol<'a>,
        task_terms: &Vec<Symbol<'a>>,
        parameters: &Vec<Symbol<'a>>,
        declared_constants: &HashSet<&Symbol<'a>>,
        declared_tasks: &HashSet<&Task<'a>>,
        declared_actions: &HashSet<&Action<'a>>,
    ) -> Result<(), SemanticErrorType> {
        // Store parameter types as a mapping from name to (type and position)
        let par_types: HashMap<&str, &Symbol> =
            HashMap::from_iter(parameters.iter().map(|par| (par.name, par)));
        let mut found_types = vec![];
        for term in task_terms.iter() {
            match par_types.get(term.name) {
                Some(par_definition) => {
                    found_types.push((term, par_definition));
                }
                None => match declared_constants.get(term) {
                    Some(constant) => found_types.push((term, constant)),
                    None => {
                        return Err(SemanticErrorType::UndefinedParameter(
                            UndefinedSymbolError {
                                symbol: term.name.to_string(),
                                position: term.name_pos,
                            },
                        ));
                    }
                },
            }
        }
        match declared_actions.iter().find(|x| x.name == task.name) {
            Some(definition) => {
                let expected_types: Vec<Option<&str>> = definition
                    .parameters
                    .iter()
                    .map(|x| x.symbol_type)
                    .collect();
                if task_terms.len() != expected_types.len() {
                    return Err(SemanticErrorType::InconsistentTaskArity(ArityError {
                        symbol: task.name.to_string(),
                        expected_arity: expected_types.len() as u32,
                        found_arity: task_terms.len() as u32,
                        position: task.name_pos,
                    }));
                }
                for ((term, parameter), expected_type) in
                    found_types.iter().zip(expected_types.iter())
                {
                    if !self
                        .generic_type_checker
                        .is_var_type_consistent(parameter.symbol_type, *expected_type)
                    {
                        return Err(SemanticErrorType::InconsistentTaskArgType(TypeError {
                            expected: expected_type.map(|inner| inner.to_string()),
                            found: parameter.symbol_type.map(|inner| inner.to_string()),
                            var_name: term.name.to_string(),
                            position: term.name_pos,
                        }));
                    }
                }
                Ok(())
            }
            None => match declared_tasks.iter().find(|x| x.name == task.name) {
                Some(definition) => {
                    let expected: Vec<Option<&str>> = definition
                        .parameters
                        .iter()
                        .map(|x| x.symbol_type)
                        .collect();
                    if found_types.len() != expected.len() {
                        return Err(SemanticErrorType::InconsistentTaskArity(ArityError {
                            symbol: task.name.to_string(),
                            expected_arity: expected.len() as u32,
                            found_arity: found_types.len() as u32,
                            position: task.name_pos,
                        }));
                    }
                    for ((term, parameter), expected_type) in
                        found_types.iter().zip(expected.iter())
                    {
                        if !self
                            .generic_type_checker
                            .is_var_type_consistent(parameter.symbol_type, *expected_type)
                        {
                            return Err(SemanticErrorType::InconsistentTaskArgType(TypeError {
                                expected: expected_type.map(|inner| inner.to_string()),
                                found: parameter.symbol_type.map(|inner| inner.to_string()),
                                var_name: term.name.to_string(),
                                position: term.name_pos,
                            }));
                        }
                    }
                    Ok(())
                }
                None => Err(SemanticErrorType::UndefinedSubtask(UndefinedSymbolError {
                    symbol: task.name.to_string(),
                    position: task.name_pos,
                })),
            },
        }
    }
}
