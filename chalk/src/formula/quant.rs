/// Something like `forall(x, y, z -> F)`.
#[derive(Clone, PartialEq, Eq)]
pub struct Quantification<F> {
    /// Number of bound variables introduced. (3 in the above example.)
    pub num_binders: usize,

    /// The quantified formula (`F` in the above example.)
    ///
    /// This is private because accessing it is error-prone. Use
    /// `skip_binders()` for easier audit.
    formula: F,
}

impl<F> Quantification<F> {
    pub fn new(num_binders: usize, formula: F) -> Self {
        Quantification {
            num_binders: num_binders,
            formula: formula,
        }
    }

    /// Get the raw formula, disregarding the binders. Generally a
    /// warning sign that you're going to do some of the deBruijn
    /// arithmetic wrong.
    pub fn skip_binders(&self) -> &F {
        &self.formula
    }

    pub fn map_bound<OP>(&self, op: OP) -> Self
        where OP: FnOnce(&F) -> F
    {
        Quantification {
            num_binders: self.num_binders,
            formula: op(self.skip_binders())
        }
    }
}


