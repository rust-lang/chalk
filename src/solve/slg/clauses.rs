use ir::{ProgramEnvironment, InEnvironment, DomainGoal, ProgramClause};
use ir::could_match::CouldMatch;
use std::sync::Arc;

/// Returns all clauses that are relevant to `goal`, either from
/// the environment or the program.
pub(super) fn clauses(
    program: &Arc<ProgramEnvironment>,
    goal: &InEnvironment<DomainGoal>,
) -> Vec<ProgramClause> {
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
