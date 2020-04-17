//! This module contains impls of `Visit` for those types that
//! introduce binders.
//!
//! The more interesting impls of `Visit` remain in the `visit` module.

use crate::interner::HasInterner;
use crate::{Binders, Canonical, DebruijnIndex, Fn, Interner, Visit, VisitResult, Visitor};

impl<I: Interner> Visit<I> for Fn<I> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        let interner = visitor.interner();
        self.substitution
            .parameters(interner)
            .visit_with(visitor, outer_binder.shifted_in())
    }
}

impl<T, I: Interner> Visit<I> for Binders<T>
where
    T: HasInterner + Visit<I>,
{
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        self.value.visit_with(visitor, outer_binder.shifted_in())
    }
}

impl<I, T> Visit<I> for Canonical<T>
where
    I: Interner,
    T: HasInterner<Interner = I> + Visit<I>,
{
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        self.value.visit_with(visitor, outer_binder.shifted_in())
    }
}
