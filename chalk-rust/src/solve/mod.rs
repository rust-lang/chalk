mod environment;
mod infer;
mod quantify;
mod implemented;
mod implemented_with;

pub struct Solution<G> {
    successful: bool,
    refined_goal: G
}
