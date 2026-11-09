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
                Self::expand_quantified_effects(effect, problem, &type_checker);
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
        formula: &mut Precondition<'a>,
        problem: &ProblemAST<'a>,
        type_checker: &TypeChecker<'a>,
    ) {
        match formula {
            Precondition::ForAll(_, _) | Precondition::Exists(_, _) => {
                let quantified = std::mem::replace(formula, Precondition::Empty);
                let (vars, body, is_forall) = match quantified {
                    Precondition::ForAll(vars, body) => (vars, body, true),
                    Precondition::Exists(vars, body) => (vars, body, false),
                    _ => unreachable!(),
                };

                let groundings = Self::groundings(&vars, problem, type_checker);
                let mut instances = Vec::with_capacity(groundings.len());
                for grounding in groundings {
                    let mut instance = (*body).clone();
                    for (var, object) in vars.iter().zip(&grounding) {
                        instance = instance.substitute(var.name, object);
                    }
                    // the body may contain further quantifiers
                    Self::expand_quantifiers(&mut instance, problem, type_checker);
                    instances.push(Box::new(instance));
                }

                *formula = if is_forall {
                    Precondition::And(instances)
                } else {
                    Precondition::Or(instances)
                };
            }
            Precondition::Not(f) => Self::expand_quantifiers(f, problem, type_checker),
            Precondition::And(fs) | Precondition::Or(fs) | Precondition::Xor(fs) => {
                for f in fs.iter_mut() {
                    Self::expand_quantifiers(f, problem, type_checker);
                }
            }
            Precondition::Imply(lhs, rhs) => {
                Self::expand_quantifiers(lhs, problem, type_checker);
                Self::expand_quantifiers(rhs, problem, type_checker);
            }
            Precondition::Empty | Precondition::Atom(_) | Precondition::Equals(_, _) => {}
        }
    }

    /// The effect counterpart of `expand_quantifiers`.
    fn expand_quantified_effects(
        effect: &mut Effect<'a>,
        problem: &ProblemAST<'a>,
        type_checker: &TypeChecker<'a>,
    ) {
        match effect {
            Effect::ForAll(_, _) | Effect::Exists(_, _) => {
                let quantified = std::mem::replace(effect, Effect::Empty);
                let (vars, body, is_forall) = match quantified {
                    Effect::ForAll(vars, body) => (vars, body, true),
                    Effect::Exists(vars, body) => (vars, body, false),
                    _ => unreachable!(),
                };

                let groundings = Self::groundings(&vars, problem, type_checker);
                let mut instances = Vec::with_capacity(groundings.len());
                for grounding in groundings {
                    let mut instance = (*body).clone();
                    for (var, object) in vars.iter().zip(&grounding) {
                        instance = instance.substitute(var.name, object);
                    }
                    // the body may contain further quantifiers
                    Self::expand_quantified_effects(&mut instance, problem, type_checker);
                    instances.push(Box::new(instance));
                }

                *effect = if is_forall {
                    Effect::And(instances)
                } else {
                    Effect::Or(instances)
                };
            }
            Effect::Not(f) | Effect::Probabilistic(_, f) => {
                Self::expand_quantified_effects(f, problem, type_checker)
            }
            Effect::And(fs) | Effect::Or(fs) | Effect::Xor(fs) => {
                for f in fs.iter_mut() {
                    Self::expand_quantified_effects(f, problem, type_checker);
                }
            }
            Effect::Imply(lhs, rhs) => {
                Self::expand_quantified_effects(lhs, problem, type_checker);
                Self::expand_quantified_effects(rhs, problem, type_checker);
            }
            Effect::Empty | Effect::Atom(_) | Effect::Equals(_, _) => {}
        }
    }

    /// Every assignment of type-compatible objects to the bound variables, in
    /// cartesian-product order.
    fn groundings(
        vars: &[Symbol<'a>],
        problem: &ProblemAST<'a>,
        type_checker: &TypeChecker<'a>,
    ) -> Vec<Vec<Symbol<'a>>> {
        let mut groundings: Vec<Vec<Symbol<'a>>> = vec![Vec::new()];
        for var in vars {
            // objects the variable can take, respecting subtyping
            let domain = Self::objects_of_type(problem, var.symbol_type, type_checker);
            let mut extended = Vec::with_capacity(groundings.len() * domain.len());
            for grounding in &groundings {
                for object in &domain {
                    let mut grounding = grounding.clone();
                    grounding.push(object.clone());
                    extended.push(grounding);
                }
            }
            groundings = extended;
        }
        groundings
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
