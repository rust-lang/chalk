use chalk_ir::{
    Goal, InEnvironment, UCanonical
};
use super::search_graph::DepthFirstNumber;

pub type UCanonicalGoal<I> = UCanonical<InEnvironment<Goal<I>>>;

/// The `minimums` struct is used while solving to track whether we encountered
/// any cycles in the process.
#[derive(Copy, Clone, Debug)]
pub struct Minimums {
    pub positive: DepthFirstNumber,
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
