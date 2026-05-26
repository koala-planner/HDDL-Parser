use std::collections::{HashMap, HashSet};

use super::*;
use crate::lexical_analyzer::RequirementType;

pub struct DomainSemanticAnalyzer<'a> {
    domain: &'a DomainAST<'a>,
    pub type_checker: DomainTypeChecker<'a>,
}

impl<'a> DomainSemanticAnalyzer<'a> {
    pub fn new(domain: &'a DomainAST<'a>) -> DomainSemanticAnalyzer<'a> {
        DomainSemanticAnalyzer {
            domain,
            type_checker: DomainTypeChecker::new(&domain.types),
        }
    }

    pub fn verify_domain(&'a self) -> Result<SymbolTable<'a>, SemanticErrorType> {
        // Assert there are no duplicate requirements
        if let Some(duplicate) =
            DomainSemanticAnalyzer::check_duplicate_requirements(&self.domain.requirements)
        {
            return Err(duplicate);
        }
        // Assert type hierarchy is acyclic
        let _ = self.type_checker.verify_type_hierarchy()?;
        let mut warnings = vec![];
        // Domain declarations
        let declared_predicates = self.verify_predicates()?;
        let declared_tasks = self.verify_compound_tasks()?;
        let mut declared_constants = HashSet::new();
        match &self.domain.constants {
            Some(constants) => {
                for c in constants {
                    declared_constants.insert(c);
                }
            }
            None => {}
        }

        // assert actions are correct
        let mut declared_actions = HashSet::new();
        let mut action_positions = HashMap::new();
        for action in self.domain.actions.iter() {
            if !declared_actions.insert(action) {
                return Err(SemanticErrorType::DuplicateActionDeclaration(
                    DuplicateError {
                        symbol: action.name.to_string(),
                        first_pos: *action_positions.get(action.name).unwrap(),
                        second_pos: action.name_pos,
                    },
                ));
            } else {
                action_positions.insert(action.name, action.name_pos);
            }
            // assert there is no duplicate paramter
            let mut declared_params = HashSet::new();
            let mut param_positions = HashMap::new();
            for param in action.parameters.iter() {
                if !declared_params.insert(param) {
                    return Err(SemanticErrorType::DuplicateParameterDeclaration(
                        DuplicateError {
                            symbol: param.name.to_string(),
                            first_pos: *param_positions.get(param.name).unwrap(),
                            second_pos: param.name_pos,
                        },
                    ));
                } else {
                    param_positions.insert(param.name, param.name_pos);
                }
            }
            // assert precondition predicates are declared
            match &action.preconditions {
                Some(precondition) => {
                    check_predicate_declarations(precondition, &self.domain.predicates)?;
                    let precond_predicates = precondition.get_propositional_predicates();
                    self.type_checker.check_formula(
                        &precond_predicates,
                        &action.parameters,
                        &declared_constants,
                        &declared_predicates,
                    )?;
                    if !precondition.is_sat() {
                        return Err(SemanticErrorType::ComplementaryActionPrecondition(
                            action.name_pos
                        ));
                    }
                }
                _ => {}
            }
            // assert effect predicates are declared
            match &action.effects {
                Some(effect) => {
                    check_predicate_declarations(effect, &self.domain.predicates)?;
                    let eff_predicates = effect.get_propositional_predicates();
                    self.type_checker.check_formula(
                        &eff_predicates,
                        &action.parameters,
                        &declared_constants,
                        &declared_predicates,
                    )?;
                }
                _ => {}
            }
        }

        // assert methods are correct
        let mut declared_methods = HashSet::new();
        let mut method_positions = HashMap::new();
        for method in self.domain.methods.iter() {
            if !declared_methods.insert(&method.name) {
                return Err(SemanticErrorType::DuplicateMethodDeclaration(
                    DuplicateError {
                        symbol: method.name.name.to_string(),
                        first_pos: *method_positions.get(&method.name).unwrap(),
                        second_pos: method.name.name_pos,
                    },
                ));
            } else {
                method_positions.insert(&method.name, method.name.name_pos);
            }
            // assert there is no duplicate paramter
            let mut declared_params = HashSet::new();
            let mut param_positions = HashMap::new();
            for param in method.params.iter() {
                if !declared_params.insert(param) {
                    return Err(SemanticErrorType::DuplicateParameterDeclaration(
                        DuplicateError {
                            symbol: param.name.to_string(),
                            first_pos: *param_positions.get(param.name).unwrap(),
                            second_pos: param.name_pos,
                        },
                    ));
                } else {
                    param_positions.insert(param.name, param.name_pos);
                }
            }
            // Assert preconditions are valid
            match &method.precondition {
                Some(precondition) => {
                    check_predicate_declarations(precondition, &self.domain.predicates)?;
                    let precond_predicates = precondition.get_propositional_predicates();
                    self.type_checker.check_formula(
                        &precond_predicates,
                        &method.params,
                        &declared_constants,
                        &declared_predicates,
                    )?;
                    if !precondition.is_sat() {
                        return Err(SemanticErrorType::ComplementaryMethodPrecondition(
                            method.name.name_pos
                        ));
                    }
                }
                _ => {}
            }
            // Assert task is defined
            if !declared_tasks.contains(method.task.name) {
                return Err(SemanticErrorType::UndefinedTask(UndefinedSymbolError {
                    symbol: method.task.name.to_string(),
                    position: method.task.name_pos,
                }));
            } else {
                // Assert task arity is consistent
                for declared_compound_task in self.domain.compound_tasks.iter() {
                    if method.task.name == declared_compound_task.name {
                        if method.task_terms.len() != declared_compound_task.parameters.len() {
                            return Err(SemanticErrorType::InconsistentTaskArity(ArityError {
                                symbol: method.task.name.to_string(),
                                expected_arity: method.task_terms.len() as u32,
                                found_arity: declared_compound_task.parameters.len() as u32,
                                position: method.task.name_pos,
                            }));
                        } else {
                            break;
                        }
                    }
                }
            }

            // Assert task type is consistent
            let _ = self.type_checker.is_task_consistent(
                &method.task,
                &method.task_terms,
                &method.params,
                &declared_constants,
                &declared_tasks,
                &HashSet::new(),
            )?;

            // Assert subtask types are consistent
            for subtask in method.tn.subtasks.iter() {
                let _ = self.type_checker.is_task_consistent(
                    &subtask.task,
                    &subtask.terms,
                    &method.params,
                    &declared_constants,
                    &declared_tasks,
                    &declared_actions,
                )?;
            }
            // Assert orderings are acyclic
            if !method.tn.orderings.is_acyclic() {
                return Err(SemanticErrorType::CyclicOrderingDeclaration(
                    method.tn.ordering_pos.unwrap(),
                ));
            }
        }
        // Check whether all compound tasks can be refined to primitive ones
        let tdg = TDG::new(self.domain);
        for task in declared_tasks.iter() {
            let reachables = tdg.reachable(&task.name);
            if (reachables.primitives.len() == 0) && (reachables.nullable == false) {
                warnings.push(WarningType::NoPrimitiveRefinement(WarningInfo {
                    symbol: task.name.to_string(),
                    position: task.name_pos,
                }));
            }
        }
        let type_hierarchy = self.type_checker.get_type_hierarchy();
        Ok(SymbolTable {
            warnings: warnings,
            constants: declared_constants,
            predicates: declared_predicates,
            tasks: declared_tasks,
            actions: declared_actions,
            type_hierarchy: type_hierarchy,
        })
    }

    // returns declared predicates (if there is no error)
    fn verify_predicates(&'a self) -> Result<HashSet<&'a Predicate>, SemanticErrorType> {
        let mut declared_predicates = HashSet::new();
        let mut predicate_positions = HashMap::new();
        for predicate in self.domain.predicates.iter() {
            if !declared_predicates.insert(predicate) {
                return Err(SemanticErrorType::DuplicatePredicateDeclaration(
                    DuplicateError {
                        symbol: predicate.name.to_string(),
                        first_pos: *predicate_positions.get(predicate.name).unwrap(),
                        second_pos: predicate.name_pos,
                    },
                ));
            } else {
                predicate_positions.insert(predicate.name, predicate.name_pos);
            }
            if let Some(error) = self
                .type_checker
                .check_type_declarations(&predicate.variables)
            {
                return Err(error);
            }
        }
        Ok(declared_predicates)
    }

    // returns declared compound tasks (if there is no error)
    fn verify_compound_tasks(&'a self) -> Result<HashSet<&Task<'a>>, SemanticErrorType> {
        let mut declared_tasks = HashSet::new();
        let mut task_positions = HashMap::new();
        for task in self.domain.compound_tasks.iter() {
            if !declared_tasks.insert(task) {
                return Err(SemanticErrorType::DuplicateCompoundTaskDeclaration(
                    DuplicateError {
                        symbol: task.name.to_string(),
                        first_pos: *task_positions.get(task.name).unwrap(),
                        second_pos: task.name_pos,
                    },
                ));
            } else {
                task_positions.insert(task.name, task.name_pos);
            }
            // assert parameter types are declared
            if let Some(error) = self.type_checker.check_type_declarations(&task.parameters) {
                return Err(error);
            }
        }
        Ok(declared_tasks)
    }

    pub fn check_duplicate_requirements(
        requirements: &'a Vec<RequirementType>,
    ) -> Option<SemanticErrorType> {
        let mut names = HashSet::new();
        for req in requirements {
            if !names.insert(req) {
                return Some(SemanticErrorType::DuplicateRequirementDeclaration(*req));
            }
        }
        None
    }

    // fn verify_formula(formula: &Formula<'a>, declared_predicates: HashSet<u>)
}
