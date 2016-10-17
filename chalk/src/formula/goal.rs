use super::common::*;
use super::clause::Clause;
use std::sync::Arc;

/// G-formula, see page 75 of Programming with Higher-Order Logic.
#[derive(Clone, Debug)]
pub struct Goal<C> {
    data: Arc<GoalData<C>>,
}

deref_to!(Goal<C>.data => GoalData<C>);

impl<C> Goal<C> {
    pub fn new(data: GoalData<C>) -> Self {
        Goal { data: Arc::new(data) }
    }
}

#[derive(Clone, Debug)]
pub struct GoalData<C> {
    pub kind: GoalKind<C>,
}

#[derive(Clone, Debug)]
pub enum GoalKind<C> {
    True,
    Leaf(Leaf<C>),
    And(Vec<Goal<C>>),
    Or(Vec<Goal<C>>),
    Exists(Quantification<Goal<C>>),
    Implication(Clause<C>, Goal<C>),
    ForAll(Quantification<Goal<C>>),
}

macro_rules! goal {
    (true) => { Goal::new(GoalData { kind: GoalKind::True }) };
}
