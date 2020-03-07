//! This module contains impls of `Fold` for those types that
//! introduce binders.
//!
//! The more interesting impls of `Fold` remain in the `fold` module.

use crate::interner::TargetInterner;
use crate::*;

impl<I: Interner, TI: TargetInterner<I>> Fold<I, TI> for Fn<I> {
    type Result = Fn<TI>;
    fn fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        binders: usize,
    ) -> Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i,
    {
        let Fn {
            num_binders,
            parameters,
        } = self;
        Ok(Fn {
            num_binders: num_binders.clone(),
            parameters: parameters.fold_with(folder, binders + 1)?,
        })
    }
}

impl<T, I: Interner, TI: TargetInterner<I>> Fold<I, TI> for Binders<T>
where
    T: Fold<I, TI>,
    I: Interner,
{
    type Result = Binders<T::Result>;
    fn fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        binders: usize,
    ) -> Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i,
    {
        let Binders {
            binders: self_binders,
            value: self_value,
        } = self;
        let value = self_value.fold_with(folder, binders + 1)?;
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
    fn fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        binders: usize,
    ) -> Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i,
    {
        let Canonical {
            binders: self_binders,
            value: self_value,
        } = self;
        let value = self_value.fold_with(folder, binders + 1)?;
        Ok(Canonical {
            binders: self_binders.clone(),
            value: value,
        })
    }
}
