use crate::*;

// synthesized identifiers must outlive the parsed program, whose strings all
// borrow from the input buffer; leaking gives them 'static, which coerces to
// any 'a. the volume is bounded by the number of synthesized names per run.
pub(crate) fn leak(name: String) -> &'static str {
    Box::leak(name.into_boxed_str())
}

pub struct Transpiler<'a> {
    pub(crate) program: HDDLProgram<'a>,
}

impl<'a> Transpiler<'a> {
    pub fn new(program: HDDLProgram<'a>) -> Transpiler<'a> {
        Transpiler { program }
    }

    pub fn from_hddl(
        domain: &'a Vec<u8>,
        problem: Option<&'a Vec<u8>>,
    ) -> Result<Transpiler<'a>, ParsingError> {
        Ok(Transpiler::new(HDDLProgram::from_hddl(domain, problem)?))
    }

    pub fn from_json(json: &'a str) -> Result<Transpiler<'a>, ParsingError> {
        Ok(Transpiler::new(HDDLProgram::from_json(json)?))
    }

    pub fn from_input(input: Input<'a>) -> Result<Transpiler<'a>, ParsingError> {
        match input {
            Input::Hddl { domain, problem } => Transpiler::from_hddl(domain, problem),
            Input::Json(json) => Transpiler::from_json(json),
        }
    }

    // verifies the program, then applies a transformation; chainable before emission
    pub fn transform(
        mut self,
        transformation: Transformation,
    ) -> Result<Transpiler<'a>, ParsingError> {
        self.verify()?;
        match transformation {
            // TODO: make the transformation return actual errors
            Transformation::RemoveTypes => self.remove_types(),
            Transformation::ConjunctivePreconditions => self.conjunctive_preconditions()?,
            Transformation::RemoveEqualityConstraints => self.remove_equality_constraints()?,
            Transformation::QuantifierElimintation => self.remove_quantifiers()?,
        }
        Ok(self)
    }

    pub fn verify(&self) -> Result<Vec<WarningType>, ParsingError> {
        self.program.verify()
    }

    pub fn metadata(&self) -> Result<MetaData, ParsingError> {
        self.program.metadata()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.program).unwrap()
    }

    // predicates are identified by name and arity
    fn is_declared(&self, predicate_name: &str, arity: usize) -> bool {
        self.program
            .domain
            .predicates
            .iter()
            .any(|x| x.name == predicate_name && x.arity() == arity)
    }

    pub fn to_hddl(&self) -> (String, Option<String>) {
        (
            self.program.domain.to_string(),
            self.program
                .problem
                .as_ref()
                .map(|problem| problem.to_string()),
        )
    }

