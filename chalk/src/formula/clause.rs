use super::quant::Quantification;
use super::Goal;
use std::sync::Arc;

/// D-formula, see page 75 of Programming with Higher-Order Logic.
#[derive(Clone)]
pub struct Clause<L> {
    data: Arc<ClauseData<L>>,
}

deref_to!(Clause<L>.data => ClauseData<L>);

impl<L> Clause<L> {
    pub fn new(data: ClauseData<L>) -> Self {
        Clause { data: Arc::new(data) }
    }

    pub fn in_foralls(self, num_binders: usize) -> Clause<L> {
        if num_binders == 0 {
            self
        } else {
            Clause::new(ClauseData {
                kind: ClauseKind::ForAll(Quantification {
                    num_binders: num_binders,
                    formula: self
                })
            })
        }
    }
}

#[derive(Clone)]
pub struct ClauseData<L> {
    pub kind: ClauseKind<L>,
}

#[derive(Clone)]
pub enum ClauseKind<L> {
    Leaf(L),
    And(Vec<Clause<L>>),
    Implication(Goal<L>, Clause<L>),
    ForAll(Quantification<Clause<L>>),
}

macro_rules! clause {
    (true) => { Clause::new(ClauseData { kind: ClauseKind::True }) };
}
