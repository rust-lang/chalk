use ir::*;
use std::sync::Arc;

use super::infer::UniverseIndex;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Environment {
    pub universe: UniverseIndex,
    pub clauses: Vec<WhereClause>,
}

impl Environment {
    pub fn new() -> Arc<Environment> {
        Arc::new(Environment { universe: UniverseIndex::root(), clauses: vec![] })
    }

    pub fn add_clauses<I>(&self, clauses: I) -> Arc<Environment>
        where I: IntoIterator<Item = WhereClause>
    {
        let mut env = self.clone();
        env.clauses.extend(clauses);
        Arc::new(env)
    }

    pub fn new_universe(&self) -> Arc<Environment> {
        let mut env = self.clone();
        env.universe = UniverseIndex { counter: self.universe.counter + 1 };
        Arc::new(env)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InEnvironment<G> {
    pub environment: Arc<Environment>,
    pub goal: G,
}

impl<G> InEnvironment<G> {
    pub fn new(environment: &Arc<Environment>, goal: G) -> Self {
        InEnvironment { environment: environment.clone(), goal }
    }

    pub fn map<OP, H>(self, op: OP) -> InEnvironment<H>
        where OP: FnOnce(G) -> H
    {
        InEnvironment {
            environment: self.environment,
            goal: op(self.goal),
        }
    }
}
