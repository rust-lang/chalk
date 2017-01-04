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

impl<G> Solution<G> {
    pub fn map<OP, H>(self, op: OP) -> Solution<H>
        where OP: FnOnce(G) -> H
    {
        Solution {
            successful: self.successful,
            refined_goal: op(self.refined_goal),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Successful {
    Yes,
    Maybe,
}

impl Successful {
    pub fn and(self, s: Successful) -> Successful {
        use self::Successful::*;
        match (self, s) {
            (Yes, Yes) => Yes,
            (Maybe, _) | (_, Maybe) => Maybe,
        }
    }

    pub fn or(self, s: Successful) -> Successful {
        use self::Successful::*;
        match (self, s) {
            (Maybe, Maybe) => Maybe,
            (Yes, _) | (_, Yes) => Yes,
        }
    }
}
