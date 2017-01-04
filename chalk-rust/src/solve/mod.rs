pub mod environment;
pub mod infer;
pub mod implemented;
pub mod implemented_with;
pub mod solver;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Solution<G> {
    successful: Successful,
    refined_goal: G,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Successful {
    Yes,
    Maybe,
}

impl<G> Solution<G> {
    pub fn map_goal<OP, H>(self, op: OP) -> Solution<H>
        where OP: FnOnce(G) -> H
    {
        Solution {
            successful: self.successful,
            refined_goal: op(self.refined_goal),
        }
    }
}
