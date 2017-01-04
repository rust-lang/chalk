use ir::*;
use std::sync::Arc;

use super::infer::UniverseIndex;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Environment {
    pub universe: UniverseIndex,
    pub program: Arc<Program>,
    pub clauses: Vec<WhereClause>,
}

