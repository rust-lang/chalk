use formula::*;
use super::Environment;

/// Something we want to prove
pub struct Obligation<C> {
    /// number of `forall` binders we have traversed thus far
    quantification_depth: usize,

    /// formula we are trying to prove
    formula: Formula<C>,

    /// facts we can use to prove `formula`
    environment: Environment<C>,
}

