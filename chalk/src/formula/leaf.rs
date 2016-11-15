use infer::InferenceVariable;
use infer::UniverseIndex;
use lalrpop_intern::InternedString;
use std::sync::Arc;

#[derive(Clone, PartialEq, Eq)]
pub struct Leaf {
    data: Arc<LeafData>,
}

impl Leaf {
    pub fn new(data: LeafData) -> Self {
        Leaf { data: Arc::new(data) }
    }
}

deref_to!(Leaf.data => LeafData);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeafData {
    pub kind: LeafKind,
}

#[derive(Clone, PartialEq, Eq)]
pub enum LeafKind {
    BoundVariable(BoundVariable),         // X
    InferenceVariable(InferenceVariable), // ?A
    Application(Application),             // C(...)
}

// C(...)
#[derive(Clone, PartialEq, Eq)]
pub struct Application {
    pub constant: Constant,
    pub args: Vec<Leaf>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Constant {
    // Things appearing in the program; always at universe 0
    Program(InternedString),

    // Constants introduced by a forall; always at universe >0
    Skolemized(UniverseIndex),
}

impl Constant {
    pub fn universe_index(&self) -> UniverseIndex {
        match *self {
            Constant::Program(_) => UniverseIndex { counter: 0 },
            Constant::Skolemized(ui) => ui,
        }
    }
}

// X -- encoded using deBruijn indices
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

