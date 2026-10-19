use crate::*;

impl<'a> Transpiler<'a> {
    pub(crate) fn remove_quantifiers(&mut self) -> Result<(), ParsingError> {
        let Some(problem) = &self.program.problem else {
            return Err(ParsingError::Transformation(
                "problem file is not provided".to_string(),
            ));
        };
        let type_checker = TypeChecker::new(&self.program.domain.types);
        for action in self.program.domain.actions.iter_mut() {
            if let Some(precondition) = &mut action.preconditions
                && precondition.is_quantified()
            {
                Self::expand_quantifiers(precondition, problem, &type_checker);
            }
            if let Some(effect) = &mut action.effects
                && effect.is_quantified()
            {
                Self::expand_quantifiers(effect, problem, &type_checker);
            }
        }

        for method in self.program.domain.methods.iter_mut() {
            if let Some(precondition) = &mut method.precondition
                && precondition.is_quantified()
            {
                Self::expand_quantifiers(precondition, problem, &type_checker);
            }
        }
        Ok(())
    }

    /// Recursively replaces every `forall`/`exists` with the conjunction or
    /// disjunction of its body instantiated over all type-compatible objects.
    fn expand_quantifiers(
        formula: &mut Formula<'a>,
        problem: &ProblemAST<'a>,
        type_checker: &TypeChecker<'a>,
    ) {
        match formula {
            Formula::ForAll(_, _) | Formula::Exists(_, _) => {
                let quantified = std::mem::replace(formula, Formula::Empty);
                let (vars, body, is_forall) = match quantified {
                    Formula::ForAll(vars, body) => (vars, body, true),
                    Formula::Exists(vars, body) => (vars, body, false),
                    _ => unreachable!(),
                };

                // objects each bound variable can take, respecting subtyping
                let var_domains: Vec<Vec<Symbol<'a>>> = vars
                    .iter()
                    .map(|v| Self::objects_of_type(problem, v.symbol_type, type_checker))
                    .collect();

                // cartesian product over the bound variables
                let mut groundings: Vec<Vec<_>> = vec![Vec::new()];
                for domain in &var_domains {
                    let mut extended = Vec::with_capacity(groundings.len() * domain.len());
                    for grounding in &groundings {
                        for object in domain {
                            let mut grounding = grounding.clone();
                            grounding.push(object);
                            extended.push(grounding);
                        }
                    }
                    groundings = extended;
                }

                let mut instances = Vec::with_capacity(groundings.len());
                for grounding in groundings {
                    let mut instance = (*body).clone();
                    for (var, object) in vars.iter().zip(grounding) {
                        instance = instance.substitute(var.name, object);
                    }
                    // the body may contain further quantifiers
                    Self::expand_quantifiers(&mut instance, problem, type_checker);
                    instances.push(Box::new(instance));
                }

                *formula = if is_forall {
                    Formula::And(instances)
                } else {
                    Formula::Or(instances)
                };
            }
            Formula::Not(f) | Formula::Probabilistic(_, f) => {
                Self::expand_quantifiers(f, problem, type_checker)
            }
            Formula::And(fs) | Formula::Or(fs) | Formula::Xor(fs) => {
                for f in fs.iter_mut() {
                    Self::expand_quantifiers(f, problem, type_checker);
                }
            }
            Formula::Imply(lhs, rhs) => {
                Self::expand_quantifiers(lhs, problem, type_checker);
                Self::expand_quantifiers(rhs, problem, type_checker);
            }
            Formula::Empty | Formula::Atom(_) | Formula::Equals(_, _) => {}
        }
    }

    /// Objects compatible with `of_type` under the declared type hierarchy.
    fn objects_of_type(
        problem: &ProblemAST<'a>,
        of_type: Option<&'a str>,
        type_checker: &TypeChecker<'a>,
    ) -> Vec<Symbol<'a>> {
        let subtypes = if let Some(typing) = of_type {
            let mut temp = type_checker.get_subtypes(typing);
            temp.insert(typing);
            temp
        } else {
            type_checker.get_types()
        };
        problem
            .objects
            .iter()
            .filter(|o| {
                if let Some(typing) = o.symbol_type {
                    subtypes.contains(typing)
                } else {
                    of_type.is_none()
                }
            })
            .cloned()
            .collect()
    }
}
