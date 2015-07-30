use arena::TypedArena;
use rustc_data_structures::unify;
use std::collections::HashMap;
use std::marker::PhantomData;

pub type Formula<'c> = &'c FormulaKind<'c>;

///////////////////////////////////////////////////////////////////////////
// Memory arena

pub struct FormulaArena<'c> {
    formulas: TypedArena<Formula<'c>>,
    map: HashMap<FormulaKind<'c>, Formula<'c>>,
    formulas: TypedArena<Formula<'c>>,
    map: HashMap<FormulaKind<'c>, Formula<'c>>,
}

struct Intern {
}

impl FormulaArena<'c> {
    fn alloc(formula: FormulaKind<'c>) -> Formula<'c> {
    }
}

///////////////////////////////////////////////////////////////////////////
// Formulas

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum FormulaKind<'c> {
    Term(Term<'c>),
    And(Formula<'c>, Formula<'c>),
    Or(Formula<'c>, Formula<'c>),
    Quantified(Binder<Formula<'c>>),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Binder<T> {
    pub bound: T,
}

///////////////////////////////////////////////////////////////////////////
// Terms

pub type Term<'c> = &'c TermKind<'c>;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum TermKind<'c> {
    Atom(InternedString, &'c Vec<TermKind<'c>>),
    FreeVar(TermVariable),
    BoundVar(TermDepth),
}

///////////////////////////////////////////////////////////////////////////
// Bound variables, calculated using Debruijn indices

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct TermDepth {
    depth: usize
}

///////////////////////////////////////////////////////////////////////////
// Unification variables

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct TermVariable<'c> {
    phantom: PhantomData<&'c ()>,
    index: u32,
}

impl<'c> unify::UnifyKey for TermVariable<'c> {
    type Value = Term<'c>;

    fn index(&self) -> u32 {
        self.index
    }

    fn from_index(&self, i: u32) -> TermVariable<'c> {
        TermVariable { phantom: PhantomData, index: i }
    }

    fn tag(_: Option<TermVariable<'c>>) -> &'static str {
        "TermVariable"
    }
}
