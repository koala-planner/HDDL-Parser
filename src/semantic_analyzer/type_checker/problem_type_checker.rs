use std::collections::HashMap;

use super::*;

pub struct ProblemTypeChecker<'a> {
    generic_type_checker: TypeChecker<'a>,
    pub symbol_table: SymbolTable<'a>,
    objects: HashMap<&'a str, Option<&'a str>>,
}

impl<'a> ProblemTypeChecker<'a> {
    pub fn new(
        symbol_table: SymbolTable<'a>,
        problem: &'a ProblemAST<'a>,
    ) -> ProblemTypeChecker<'a> {
        let mut objects = HashMap::new();
        for object in problem.objects.iter() {
            objects.insert(object.name, object.symbol_type);
        }
        ProblemTypeChecker {
            generic_type_checker: TypeChecker {
                type_hierarchy: symbol_table.type_hierarchy.clone(),
            },
            symbol_table,
            objects,
        }
    }
    pub fn check_type_declarations(
        &self,
        parameters: &Vec<Symbol<'a>>,
    ) -> Option<SemanticErrorType> {
        self.generic_type_checker
            .check_type_declarations(parameters)
    }

    // a ground call of the problem: declared at this arity, with every
    // argument an object or constant whose type fits the declaration
    pub fn check_instantiation<C: Call<'a>>(
        &self,
        call: &C,
        declared: &[C],
    ) -> Result<(), SemanticErrorType> {
        let declaration = find_declaration(call, declared)?;
        for (argument, parameter) in call.symbols().iter().zip(declaration.symbols()) {
            let found = match self.objects.get(argument.name) {
                Some(object_type) => *object_type,
                None => match self.symbol_table.constants.get(&argument.name) {
                    Some(constant) => constant.symbol_type,
                    None => {
                        return Err(SemanticErrorType::UndefinedObject(UndefinedSymbolError {
                            symbol: argument.name.to_string(),
                            position: argument.name_pos,
                        }));
                    }
                },
            };
            let expected = parameter.symbol_type;
            if !self
                .generic_type_checker
                .is_var_type_consistent(found, expected)
            {
                return Err(C::inconsistent_arg_type(TypeError {
                    expected: expected.map(String::from),
                    found: found.map(String::from),
                    var_name: argument.name.to_string(),
                    position: argument.name_pos,
                }));
            }
        }
        Ok(())
    }

    // TODO: Refactor the redundancies out
    pub fn check_subtask_instantiation(
        &self,
        subtask: &'a Subtask<'a>,
        parameters: &Option<Vec<Symbol<'a>>>,
    ) -> Result<(), SemanticErrorType> {
        if self.symbol_table.actions.contains(&subtask.task.name) {
            let action = self.symbol_table.actions.get(&subtask.task.name).unwrap();
            if action.parameters.len() != subtask.terms.len() {
                return Err(SemanticErrorType::InconsistentTaskArity(ArityError {
                    symbol: subtask.task.name.to_string(),
                    expected_arity: action.parameters.len() as u32,
                    found_arity: subtask.terms.len() as u32,
                    position: subtask.task.name_pos,
                }));
            }
            for (expected, found) in action.parameters.iter().zip(subtask.terms.iter()) {
                match self.objects.get(found.name) {
                    Some(object_type) => {
                        let is_consistent = self
                            .generic_type_checker
                            .is_var_type_consistent(*object_type, expected.symbol_type);
                        if !is_consistent {
                            return Err(SemanticErrorType::InconsistentTaskArgType(TypeError {
                                expected: expected.symbol_type.map(String::from),
                                found: object_type.map(String::from),
                                var_name: subtask.task.name.to_string(),
                                position: found.name_pos,
                            }));
                        }
                    }
                    None => {
                        let mut undefined = false;
                        match parameters {
                            Some(params) => match params.iter().find(|x| x.name == found.name) {
                                Some(param) => {
                                    let is_consistent =
                                        self.generic_type_checker.is_var_type_consistent(
                                            param.symbol_type,
                                            expected.symbol_type,
                                        );
                                    if !is_consistent {
                                        return Err(SemanticErrorType::InconsistentTaskArgType(
                                            TypeError {
                                                expected: expected.symbol_type.map(String::from),
                                                found: param.symbol_type.map(String::from),
                                                var_name: subtask.task.name.to_string(),
                                                position: found.name_pos,
                                            },
                                        ));
                                    }
                                }
                                None => {
                                    undefined = true;
                                }
                            },
                            None => {
                                undefined = true;
                            }
                        }
                        if undefined {
                            match self.symbol_table.constants.get(found) {
                                Some(constant) => {
                                    let is_consistent =
                                        self.generic_type_checker.is_var_type_consistent(
                                            constant.symbol_type,
                                            expected.symbol_type,
                                        );
                                    if !is_consistent {
                                        return Err(
                                            SemanticErrorType::InconsistentPredicateArgType(
                                                TypeError {
                                                    expected: expected
                                                        .symbol_type
                                                        .map(String::from),
                                                    found: constant.symbol_type.map(String::from),
                                                    var_name: action.name.to_string(),
                                                    position: found.name_pos,
                                                },
                                            ),
                                        );
                                    }
                                }
                                None => {
                                    return Err(SemanticErrorType::UndefinedObject(
                                        UndefinedSymbolError {
                                            symbol: found.name.to_string(),
                                            position: found.name_pos,
                                        },
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            Ok(())
        } else if self.symbol_table.tasks.contains(subtask.task.name) {
            let task = self.symbol_table.tasks.get(&subtask.task.name).unwrap();
            if task.parameters.len() != subtask.terms.len() {
                return Err(SemanticErrorType::InconsistentTaskArity(ArityError {
                    symbol: subtask.task.name.to_string(),
                    expected_arity: task.parameters.len() as u32,
                    found_arity: subtask.terms.len() as u32,
                    position: subtask.task.name_pos,
                }));
            }
            for (expected, found) in task.parameters.iter().zip(subtask.terms.iter()) {
                match self.objects.get(found.name) {
                    Some(object_type) => {
                        let is_consistent = self
                            .generic_type_checker
                            .is_var_type_consistent(*object_type, expected.symbol_type);
                        if !is_consistent {
                            return Err(SemanticErrorType::InconsistentTaskArgType(TypeError {
                                expected: expected.symbol_type.map(String::from),
                                found: object_type.map(String::from),
                                var_name: subtask.task.name.to_string(),
                                position: found.name_pos,
                            }));
                        }
                    }
                    None => {
                        let mut undefined = false;
                        match parameters {
                            Some(params) => match params.iter().find(|x| x.name == found.name) {
                                Some(definition) => {
                                    let is_consistent =
                                        self.generic_type_checker.is_var_type_consistent(
                                            definition.symbol_type,
                                            expected.symbol_type,
                                        );
                                    if !is_consistent {
                                        return Err(SemanticErrorType::InconsistentTaskArgType(
                                            TypeError {
                                                expected: expected.symbol_type.map(String::from),
                                                found: definition.symbol_type.map(String::from),
                                                var_name: subtask.task.name.to_string(),
                                                position: found.name_pos,
                                            },
                                        ));
                                    }
                                }
                                None => {
                                    undefined = true;
                                }
                            },
                            None => {
                                undefined = true;
                            }
                        }
                        if undefined {
                            match self.symbol_table.constants.get(found) {
                                Some(constant) => {
                                    let is_consistent =
                                        self.generic_type_checker.is_var_type_consistent(
                                            constant.symbol_type,
                                            expected.symbol_type,
                                        );
                                    if !is_consistent {
                                        return Err(
                                            SemanticErrorType::InconsistentPredicateArgType(
                                                TypeError {
                                                    expected: expected
                                                        .symbol_type
                                                        .map(String::from),
                                                    found: constant.symbol_type.map(String::from),
                                                    var_name: task.name.to_string(),
                                                    position: found.name_pos,
                                                },
                                            ),
                                        );
                                    }
                                }
                                None => {
                                    return Err(SemanticErrorType::UndefinedObject(
                                        UndefinedSymbolError {
                                            symbol: found.name.to_string(),
                                            position: found.name_pos,
                                        },
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            Ok(())
        } else {
            Err(SemanticErrorType::UndefinedSubtask(UndefinedSymbolError {
                symbol: subtask.task.name.to_string(),
                position: subtask.task.name_pos,
            }))
        }
    }
}
