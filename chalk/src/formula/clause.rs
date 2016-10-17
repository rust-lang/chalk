use super::common::*;
use super::Goal;
use std::sync::Arc;

/// D-formula, see page 75 of Programming with Higher-Order Logic.
#[derive(Clone, Debug)]
pub struct Clause<C> {
    data: Arc<ClauseData<C>>,
}

deref_to!(Clause<C>.data => ClauseData<C>);

impl<C> Clause<C> {
    pub fn new(data: ClauseData<C>) -> Self {
        Clause { data: Arc::new(data) }
    }
}

#[derive(Clone, Debug)]
pub struct ClauseData<C> {
    pub kind: ClauseKind<C>,
}

#[derive(Clone, Debug)]
pub enum ClauseKind<C> {
    Leaf(Leaf<C>),
    And(Vec<Clause<C>>),
    Implication(Goal<C>, Clause<C>),
    ForAll(Quantification<Clause<C>>),
}

macro_rules! clause {
    (true) => { Clause::new(ClauseData { kind: ClauseKind::True }) };
}
