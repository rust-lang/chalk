//! This module contains impls of `Visit` for those types that
//! introduce binders.
//!
//! The more interesting impls of `Visit` remain in the `visit` module.

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
        self.parameters
            .visit_with(visitor, outer_binder.shifted_in())
    }
}

impl<T, I: Interner> Visit<I> for Binders<T>
where
    T: Visit<I>,
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

impl<T, I> Visit<I> for Canonical<T>
where
    T: Visit<I>,
    I: Interner,
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
