pub mod environment;
pub mod infer;
pub mod implemented;
pub mod implemented_with;
pub mod solver;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Solution<G> {
    successful: Successful,
    refined_goal: G
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Successful {
    Yes,
    Maybe,
}
