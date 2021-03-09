use crate::search_graph::DepthFirstNumber;
use chalk_ir::{Goal, InEnvironment, UCanonical};

pub type UCanonicalGoal<I> = UCanonical<InEnvironment<Goal<I>>>;

mod coinduction_handler;
mod combine;
mod fulfill;
mod recursive;
mod search_graph;
pub mod solve;
mod stack;

use coinduction_handler::CoinductiveCycleDependencyBoundaries;
pub use recursive::RecursiveSolver;

/// The `minimums` struct is used while solving to track whether we encountered
/// any cycles in the process.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Minimums {
    pub(crate) positive: DepthFirstNumber,
    pub(crate) coinductive_cycle_boundaries: Option<CoinductiveCycleDependencyBoundaries>,
}

impl Minimums {
    pub fn new() -> Self {
        Minimums {
            positive: DepthFirstNumber::MAX,
            coinductive_cycle_boundaries: None,
        }
    }

    pub fn update_from(&mut self, minimums: &Minimums) {
        self.positive = ::std::cmp::min(self.positive, minimums.positive);

        if let Some(other_cycle_boundaries) = minimums.coinductive_cycle_boundaries {
            self.update_coinductive_cycle_boundaries(other_cycle_boundaries);
        }
    }

    pub fn update_coinductive_cycle_boundaries(
        &mut self,
        other_boundaries: CoinductiveCycleDependencyBoundaries,
    ) {
        if let Some(ref mut boundaries) = self.coinductive_cycle_boundaries {
            boundaries.update_from(other_boundaries);
        } else {
            self.coinductive_cycle_boundaries = Some(other_boundaries);
        }
    }
}
