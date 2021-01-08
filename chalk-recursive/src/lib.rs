use crate::search_graph::DepthFirstNumber;
use chalk_ir::{Goal, InEnvironment, UCanonical};

pub type UCanonicalGoal<I> = UCanonical<InEnvironment<Goal<I>>>;

mod combine;
mod fulfill;
mod recursive;
mod search_graph;
pub mod solve;
mod stack;

pub use recursive::RecursiveSolver;

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
