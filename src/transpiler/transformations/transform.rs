// transformations that can be applied to a program before emission
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transformation {
    // every action/method precondition becomes a quantifier-free conjunction
    // of literals
    ConjunctivePreconditions,
    // (= a b) literals and method task-network :constraints become atoms over
    // a fresh static `equal` predicate with reflexive init facts
    RemoveEqualityConstraints,
    // types become unary predicates
    RemoveTypes,
    // remove quantifiers
    QuantifierElimintation,
}
