//! This module contains "rote and uninteresting" impls of `Fold` for
//! various types. In general, we prefer to derive `Fold`, but
//! sometimes that doesn't work for whatever reason.
//!
//! The more interesting impls of `Fold` remain in the `fold` module.

use crate::interner::TargetInterner;
use crate::*;
use std::marker::PhantomData;
use std::sync::Arc;

impl<'a, T: Fold<I, TI>, I: Interner, TI: TargetInterner<I>> Fold<I, TI> for &'a T {
    type Result = T::Result;
    fn fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i,
    {
        (**self).fold_with(folder, outer_binder)
    }
}

impl<T: Fold<I, TI>, I: Interner, TI: TargetInterner<I>> Fold<I, TI> for Vec<T> {
    type Result = Vec<T::Result>;
    fn fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i,
    {
        self.iter()
            .map(|e| e.fold_with(folder, outer_binder))
            .collect()
    }
}

impl<T: Fold<I, TI>, I: Interner, TI: TargetInterner<I>> Fold<I, TI> for Box<T> {
    type Result = Box<T::Result>;
    fn fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i,
    {
        Ok(Box::new((**self).fold_with(folder, outer_binder)?))
    }
}

impl<T: Fold<I, TI>, I: Interner, TI: TargetInterner<I>> Fold<I, TI> for Arc<T> {
    type Result = Arc<T::Result>;
    fn fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i,
    {
        Ok(Arc::new((**self).fold_with(folder, outer_binder)?))
    }
}

macro_rules! tuple_fold {
    ($($n:ident),*) => {
        impl<$($n: Fold<I, TI>,)* I: Interner, TI: TargetInterner<I>> Fold<I, TI> for ($($n,)*) {
            type Result = ($($n::Result,)*);
            fn fold_with<'i>(&self, folder: &mut dyn Folder<'i, I, TI>, outer_binder: DebruijnIndex) -> Fallible<Self::Result>
            where
                I: 'i,
                TI: 'i,
            {
                #[allow(non_snake_case)]
                let &($(ref $n),*) = self;
                Ok(($($n.fold_with(folder, outer_binder)?,)*))
            }
        }
    }
}

tuple_fold!(A, B);
tuple_fold!(A, B, C);
tuple_fold!(A, B, C, D);
tuple_fold!(A, B, C, D, E);

impl<T: Fold<I, TI>, I: Interner, TI: TargetInterner<I>> Fold<I, TI> for Option<T> {
    type Result = Option<T::Result>;
    fn fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i,
    {
        match self {
            None => Ok(None),
            Some(e) => Ok(Some(e.fold_with(folder, outer_binder)?)),
        }
    }
}

impl<I: Interner, TI: TargetInterner<I>> Fold<I, TI> for GenericArg<I> {
    type Result = GenericArg<TI>;
    fn fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i,
    {
        let interner = folder.interner();
        let target_interner = folder.target_interner();

        let data = self.data(interner).fold_with(folder, outer_binder)?;
        Ok(GenericArg::new(target_interner, data))
    }
}

impl<I: Interner, TI: TargetInterner<I>> Fold<I, TI> for Substitution<I> {
    type Result = Substitution<TI>;
    fn fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i,
    {
        let interner = folder.interner();
        let target_interner = folder.target_interner();
        let folded = self
            .iter(interner)
            .map(|p| p.fold_with(folder, outer_binder));
        Ok(Substitution::from_fallible(target_interner, folded)?)
    }
}

impl<I: Interner, TI: TargetInterner<I>> Fold<I, TI> for Goals<I> {
    type Result = Goals<TI>;
    fn fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i,
    {
        let interner = folder.interner();
        let target_interner = folder.target_interner();
        let folded = self
            .iter(interner)
            .map(|p| p.fold_with(folder, outer_binder));
        Ok(Goals::from_fallible(target_interner, folded)?)
    }
}

