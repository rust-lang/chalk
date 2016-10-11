//! Compiled formulas. These are based on the user syntax given in
//! `chalk_parse::ast`, but normalized and converted to use debruijn
//! indices.

pub use lalrpop_intern::InternedString;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Formula<C> {
    data: Rc<FormulaData<C>>,
}

deref_to!(Formula<C>.data => FormulaData<C>);

#[derive(Clone, Debug)]
pub struct FormulaData<C> {
    pub kind: FormulaKind<C>,
}

#[derive(Clone, Debug)]
pub enum FormulaKind<C> {
    Leaf(Leaf<C>),
    Implication(Leaf<C>, Formula<C>),
    Exists(Formula<C>),
    ForAll(Formula<C>),
    And(Vec<Formula<C>>),
    Or(Vec<Formula<C>>),
}

#[derive(Clone, Debug)]
pub struct Leaf<C> {
    data: Rc<LeafData<C>>,
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
