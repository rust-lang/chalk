// These impls for PartialEq, Eq, etc are written by hand. This is
// because the `#[derive()]` would add requirements onto the context
// object that are not needed.

use super::*;
use std::cmp::{Eq, PartialEq};
use std::hash::{Hash, Hasher};
use std::mem;

///////////////////////////////////////////////////////////////////////////

impl<C: Context> PartialEq for Literal<C> {
    fn eq(&self, other: &Literal<C>) -> bool {
        match (self, other) {
            (Literal::Positive(goal1), Literal::Positive(goal2))
            | (Literal::Negative(goal1), Literal::Negative(goal2)) => goal1 == goal2,

            _ => false,
        }
    }
}

impl<C: Context> Eq for Literal<C> {}

impl<C: Context> Hash for Literal<C> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);
        match self {
            Literal::Positive(goal) | Literal::Negative(goal) => {
                goal.hash(state);
            }
        }
    }
}
