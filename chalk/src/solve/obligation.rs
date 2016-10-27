use formula::*;
use solve::*;
use std::sync::Arc;

pub struct Obligation {
    pub environment: Arc<Environment>,
    pub goal: Goal<Leaf>,
}

impl Obligation {
    pub fn new(environment: Arc<Environment>,
               goal: Goal<Leaf>)
               -> Self {
        Obligation {
            environment: environment,
            goal: goal,
        }
    }
}
