use formula::*;
use solve::*;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Obligation {
    pub environment: Arc<Environment>,
    pub goal: Goal<Application>,
    pub depth: usize,
}

impl Obligation {
    pub fn new(environment: Arc<Environment>,
               goal: Goal<Application>,
               depth: usize)
               -> Self {
        Obligation {
            environment: environment,
            goal: goal,
            depth: depth,
        }
    }
}
