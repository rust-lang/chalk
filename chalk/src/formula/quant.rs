/// Something like `forall(x, y, z -> F)`.
#[derive(Clone, Debug)]
pub struct Quantification<F> {
    /// Number of bound variables introduced. (3 in the above example.)
    pub num_binders: usize,

    /// The quantified formula (`F` in the above example.)
    pub formula: F,
}

