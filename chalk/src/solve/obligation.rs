use formula::*;
use solve::*;
use std::sync::Arc;

pub struct Obligation {
    pub environment: Arc<Environment>,
    pub goal: Goal<Leaf>,
    pub universe_index: UniverseIndex,
}

