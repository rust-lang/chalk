pub use lalrpop_intern::InternedString;
use std::ops::Deref;
use std::rc::Rc;

macro_rules! deref_to {
    ($source:ident<$($param:ident),*>.$field:ident => $target:ty) => {
        impl<$($param),*> Deref for $source<$($param),*> {
            type Target = $target;

            fn deref(&self) -> &$target {
                &self.$field
            }
        }
    }
}

pub struct Formula<C> {
    data: Rc<FormulaData<C>>,
}

deref_to!(Formula<C>.data => FormulaData<C>);

pub struct FormulaData<C> {
    pub kind: FormulaKind<C>,
}

pub enum FormulaKind<C> {
    Leaf(Leaf<C>),
    Implication(Leaf<C>, Formula<C>),
    Exists(Formula<C>),
    ForAll(Formula<C>),
}

pub struct Leaf<C> {
    data: Rc<LeafData<C>>,
}

deref_to!(Leaf<C>.data => LeafData<C>);

pub struct LeafData<C> {
    pub kind: LeafKind<C>,
}

pub enum LeafKind<C> {
    Wildcard, // _
    BoundVariable(BoundVariable), // X
    Constant(Constant<C>), // C(...)
}

// C(...)
pub struct Constant<C> {
    pub data: C,
    pub args: Vec<Leaf<C>>,
}

// X
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
