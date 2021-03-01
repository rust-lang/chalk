use crate::search_graph::DepthFirstNumber;
use chalk_ir::{Goal, InEnvironment, UCanonical};

pub type UCanonicalGoal<I> = UCanonical<InEnvironment<Goal<I>>>;

mod coinduction;
mod combine;
mod fulfill;
mod recursive;
mod search_graph;
pub mod solve;
mod stack;

pub use recursive::RecursiveSolver;
use rustc_hash::FxHashSet;

/// The `minimums` struct is used while solving to track whether we encountered
/// any cycles in the process.
#[derive(Clone, Debug)]
pub(crate) struct Minimums {
    pub(crate) positive: DepthFirstNumber,
    pub(crate) coinductive_cycle_starts: FxHashSet<DepthFirstNumber>,
}

impl Minimums {
    pub fn new() -> Self {
        Minimums {
            positive: DepthFirstNumber::MAX,
            coinductive_cycle_starts: FxHashSet::default(),
        }
    }

    pub fn update_from(&mut self, minimums: &Minimums) {
        self.positive = ::std::cmp::min(self.positive, minimums.positive);
        self.add_cycle_starts(&minimums.coinductive_cycle_starts);
    }

    pub fn add_cycle_start(&mut self, start: DepthFirstNumber) {
        self.coinductive_cycle_starts.insert(start);
    }

    pub fn add_cycle_starts(&mut self, starts: &FxHashSet<DepthFirstNumber>) {
        self.coinductive_cycle_starts.extend(starts.iter());
    }

    pub fn is_mature(&self) -> bool {
        self.coinductive_cycle_starts.is_empty()
    }
}