impl<I: Interner, TI: TargetInterner<I>> Fold<I, TI> for ProgramClauses<I> {
    type Result = ProgramClauses<TI>;
    fn fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i,
    {
        let interner = folder.interner();
        let target_interner = folder.target_interner();
        let folded = self
            .iter(interner)
            .map(|p| p.fold_with(folder, outer_binder));
        Ok(ProgramClauses::from_fallible(target_interner, folded)?)
    }
}

impl<I: Interner, TI: TargetInterner<I>> Fold<I, TI> for QuantifiedWhereClauses<I> {
    type Result = QuantifiedWhereClauses<TI>;
    fn fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i,
    {
        let interner = folder.interner();
        let target_interner = folder.target_interner();
        let folded = self
            .iter(interner)
            .map(|p| p.fold_with(folder, outer_binder));
        Ok(QuantifiedWhereClauses::from_fallible(
            target_interner,
            folded,
        )?)
    }
}

#[macro_export]
macro_rules! copy_fold {
    ($t:ty) => {
        impl<I: Interner, TI: TargetInterner<I>> $crate::fold::Fold<I, TI> for $t {
            type Result = Self;
            fn fold_with<'i>(
                &self,
                _folder: &mut dyn ($crate::fold::Folder<'i, I, TI>),
                _outer_binder: DebruijnIndex,
            ) -> ::chalk_ir::Fallible<Self::Result>
            where
                I: 'i,
                TI: 'i,
            {
                Ok(*self)
            }
        }
    };
}

copy_fold!(bool);
copy_fold!(usize);
copy_fold!(UniverseIndex);
copy_fold!(PlaceholderIndex);
copy_fold!(QuantifierKind);
copy_fold!(DebruijnIndex);
copy_fold!(());
copy_fold!(UintTy);
copy_fold!(IntTy);
copy_fold!(FloatTy);
copy_fold!(Scalar);
copy_fold!(ClausePriority);
copy_fold!(Mutability);

#[macro_export]
macro_rules! id_fold {
    ($t:ident) => {
        $crate::id_fold!($t, transfer_def_id);
    };

    ($t:ident, $transfer_fn:ident) => {
        impl<I: Interner, TI: TargetInterner<I>> $crate::fold::Fold<I, TI> for $t<I> {
            type Result = $t<TI>;
            fn fold_with<'i>(
                &self,
                _folder: &mut dyn ($crate::fold::Folder<'i, I, TI>),
                _outer_binder: DebruijnIndex,
            ) -> ::chalk_ir::Fallible<Self::Result>
            where
                I: 'i,
                TI: 'i,
            {
                let $t(def_id_tf) = *self;
                let def_id_ttf = TI::$transfer_fn(def_id_tf);
                Ok($t(def_id_ttf))
            }
        }
    };
}

id_fold!(ImplId);
id_fold!(AdtId, transfer_adt_id);
id_fold!(TraitId);
id_fold!(AssocTypeId);
id_fold!(OpaqueTyId);
id_fold!(FnDefId);
id_fold!(ClosureId);

impl<I: Interner, TI: TargetInterner<I>> SuperFold<I, TI> for ProgramClauseData<I> {
    fn super_fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        outer_binder: DebruijnIndex,
    ) -> ::chalk_ir::Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i,
    {
        Ok(ProgramClauseData(self.0.fold_with(folder, outer_binder)?))
    }
}

impl<I: Interner, TI: TargetInterner<I>> SuperFold<I, TI> for ProgramClause<I> {
    fn super_fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        outer_binder: DebruijnIndex,
    ) -> ::chalk_ir::Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i,
    {
        let clause = self.data(folder.interner());
        Ok(clause
            .super_fold_with(folder, outer_binder)?
            .intern(folder.target_interner()))
    }
}

impl<I: Interner, TI: TargetInterner<I>> Fold<I, TI> for PhantomData<I> {
    type Result = PhantomData<TI>;

    fn fold_with<'i>(
        &self,
        _folder: &mut dyn Folder<'i, I, TI>,
        _outer_binder: DebruijnIndex,
    ) -> ::chalk_ir::Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i,
    {
        Ok(PhantomData)
    }
}
