use crate::ir::{DomainGoal, InEnvironment, ProgramClause, ProgramEnvironment};
use crate::ir::could_match::CouldMatch;
use crate::solve::slg::context::prelude::*;
use crate::solve::slg::forest::Forest;
use std::sync::Arc;

impl<C: Context> Forest<C> {
    /// Returns all clauses that are relevant to `goal`, either from
    /// the environment or the program.
    pub(super) fn clauses(
        program: &Arc<ProgramEnvironment<DomainGoal>>,
        goal: &InEnvironment<DomainGoal>,
    ) -> Vec<ProgramClause<DomainGoal>> {
        let &InEnvironment {
            ref environment,
            ref goal,
        } = goal;

        let environment_clauses = environment
            .clauses
            .iter()
            .filter(|&env_clause| env_clause.could_match(goal))
            .map(|env_clause| env_clause.clone().into_program_clause());

        let program_clauses = program
            .program_clauses
            .iter()
            .filter(|clause| clause.could_match(goal))
            .cloned();

        environment_clauses.chain(program_clauses).collect()
    }
}
