use super::quant::Quantification;
use super::clause::Clause;
use std::sync::Arc;

/// G-formula, see page 75 of Programming with Higher-Order Logic.
#[derive(Clone)]
pub struct Goal<L> {
    data: Arc<GoalData<L>>,
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
                    formula: self
                })
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
                    formula: self
                })
            })
        }
    }
}

#[derive(Clone)]
pub struct GoalData<L> {
    pub kind: GoalKind<L>,
}

#[derive(Clone)]
pub enum GoalKind<L> {
    True,
    Leaf(L),
    And(Vec<Goal<L>>),
    Or(Vec<Goal<L>>),
    Exists(Quantification<Goal<L>>),
    Implication(Clause<L>, Goal<L>),
    ForAll(Quantification<Goal<L>>),
}

macro_rules! goal {
    (true) => { Goal::new(GoalData { kind: GoalKind::True }) };
}
