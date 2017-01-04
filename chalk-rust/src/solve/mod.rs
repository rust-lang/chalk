mod environment;
mod infer;
mod implemented;
mod implemented_with;
mod solver;

pub struct Solution<G> {
    successful: Successful,
    refined_goal: G
}

pub enum Successful {
    Yes,
    Maybe,
}
