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

    pub fn check_predicate_instantiation(
        &self,
        predicate: &'a Predicate<'a>,
    ) -> Result<(), SemanticErrorType> {
        match &self.symbol_table.predicates.get(predicate) {
            Some(definition) => {
                if definition.variables.len() != predicate.variables.len() {
                    return Err(SemanticErrorType::InconsistentPredicateArity(ArityError {
                        symbol: predicate.name.to_string(),
                        expected_arity: definition.variables.len() as u32,
                        found_arity: predicate.variables.len() as u32,
                        position: predicate.name_pos,
                    }));
                }
                for (expected, found) in definition.variables.iter().zip(predicate.variables.iter())
                {
                    match self.objects.get(found.name) {
                        Some(object_type) => {
                            let is_consistent = self
                                .generic_type_checker
                                .is_var_type_consistent(*object_type, expected.symbol_type);
                            if !is_consistent {
                                return Err(SemanticErrorType::InconsistentPredicateArgType(
                                    TypeError {
                                        expected: expected.symbol_type.map(String::from),
                                        found: found.symbol_type.map(String::from),
                                        var_name: predicate.name.to_string(),
                                        position: found.name_pos,
                                    },
                                ));
                            }
                        }
                        None => match self.symbol_table.constants.get(&found.name) {
                            Some(constant) => {
                                let is_consistent =
                                    self.generic_type_checker.is_var_type_consistent(
                                        constant.symbol_type,
                                        expected.symbol_type,
                                    );
                                if !is_consistent {
                                    return Err(SemanticErrorType::InconsistentPredicateArgType(
                                        TypeError {
                                            expected: expected.symbol_type.map(String::from),
                                            found: constant.symbol_type.map(String::from),
                                            var_name: predicate.name.to_string(),
                                            position: found.name_pos,
                                        },
                                    ));
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
                        },
                    }
                }
                Ok(())
            }
            None => Err(SemanticErrorType::UndefinedPredicate(
                UndefinedSymbolError {
                    symbol: predicate.name.to_string(),
                    position: predicate.name_pos,
                },
            )),
        }
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
