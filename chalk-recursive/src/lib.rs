use crate::search_graph::DepthFirstNumber;
use chalk_ir::{
    interner::Interner, ClausePriority, Fallible, Goal, InEnvironment, NoSolution, UCanonical,
};

pub type UCanonicalGoal<I> = UCanonical<InEnvironment<Goal<I>>>;

mod cache;
mod combine;
mod fulfill;
mod recursive;
mod search_graph;
pub mod solve;
mod stack;

pub use cache::Cache;
use chalk_solve::{Guidance, Solution};
pub use recursive::RecursiveSolver;
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PrioritizedSolution<I: Interner> {
    priority: ClausePriority,
    solution: Fallible<Solution<I>>,
}

impl<I: Interner> PrioritizedSolution<I> {
    /// Create a new prioritized solution.
    pub(crate) fn new(solution: Fallible<Solution<I>>, priority: ClausePriority) -> Self {
        Self { priority, solution }
    }

    /// Create a high priority solution.
    pub(crate) fn high(solution: Fallible<Solution<I>>) -> Self {
        Self::new(solution, ClausePriority::High)
    }

    /// Returns a high-priority solution that represents an error (no solution)
    pub(crate) fn error() -> Self {
        Self::high(Err(NoSolution))
    }

    /// Returns a high-priority solution that represents ambiguity with no guidance.
    pub(crate) fn ambiguity() -> Self {
        Self::high(Ok(Solution::Ambig(Guidance::Unknown)))
    }
}
/// The `minimums` struct is used while solving to track whether we encountered
/// any cycles in the process.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Minimums {
    pub(crate) positive: DepthFirstNumber,
}

impl Minimums {
    pub fn new() -> Self {
        Minimums {
            positive: DepthFirstNumber::MAX,
        }
    }

    pub fn update_from(&mut self, minimums: Minimums) {
        self.positive = ::std::cmp::min(self.positive, minimums.positive);
    }
}
