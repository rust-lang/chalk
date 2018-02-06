//! An alternative solver based around the SLG algorithm, which
//! implements the well-formed semantics. See the README.md
//! file for details.

#![cfg_attr(not(test), allow(dead_code))] // FOR NOW

crate mod forest;

mod logic;
mod stack;
mod strand;
mod table;
mod tables;
mod test;

use ir::ProgramEnvironment;
use solve::Solution;
use solve::slg::UCanonicalGoal;
use std::sync::Arc;

use self::forest::Forest;

/// Convenience fn for solving a root goal. It would be better to
/// createa a `Forest` so as to enable cahcing between goals, however.
crate fn solve_root_goal(
    max_size: usize,
    program: &Arc<ProgramEnvironment>,
    root_goal: &UCanonicalGoal,
) -> Option<Solution> {
    let mut forest = Forest::new(program, max_size);
    forest.solve(root_goal)
}
