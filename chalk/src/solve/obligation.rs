use formula::*;
use solve::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct Obligation {
    pub environment: Arc<Environment>,
    pub goal: Goal<Application>,
}

impl Obligation {
    pub fn new(environment: Arc<Environment>,
               goal: Goal<Application>)
               -> Self {
        Obligation {
            environment: environment,
            goal: goal,
        }
    }
}
