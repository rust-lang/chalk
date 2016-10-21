use lalrpop_intern::InternedString;
use std::sync::Arc;

#[derive(Clone)]
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

#[derive(Clone)]
pub enum LeafKind {
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
#[derive(Copy, Clone, Debug)]
pub struct BoundVariable {
    pub depth: usize,
}

impl BoundVariable {
    pub fn new(depth: usize) -> Self {
        BoundVariable { depth: depth }
    }

    pub fn depth(&self) -> usize {
        self.depth
    }
}

