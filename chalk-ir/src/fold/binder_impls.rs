//! This module contains impls of `Fold` for those types that
//! introduce binders.
//!
//! The more interesting impls of `Fold` remain in the `fold` module.

use crate::interner::TargetInterner;
use crate::*;

impl<I: Interner, TI: TargetInterner<I>> Fold<I, TI> for Fn<I> {
    type Result = Fn<TI>;
    fn fold_with(&self, folder: &mut dyn Folder<I, TI>, binders: usize) -> Fallible<Self::Result> {
        let Fn {
            num_binders,
            ref parameters,
        } = *self;
        Ok(Fn {
            num_binders,
            parameters: parameters.fold_with(folder, binders + num_binders)?,
        })
    }
}

impl<T, I: Interner, TI: TargetInterner<I>> Fold<I, TI> for Binders<T>
where
    T: Fold<I, TI>,
    I: Interner,
{
    type Result = Binders<T::Result>;
    fn fold_with(&self, folder: &mut dyn Folder<I, TI>, binders: usize) -> Fallible<Self::Result> {
        let Binders {
            binders: ref self_binders,
            value: ref self_value,
        } = *self;
        let value = self_value.fold_with(folder, binders + self_binders.len())?;
        Ok(Binders {
            binders: self_binders.clone(),
            value: value,
        })
    }
}

impl<T, I, TI> Fold<I, TI> for Canonical<T>
where
    T: Fold<I, TI>,
    I: Interner,
    TI: TargetInterner<I>,
{
    type Result = Canonical<T::Result>;
    fn fold_with(&self, folder: &mut dyn Folder<I, TI>, binders: usize) -> Fallible<Self::Result> {
        let Canonical {
            binders: ref self_binders,
            value: ref self_value,
        } = *self;
        let value = self_value.fold_with(folder, binders + self_binders.len())?;
        Ok(Canonical {
            binders: self_binders.clone(),
            value: value,
        })
    }
}
