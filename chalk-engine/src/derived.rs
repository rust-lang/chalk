// These impls for PartialEq, Eq, etc are written by hand. This is
// because the `#[derive()]` would add requirements onto the context
// object that are not needed.

use super::*;
use std::cmp::{Eq, PartialEq};
use std::hash::{Hash, Hasher};
use std::mem;

///////////////////////////////////////////////////////////////////////////

impl<I: Interner> PartialEq for Literal<I> {
    fn eq(&self, other: &Literal<I>) -> bool {
        match (self, other) {
            (Literal::Positive(goal1), Literal::Positive(goal2))
            | (Literal::Negative(goal1), Literal::Negative(goal2)) => goal1 == goal2,

            _ => false,
        }
    }
}

impl<I: Interner> Eq for Literal<I> {}

impl<I: Interner> Hash for Literal<I> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);
        match self {
            Literal::Positive(goal) | Literal::Negative(goal) => {
                goal.hash(state);
            }
        }
    }
}
