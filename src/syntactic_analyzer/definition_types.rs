pub enum DefinitionType<'a> {
    Domain(&'a str),
    Problem(ProblemDefinition<'a>),
}

pub struct ProblemDefinition<'a> {
    pub problem_name: &'a str,
    pub domain_name: &'a str,
}

pub enum FileVariant {
    Domain,
    Problem,
    MaybeNotHDDL,
}
