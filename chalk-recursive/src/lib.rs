use chalk_ir::{Goal, InEnvironment, UCanonical};

pub type UCanonicalGoal<I> = UCanonical<InEnvironment<Goal<I>>>;

mod combine;
mod fixed_point;
mod fulfill;
mod recursive;
pub mod solve;

pub use fixed_point::Cache;
pub use recursive::RecursiveSolver;
