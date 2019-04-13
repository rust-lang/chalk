use chalk_ir::{DomainGoal, ProgramClause, TraitId};
use std::fmt::Debug;

#[macro_use]
extern crate chalk_macros;

mod coinductive_goal;
pub mod ext;
mod infer;
mod solve;

/// The trait for defining the program clauses that are in scope when
/// solving a goal.
pub trait ChalkSolveDatabase: Debug {
    /// Returns a set of program clauses that could possibly match
    /// `goal`. This can be any superset of the correct set, but the
    /// more precise you can make it, the more efficient solving will
    /// be.
    fn program_clauses_that_could_match(&self, goal: &DomainGoal, vec: &mut Vec<ProgramClause>);

    fn is_coinductive_trait(&self, trait_id: TraitId) -> bool;
}

pub use solve::Solution;
pub use solve::Solver;
pub use solve::SolverChoice;
pub use solve::TestSolver;
