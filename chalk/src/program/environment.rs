use formula::*;
use std::sync::Arc;

/// List of fact that we believe to be true. Most of these come from
/// the initial program. So if you have a program like:
///
///     foo(X) :- bar(X).
///
/// This corresponds to an environmental fact like:
///
///     forall(X -> bar(X) => foo(X))
///
/// Environments can be extended dynamically, if we encounter a goal
/// like `X => Y` (that would cause us to push `X` into the
/// environment and then try to prove `Y`). Therefore they are
/// structured into a chain.
#[derive(Clone, Debug)]
pub struct Environment<C> {
    data: Arc<EnvironmentData<C>>
}

deref_to!(Environment<C>.data => EnvironmentData<C>);

impl<C> Environment<C> {
    pub fn new(data: EnvironmentData<C>) -> Self {
        Environment { data: Arc::new(data) }
    }
}

#[derive(Debug)]
pub struct EnvironmentData<C> {
    parent: Option<Environment<C>>,
    facts: Vec<Formula<C>>,
}

