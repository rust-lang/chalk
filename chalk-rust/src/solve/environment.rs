use ir::*;
use std::sync::Arc;

use super::infer::UniverseIndex;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Environment {
    pub universe: UniverseIndex,
    pub clauses: Vec<WhereClause>,
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
