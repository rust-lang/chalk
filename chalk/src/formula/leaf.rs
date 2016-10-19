use lalrpop_intern::InternedString;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Leaf {
    data: Arc<LeafData>,
}

impl Leaf {
    pub fn new(data: LeafData) -> Self {
        Leaf { data: Arc::new(data) }
    }
}

deref_to!(Leaf.data => LeafData);

#[derive(Clone, Debug)]
pub struct LeafData {
    pub kind: LeafKind,
}

#[derive(Clone, Debug)]
pub enum LeafKind {
    Wildcard, // _
    BoundVariable(BoundVariable), // X
    Constant(Constant), // C(...)
}

// C(...)
#[derive(Clone, Debug)]
pub struct Constant {
    pub operator: InternedString,
    pub args: Vec<Leaf>,
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

