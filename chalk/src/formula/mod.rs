#![allow(dead_code)]

use intern::InternedString;
use std::rc::Rc;

mod arena;
mod debug;
mod ops;

///////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone)]
pub struct Term {
    index: u32
}

impl Term {
    pub fn from_index(index: usize) -> Term {
        Term { index: index as u32 }
    }

    pub fn index(self) -> usize {
        self.index as usize
    }

    pub fn new(data: TermData) -> Term {
        arena::write(|a| a.push(data))
    }

    pub fn data<FUNC,R>(self, func: FUNC) -> R
        where FUNC: FnOnce(&TermData) -> R
    {
        arena::read(|a| func(a.data(self)))
    }

    pub fn take(self) -> TermData {
        arena::write(|a| a.take(self))
    }

    pub fn replace(self, data: TermData) {
        arena::write(|a| a.replace(self, data))
    }

    pub fn swap<F>(self, func: F)
        where F: FnOnce(TermData) -> TermData
    {
        arena::write(|a| a.swap(self, func))
    }
}

///////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone)]
pub struct DebruijnIndex(pub u32);

#[derive(Clone, Debug)]
pub enum TermData {
    Constant(InternedString),
    FreeVariable(InternedString),
    BoundVariable(DebruijnIndex),
    Lambda(Term),
    Apply(Term, Term),
    Suspension(Box<Suspension>),
}

#[derive(Clone)]
pub struct Suspension {
    pub term: Term,

    /// the environment tracks bindings for some subset of the bound
    /// variables in `term`. It can contain either other terms or
    /// dummies. Other terms are used when there has been a suspended
    /// application:
    ///
    ///    (λ t1) t2 => <t1; 0 => t2; 0>
    ///
    /// Meaning, when we suspend the application of t1 to t2, we are
    /// saying "t1, but replace bound variable 0 with t2". The final 0
    /// represents the lifts value, and it just says "if we encounter
    /// a bound variable with index 1 or higher, leave it alone".
    ///
    /// Dummies result from lifting a λ over the suspension:
    ///
    ///    <λt1; env> => <λ
    ///
    ///
    pub environment: Environment,

    /// "bound" is simply the size of the environment
    pub bound: u32,

    /// "lifts" is the number of binders that we have lifted "over"
    /// the suspension. So, basically, when we change from `<λX>` to
    /// `λ<X>` (where `<X>` represents a suspension, lifts will be
    /// incremented by 1 (and a dummy will be added to the
    /// environment). This is used to adjust the depth of bound
    /// variables that do not appear in the environment.
    ///
    /// Here is an example:
    ///
    ///     λ ((λ #0) #1)        // to start, we suspend the inner application
    ///     λ (<#0 #1; 0 => t2; 0>) // lifts is initially 0
    pub lifts: u32,
}

#[derive(Clone)]
pub struct Environment {
    first_cell: Option<Rc<Cell>>
}

#[derive(Clone)]
pub struct Cell {
    data: CellData,
    next: Environment,
}

#[derive(Clone)]
pub enum CellData {
    Term(Term),
    Dummy(u32),
}
