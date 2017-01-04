use ir::*;

use super::infer::UniverseIndex;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Environment {
    pub universe: UniverseIndex,
    pub clauses: Vec<WhereClause>,
}

