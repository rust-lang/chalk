//! This module contains impls of `TypeVisitable` for those types that
//! introduce binders.
//!
//! The more interesting impls of `TypeVisitable` remain in the `visit` module.

use crate::interner::HasInterner;
use crate::{
    Binders, Canonical, ControlFlow, DebruijnIndex, FnPointer, Interner, TypeVisitable, TypeVisitor,
};

impl<I: Interner> TypeVisitable<I> for FnPointer<I> {
    fn visit_with<B>(
        &self,
        visitor: &mut dyn TypeVisitor<I, BreakTy = B>,
        outer_binder: DebruijnIndex,
    ) -> ControlFlow<B> {
        self.substitution
            .visit_with(visitor, outer_binder.shifted_in())
    }
}

impl<T, I: Interner> TypeVisitable<I> for Binders<T>
where
    T: HasInterner + TypeVisitable<I>,
{
    fn visit_with<B>(
        &self,
        visitor: &mut dyn TypeVisitor<I, BreakTy = B>,
        outer_binder: DebruijnIndex,
    ) -> ControlFlow<B> {
        self.value.visit_with(visitor, outer_binder.shifted_in())
    }
}

impl<I, T> TypeVisitable<I> for Canonical<T>
where
    I: Interner,
    T: HasInterner<Interner = I> + TypeVisitable<I>,
{
    fn visit_with<B>(
        &self,
        visitor: &mut dyn TypeVisitor<I, BreakTy = B>,
        outer_binder: DebruijnIndex,
    ) -> ControlFlow<B> {
        self.value.visit_with(visitor, outer_binder.shifted_in())
    }
}
