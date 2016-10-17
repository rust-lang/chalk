use formula::*;
use std::sync::Arc;
use std::slice::Iter;

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
    data: Arc<EnvironmentData<C>>,
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
    clauses: Vec<Clause<C>>,
}

impl<C> EnvironmentData<C> {
    /// Iterator of clauses that may satisfy `goal`
    pub fn clauses<'a>(&'a self, goal: &'a Goal<C>) -> Clauses<'a, C> {
        Clauses {
            goal: goal,
            next_env: self.parent.as_ref(),
            next_clauses: self.clauses.iter(),
        }
    }
}

pub struct Clauses<'a, C: 'a> {
    goal: &'a Goal<C>,
    next_env: Option<&'a Environment<C>>,
    next_clauses: Iter<'a, Clause<C>>,
}

impl<'a, C> Iterator for Clauses<'a, C> {
    type Item = &'a Clause<C>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(formula) = self.next_clauses.next() {
            Some(formula)
        } else if let Some(env) = self.next_env {
            *self = env.clauses(self.goal);
            self.next()
        } else {
            None
        }
    }
}
