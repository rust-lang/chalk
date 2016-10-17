use std::sync::Arc;

/// Something like `forall(x, y, z -> F)`.
#[derive(Clone, Debug)]
pub struct Quantification<F> {
    /// Number of bound variables introduced. (3 in the above example.)
    pub num_binders: usize,

    /// The quantified formula (`F` in the above example.)
    pub formula: F,
}

#[derive(Clone, Debug)]
pub struct Leaf<C> {
    data: Arc<LeafData<C>>,
}

deref_to!(Leaf<C>.data => LeafData<C>);

#[derive(Clone, Debug)]
pub struct LeafData<C> {
    pub kind: LeafKind<C>,
}

#[derive(Clone, Debug)]
pub enum LeafKind<C> {
    Wildcard, // _
    BoundVariable(BoundVariable), // X
    Constant(Constant<C>), // C(...)
}

// C(...)
#[derive(Clone, Debug)]
pub struct Constant<C> {
    pub data: C,
    pub args: Vec<Leaf<C>>,
}

// X -- encoded using deBruijn indices
#[derive(Clone, Debug)]
pub struct BoundVariable {
    depth: usize,
}

impl BoundVariable {
    pub fn new(depth: usize) -> Self {
        BoundVariable { depth: depth }
    }

    pub fn depth(&self) -> usize {
        self.depth
    }
}

