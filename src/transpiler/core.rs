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

    // recompute the requirement flags from the actual content
    pub(crate) fn refresh_requirements(&mut self) {
        let program = &mut self.program;
        let mut preconditions: Vec<&Formula> = vec![];
        let mut effects: Vec<&Formula> = vec![];
        for action in program.domain.actions.iter() {
            preconditions.extend(action.preconditions.iter());
            effects.extend(action.effects.iter());
        }
        for method in program.domain.methods.iter() {
            preconditions.extend(method.precondition.iter());
        }
        preconditions.extend(program.problem.iter().flat_map(|p| p.goal.iter()));

        let has_forall = preconditions
            .iter()
            .any(|formula| formula.any_subformula(&mut |f| matches!(f, Formula::ForAll(_, _))));
        let has_exists = preconditions
            .iter()
            .any(|formula| formula.any_subformula(&mut |f| matches!(f, Formula::Exists(_, _))));
        let has_negative = preconditions
            .iter()
            .any(|formula| formula.any_subformula(&mut |f| matches!(f, Formula::Not(_))));
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
            || preconditions
                .iter()
                .chain(effects.iter())
                .any(|formula| formula.any_subformula(&mut |f| matches!(f, Formula::Equals(_, _))));
        drop(preconditions);
        drop(effects);

        let requirements = &mut program.domain.requirements;
        if has_forall && !requirements.contains(&RequirementType::UniversalPreconditions) {
            requirements.push(RequirementType::UniversalPreconditions);
        }
        if !has_forall {
            requirements.retain(|r| *r != RequirementType::UniversalPreconditions);
        }
        if has_exists && !requirements.contains(&RequirementType::ExistentialPreconditions) {
            requirements.push(RequirementType::ExistentialPreconditions);
        }
        if !has_exists {
            requirements.retain(|r| *r != RequirementType::ExistentialPreconditions);
        }
        if has_negative && !requirements.contains(&RequirementType::NegativePreconditions) {
            requirements.push(RequirementType::NegativePreconditions);
        }
        if has_equality && !requirements.contains(&RequirementType::Equality) {
            requirements.push(RequirementType::Equality);
        }
        if !has_equality {
            requirements.retain(|r| *r != RequirementType::Equality);
        }
        if let Some(problem) = program.problem.as_mut() {
            if !has_forall {
                problem
                    .requirements
                    .retain(|r| *r != RequirementType::UniversalPreconditions);
            }
            if !has_equality {
                problem
                    .requirements
                    .retain(|r| *r != RequirementType::Equality);
            }
        }
    }
}
