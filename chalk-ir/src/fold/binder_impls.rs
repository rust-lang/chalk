//! This module contains impls of `Fold` for those types that
//! introduce binders.
//!
//! The more interesting impls of `Fold` remain in the `fold` module.

use crate::family::TargetTypeFamily;
use crate::*;

impl<TF: TypeFamily, TTF: TargetTypeFamily<TF>> Fold<TF, TTF> for QuantifiedTy<TF> {
    type Result = QuantifiedTy<TTF>;
    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        let QuantifiedTy {
            num_binders,
            ref ty,
        } = *self;
        Ok(QuantifiedTy {
            num_binders,
            ty: ty.fold_with(folder, binders + num_binders)?,
        })
    }
}

impl<T, TF: TypeFamily, TTF: TargetTypeFamily<TF>> Fold<TF, TTF> for Binders<T>
where
    T: Fold<TF, TTF>,
    TF: TypeFamily,
{
    type Result = Binders<T::Result>;
    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
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

impl<T, TF, TTF> Fold<TF, TTF> for Canonical<T>
where
    T: Fold<TF, TTF>,
    TF: TypeFamily,
    TTF: TargetTypeFamily<TF>,
{
    type Result = Canonical<T::Result>;
    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
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
