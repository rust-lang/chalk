pub mod environment;
pub mod infer;
pub mod implemented;
pub mod implemented_with;
pub mod solver;

pub struct Solution<G> {
    successful: Successful,
    refined_goal: G
}

pub enum Successful {
    Yes,
    Maybe,
}
