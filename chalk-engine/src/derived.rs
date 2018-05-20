// These impls for PartialEq, Eq, etc are written by hand. This is
// because the `#[derive()]` would add requirements onto the context
// object that are not needed.

use std::cmp::{PartialEq, Eq};
use std::hash::{Hash, Hasher};
use std::mem;
use super::*;

impl<E: ExClauseContext> PartialEq for DelayedLiteralSet<E> {
    fn eq(&self, other: &Self) -> bool {
        let DelayedLiteralSet { delayed_literals: a1 } = self;
        let DelayedLiteralSet { delayed_literals: a2 } = other;
        a1 == a2
    }
}

impl<E: ExClauseContext> Eq for DelayedLiteralSet<E> {
}

///////////////////////////////////////////////////////////////////////////

impl<E: ExClauseContext> PartialEq for DelayedLiteral<E> {
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

impl<E: ExClauseContext> Eq for DelayedLiteral<E> {
}

impl<E: ExClauseContext> Hash for DelayedLiteral<E> {
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

impl<I: ExClauseContext> PartialEq for Literal<I> {
    fn eq(&self, other: &Literal<I>) -> bool {
        match (self, other) {
            (Literal::Positive(goal1), Literal::Positive(goal2))
            | (Literal::Negative(goal1), Literal::Negative(goal2)) => goal1 == goal2,

            _ => false,
        }
    }
}

impl<I: ExClauseContext> Eq for Literal<I> {
}

impl<I: ExClauseContext> Hash for Literal<I> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);
        match self {
            Literal::Positive(goal) | Literal::Negative(goal) => {
                goal.hash(state);
            }
        }
    }
}

