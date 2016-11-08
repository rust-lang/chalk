use super::quant::Quantification;
use super::clause::Clause;
use std::sync::Arc;

/// G-formula, see page 75 of Programming with Higher-Order Logic.
#[derive(PartialEq, Eq)]
pub struct Goal<L> {
    data: Arc<GoalData<L>>,
}

impl<L> Clone for Goal<L> {
    fn clone(&self) -> Self {
        Goal { data: self.data.clone() }
    }
}

deref_to!(Goal<L>.data => GoalData<L>);

impl<L> Goal<L> {
    pub fn new(data: GoalData<L>) -> Self {
        Goal { data: Arc::new(data) }
    }

    pub fn in_foralls(self, num_binders: usize) -> Goal<L> {
        if num_binders == 0 {
            self
        } else {
            Goal::new(GoalData {
                kind: GoalKind::ForAll(Quantification {
                    num_binders: num_binders,
                    formula: self,
                }),
            })
        }
    }

    pub fn in_exists(self, num_binders: usize) -> Goal<L> {
        if num_binders == 0 {
            self
        } else {
            Goal::new(GoalData {
                kind: GoalKind::Exists(Quantification {
                    num_binders: num_binders,
                    formula: self,
                }),
            })
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct GoalData<L> {
    pub kind: GoalKind<L>,
}

#[derive(Clone, PartialEq, Eq)]
pub enum GoalKind<L> {
    True,
    Leaf(L),
    And(Goal<L>, Goal<L>),
    Or(Goal<L>, Goal<L>),
    Exists(Quantification<Goal<L>>),
    Implication(Clause<L>, Goal<L>),
    ForAll(Quantification<Goal<L>>),
}

macro_rules! goal {
    (true) => { Goal::new(GoalData { kind: GoalKind::True }) };
}
