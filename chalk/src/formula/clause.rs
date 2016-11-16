use super::quant::Quantification;
use super::Goal;
use std::sync::Arc;

/// D-formula, see page 75 of Programming with Higher-Order Logic.
#[derive(Clone, PartialEq, Eq)]
pub struct Clause<L> {
    data: Arc<ClauseData<L>>,
}

pub type ClauseData<L> = Quantification<ClauseImplication<L>>;

deref_to!(Clause<L>.data => ClauseData<L>);

impl<L> Clause<L> {
    pub fn new(data: ClauseData<L>) -> Self {
        Clause { data: Arc::new(data) }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ClauseImplication<L> {
    pub condition: Option<Goal<L>>, // if None, implies True
    pub consequence: L,
}

