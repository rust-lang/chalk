// These impls for PartialEq, Eq, etc are written by hand. This is
// because the `#[derive()]` would add requirements onto the context
// object that are not needed.

use std::cmp::{PartialEq, Eq};
use std::hash::{Hash, Hasher};
use std::mem;
use super::*;

impl<C: Context> PartialEq for DelayedLiteralSet<C> {
    fn eq(&self, other: &Self) -> bool {
        let DelayedLiteralSet { delayed_literals: a1 } = self;
        let DelayedLiteralSet { delayed_literals: a2 } = other;
        a1 == a2
    }
}

impl<C: Context> Eq for DelayedLiteralSet<C> {
}

///////////////////////////////////////////////////////////////////////////

impl<C: Context> PartialEq for DelayedLiteral<C> {
    fn eq(&self, other: &Self) -> bool {
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }

        match (self, other) {
            (DelayedLiteral::CannotProve(()), DelayedLiteral::CannotProve(())) =>
                true,

            (DelayedLiteral::Negative(a1), DelayedLiteral::Negative(a2)) =>
                a1 == a2,

            (DelayedLiteral::Positive(a1, b1), DelayedLiteral::Positive(a2, b2)) =>
                a1 == a2 && b1 == b2,

            _ => panic!()
        }
    }
}

impl<C: Context> Eq for DelayedLiteral<C> {
}

impl<C: Context> Hash for DelayedLiteral<C> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        mem::discriminant(self).hash(hasher);

        match self {
            DelayedLiteral::CannotProve(()) => (),

            DelayedLiteral::Negative(a) => {
                a.hash(hasher);
            }

            DelayedLiteral::Positive(a, b) => {
                a.hash(hasher);
                b.hash(hasher);
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////

impl<C: Context, I: ExClauseContext<C>> PartialEq for Literal<C, I> {
    fn eq(&self, other: &Literal<C, I>) -> bool {
        match (self, other) {
            (Literal::Positive(goal1), Literal::Positive(goal2))
            | (Literal::Negative(goal1), Literal::Negative(goal2)) => goal1 == goal2,

            _ => false,
        }
    }
}

impl<C: Context, I: ExClauseContext<C>> Eq for Literal<C, I> {
}

impl<C: Context, I: ExClauseContext<C>> Hash for Literal<C, I> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);
        match self {
            Literal::Positive(goal) | Literal::Negative(goal) => {
                goal.hash(state);
            }
        }
    }
}

