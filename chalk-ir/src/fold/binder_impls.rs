//! This module contains impls of `TypeFoldable` for those types that
//! introduce binders.
//!
//! The more interesting impls of `TypeFoldable` remain in the `fold` module.

use crate::*;

impl<I: Interner> TypeFoldable<I> for FnPointer<I> {
    fn try_fold_with<E>(
        self,
        folder: &mut dyn FallibleTypeFolder<I, Error = E>,
        outer_binder: DebruijnIndex,
    ) -> Result<Self, E> {
        let FnPointer {
            num_binders,
            substitution,
            sig,
        } = self;
        Ok(FnPointer {
            num_binders,
            substitution: substitution.try_fold_with(folder, outer_binder.shifted_in())?,
            sig: FnSig {
                abi: sig.abi,
                safety: sig.safety,
                variadic: sig.variadic,
            },
        })
    }
}

impl<T, I: Interner> TypeFoldable<I> for Binders<T>
where
    T: HasInterner<Interner = I> + TypeFoldable<I>,
    I: Interner,
{
    fn try_fold_with<E>(
        self,
        folder: &mut dyn FallibleTypeFolder<I, Error = E>,
        outer_binder: DebruijnIndex,
    ) -> Result<Self, E> {
        let Binders {
            binders: self_binders,
            value: self_value,
        } = self;
        let value = self_value.try_fold_with(folder, outer_binder.shifted_in())?;
        let binders = VariableKinds {
            interned: self_binders.interned().clone(),
        };
        Ok(Binders::new(binders, value))
    }
}

impl<I, T> TypeFoldable<I> for Canonical<T>
where
    I: Interner,
    T: HasInterner<Interner = I> + TypeFoldable<I>,
{
    fn try_fold_with<E>(
        self,
        folder: &mut dyn FallibleTypeFolder<I, Error = E>,
        outer_binder: DebruijnIndex,
    ) -> Result<Self, E> {
        let Canonical {
            binders: self_binders,
            value: self_value,
        } = self;
        let value = self_value.try_fold_with(folder, outer_binder.shifted_in())?;
        let binders = CanonicalVarKinds {
            interned: self_binders.interned().clone(),
        };
        Ok(Canonical { binders, value })
    }
}