    pub fn into_program(self) -> HDDLProgram<'a> {
        self.program
    }

    // a predicate name unused for the given arity: `name` itself when free,
    // otherwise the first numeric-suffixed variant that is
    pub(crate) fn fresh_predicate_name(&self, name: &'a str, arity: usize) -> &'a str {
        if !self.is_declared(name, arity) {
            return name;
        }
        let fresh = (0..)
            .map(|i| format!("{name}_{i}"))
            .find(|candidate| !self.is_declared(candidate, arity))
            .unwrap();
        leak(fresh)
    }

    // recomputes the requirement flags from the actual content: a flag is
    // added when its construct is used and no declared flag covers it, and
    // dropped (from the domain and the problem) when the construct is absent.
    pub(crate) fn refresh_requirements(&mut self) {
        let program = &mut self.program;
        let mut preconditions: Vec<&Precondition> = vec![];
        let mut effects: Vec<&Effect> = vec![];
        for action in program.domain.actions.iter() {
            preconditions.extend(action.preconditions.iter());
            effects.extend(action.effects.iter());
        }
        for method in program.domain.methods.iter() {
            preconditions.extend(method.precondition.iter());
        }
        preconditions.extend(program.problem.iter().flat_map(|p| p.goal.iter()));
        let any_precondition = |predicate: fn(&Precondition) -> bool| {
            preconditions
                .iter()
                .any(|formula| formula.any_subformula(&mut |f| predicate(f)))
        };
        let any_effect = |predicate: fn(&Effect) -> bool| {
            effects
                .iter()
                .any(|effect| effect.any_subformula(&mut |f| predicate(f)))
        };

        let has_forall = any_precondition(|f| matches!(f, Precondition::ForAll(_, _)));
        let has_exists = any_precondition(|f| matches!(f, Precondition::Exists(_, _)));
        let has_negative = any_precondition(|f| matches!(f, Precondition::Not(_)));
        let has_disjunction = any_precondition(|f| {
            matches!(
                f,
                Precondition::Or(_) | Precondition::Xor(_) | Precondition::Imply(_, _)
            )
        });
        let has_conditional =
            any_effect(|f| matches!(f, Effect::Imply(_, _) | Effect::ForAll(_, _)));
        let has_probabilistic = any_effect(|f| matches!(f, Effect::Probabilistic(_)));
        let has_constraints = program
            .domain
            .methods
            .iter()
            .filter_map(|method| method.tn.constraints.as_ref())
            .chain(
                program
                    .problem
                    .iter()
                    .filter_map(|p| p.init_tn.as_ref())
                    .filter_map(|init_tn| init_tn.tn.constraints.as_ref()),
            )
            .any(|constraints| !constraints.is_empty());
        let has_equality = has_constraints
            || any_precondition(|f| matches!(f, Precondition::Equals(_, _)))
            || any_effect(|f| match f {
                Effect::Equals(_, _) => true,
                // the guard of a conditional effect is a precondition
                Effect::Imply(guard, _) => {
                    guard.any_subformula(&mut |g| matches!(g, Precondition::Equals(_, _)))
                }
                _ => false,
            });
        let has_hierarchy = !program.domain.compound_tasks.is_empty()
            || !program.domain.methods.is_empty()
            || program
                .problem
                .as_ref()
                .is_some_and(|p| p.init_tn.is_some());
        let has_method_preconditions = program
            .domain
            .methods
            .iter()
            .any(|method| method.precondition.is_some());
        // numerics: action costs cover a domain that only increases total-cost
        // by a constant or a fluent and, at most, minimises it; anything else
        // needs numeric fluents
        let is_cost_increase = |f: &Effect| {
            matches!(
                f,
                Effect::Numeric(
                    AssignmentOperator::Increase,
                    fluent,
                    NumericExpr::Number(_) | NumericExpr::Fluent(_)
                ) if fluent.name == "total-cost"
            )
        };
        let has_numeric_effect = any_effect(|f| matches!(f, Effect::Numeric(_, _, _)));
        let exceeds_action_costs = effects.iter().any(|effect| {
            effect.any_subformula(&mut |f| {
                matches!(f, Effect::Numeric(_, _, _)) && !is_cost_increase(f)
            })
        }) || program.problem.as_ref().is_some_and(|p| {
            p.metric.as_ref().is_some_and(|metric| {
                !matches!(metric, Metric::Minimize(NumericExpr::Fluent(f)) if f.name == "total-cost")
            })
        });
        let uses_numerics = has_numeric_effect
            || !program.domain.functions.is_empty()
            || program
                .problem
                .as_ref()
                .is_some_and(|p| p.init_values.is_some() || p.metric.is_some());
        drop(preconditions);
        drop(effects);

        let uses = [
            (RequirementType::UniversalPreconditions, has_forall),
            (RequirementType::ExistentialPreconditions, has_exists),
            (RequirementType::NegativePreconditions, has_negative),
            (RequirementType::DisjunctivePreconditions, has_disjunction),
            (RequirementType::ConditionalEffects, has_conditional),
            (RequirementType::ProbabilisticEffects, has_probabilistic),
            (RequirementType::Equality, has_equality),
            (RequirementType::Hierarchy, has_hierarchy),
            (
                RequirementType::MethodPreconditions,
                has_method_preconditions,
            ),
            (
                RequirementType::ActionCosts,
                uses_numerics && !exceeds_action_costs,
            ),
            (
                RequirementType::NumericFluents,
                uses_numerics && exceeds_action_costs,
            ),
        ];
        let requirements = &mut program.domain.requirements;
        for (flag, used) in uses {
            let covered = requirements.iter().any(|declared| declared.covers(flag));
            if used && !covered {
                requirements.push(flag);
            }
        }
        // the numeric spellings are one family: with no numerics all of them go,
        // otherwise the declared one stays whichever fragment is in use
        let unused: Vec<RequirementType> = uses
            .iter()
            .filter(|(_, used)| !used)
            .map(|(flag, _)| *flag)
            .filter(|flag| !(uses_numerics && RequirementType::Fluents.covers(*flag)))
            .collect();
        requirements.retain(|declared| !unused.contains(declared));
        if !uses_numerics {
            requirements.retain(|declared| *declared != RequirementType::Fluents);
        }
        if let Some(problem) = program.problem.as_mut() {
            problem
                .requirements
                .retain(|declared| !unused.contains(declared));
            if !uses_numerics {
                problem
                    .requirements
                    .retain(|declared| *declared != RequirementType::Fluents);
            }
        }
    }
}
