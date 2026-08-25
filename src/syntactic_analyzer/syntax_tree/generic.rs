use super::*;

#[derive(Serialize)]
#[serde(untagged)]
pub enum AbstractSyntaxTree<'a> {
    Domain(DomainAST<'a>),
    Problem(ProblemAST<'a>),
}

impl<'a> From<DomainAST<'a>> for AbstractSyntaxTree<'a> {
    fn from(d: domain::DomainAST<'a>) -> Self {
        AbstractSyntaxTree::Domain(d)
    }
}

impl<'a> From<ProblemAST<'a>> for AbstractSyntaxTree<'a> {
    fn from(p: problem::ProblemAST<'a>) -> Self {
        AbstractSyntaxTree::Problem(p)
    }
}
