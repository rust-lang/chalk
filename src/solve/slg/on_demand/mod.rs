//! An alternative solver based around the SLG algorithm, which
//! implements the well-formed semantics. The semantics are my own
//! take on two papers, which I will refer to as EWFS and NTFD
//! respectively:
//!
//! > Efficient Top-Down Computation of Queries Under the Well-formed Semantics
//! > (Chen, Swift, and Warren; Journal of Logic Programming '95)
//!
//! > A New Formulation of Tabled resolution With Delay
//! > (Swift; EPIA '99)
//!
//! In addition, I incorporated extensions from the following papers,
//! which I will refer to as SA and RR respectively, that
//! describes how to do introduce approximation when processing
//! subgoals and so forth:
//!
//! > Terminating Evaluation of Logic Programs with Finite Three-Valued Models
//! > Riguzzi and Swift; ACM Transactions on Computational Logic 2013
//! > (Introduces "subgoal abstraction", hence the name SA)
//! >
//! > Radial Restraint
//! > Grosof and Swift; 2013
//!
//! Another useful paper that gives a kind of high-level overview of
//! concepts at play is the following, which I will refer to as XSB:
//!
//! > XSB: Extending Prolog with Tabled Logic Programming
//! > (Swift and Warren; Theory and Practice of Logic Programming '10)
//!
//! While this code is inspired by the algorithms described in those
//! papers, it is quite different. It also takes some inspiration from MiniKanren
//! and in particular the "breadth-first and functional" approach used there.
//!
//! Also, the SLG algorithm in general had to be extended to our
//! context, and in particular to coping with hereditary harrop
//! predicates and our version of unification (which produces
//! subgoals). I believe those to be largely faithful
//! extensions. However, there are some other places where I
//! intentionally dieverged from the semantics as described in the
//! papers -- e.g. by more aggressively approximating -- which I
//! marked them with a comment DIVERGENCE. Those places may want to be
//! evaluated in the future.
//!
//! Glossary of other terms:
//!
//! - WAM: Warren abstract machine, an efficient way to evaluate Prolog programs.
//!   See <http://wambook.sourceforge.net/>.
//! - HH: Hereditary harrop predicates. What Chalk deals in.
//!   Popularized by Lambda Prolog.

#![cfg_attr(not(test), allow(dead_code))] // FOR NOW

pub mod forest;

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
pub fn solve_root_goal(
    max_size: usize,
    program: &Arc<ProgramEnvironment>,
    root_goal: &UCanonicalGoal,
) -> Option<Solution> {
    let mut forest = Forest::new(program, max_size);
    forest.solve(root_goal)
}
