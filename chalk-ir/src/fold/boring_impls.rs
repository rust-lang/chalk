//! This module contains "rote and uninteresting" impls of `TypeFoldable` for
//! various types. In general, we prefer to derive `TypeFoldable`, but
//! sometimes that doesn't work for whatever reason.
//!
//! The more interesting impls of `TypeFoldable` remain in the `fold` module.

use super::in_place;
use crate::*;
use std::marker::PhantomData;

impl<T: TypeFoldable<I>, I: Interner> TypeFoldable<I> for Vec<T> {
    fn try_fold_with<E>(
        self,
        folder: &mut dyn FallibleTypeFolder<I, Error = E>,
        outer_binder: DebruijnIndex,
    ) -> Result<Self, E> {
        in_place::fallible_map_vec(self, |e| e.try_fold_with(folder, outer_binder))
    }
}

impl<T: TypeFoldable<I>, I: Interner> TypeFoldable<I> for Box<T> {
    fn try_fold_with<E>(
        self,
        folder: &mut dyn FallibleTypeFolder<I, Error = E>,
        outer_binder: DebruijnIndex,
    ) -> Result<Self, E> {
        in_place::fallible_map_box(self, |e| e.try_fold_with(folder, outer_binder))
    }
}

macro_rules! tuple_fold {
    ($($n:ident),*) => {
        impl<$($n: TypeFoldable<I>,)* I: Interner> TypeFoldable<I> for ($($n,)*) {
            fn try_fold_with<Error>(self, folder: &mut dyn FallibleTypeFolder<I, Error = Error>, outer_binder: DebruijnIndex) -> Result<Self, Error>
            {
                #[allow(non_snake_case)]
                let ($($n),*) = self;
                Ok(($($n.try_fold_with(folder, outer_binder)?,)*))
            }
        }
    }
}

tuple_fold!(A, B);
tuple_fold!(A, B, C);
tuple_fold!(A, B, C, D);
tuple_fold!(A, B, C, D, E);

impl<T: TypeFoldable<I>, I: Interner> TypeFoldable<I> for Option<T> {
    fn try_fold_with<E>(
        self,
        folder: &mut dyn FallibleTypeFolder<I, Error = E>,
        outer_binder: DebruijnIndex,
    ) -> Result<Self, E> {
        match self {
            None => Ok(None),
            Some(e) => Ok(Some(e.try_fold_with(folder, outer_binder)?)),
        }
    }
}

impl<I: Interner> TypeFoldable<I> for GenericArg<I> {
    fn try_fold_with<E>(
        self,
        folder: &mut dyn FallibleTypeFolder<I, Error = E>,
        outer_binder: DebruijnIndex,
    ) -> Result<Self, E> {
        let interner = folder.interner();

        let data = self
            .data(interner)
            .clone()
            .try_fold_with(folder, outer_binder)?;
        Ok(GenericArg::new(interner, data))
    }
}

impl<I: Interner> TypeFoldable<I> for Substitution<I> {
    fn try_fold_with<E>(
        self,
        folder: &mut dyn FallibleTypeFolder<I, Error = E>,
        outer_binder: DebruijnIndex,
    ) -> Result<Self, E> {
        let interner = folder.interner();

        let folded = self
            .iter(interner)
            .cloned()
            .map(|p| p.try_fold_with(folder, outer_binder));
        Substitution::from_fallible(interner, folded)
    }
}

impl<I: Interner> TypeFoldable<I> for Goals<I> {
    fn try_fold_with<E>(
        self,
        folder: &mut dyn FallibleTypeFolder<I, Error = E>,
        outer_binder: DebruijnIndex,
    ) -> Result<Self, E> {
        let interner = folder.interner();
        let folded = self
            .iter(interner)
            .cloned()
            .map(|p| p.try_fold_with(folder, outer_binder));
        Goals::from_fallible(interner, folded)
    }
}

impl<I: Interner> TypeFoldable<I> for ProgramClauses<I> {
    fn try_fold_with<E>(
        self,
        folder: &mut dyn FallibleTypeFolder<I, Error = E>,
        outer_binder: DebruijnIndex,
    ) -> Result<Self, E> {
        let interner = folder.interner();
        let folded = self
            .iter(interner)
            .cloned()
            .map(|p| p.try_fold_with(folder, outer_binder));
        ProgramClauses::from_fallible(interner, folded)
    }
}

impl<I: Interner> TypeFoldable<I> for QuantifiedWhereClauses<I> {
    fn try_fold_with<E>(
        self,
        folder: &mut dyn FallibleTypeFolder<I, Error = E>,
        outer_binder: DebruijnIndex,
    ) -> Result<Self, E> {
        let interner = folder.interner();
        let folded = self
            .iter(interner)
            .cloned()
            .map(|p| p.try_fold_with(folder, outer_binder));
        QuantifiedWhereClauses::from_fallible(interner, folded)
    }
}

impl<I: Interner> TypeFoldable<I> for Constraints<I> {
    fn try_fold_with<E>(
        self,
        folder: &mut dyn FallibleTypeFolder<I, Error = E>,
        outer_binder: DebruijnIndex,
    ) -> Result<Self, E> {
        let interner = folder.interner();
        let folded = self
            .iter(interner)
            .cloned()
            .map(|p| p.try_fold_with(folder, outer_binder));
        Constraints::from_fallible(interner, folded)
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! copy_fold {
    ($t:ty) => {
        impl<I: Interner> $crate::fold::TypeFoldable<I> for $t {
            fn try_fold_with<E>(
                self,
                _folder: &mut dyn ($crate::fold::FallibleTypeFolder<I, Error = E>),
                _outer_binder: DebruijnIndex,
            ) -> ::std::result::Result<Self, E> {
                Ok(self)
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
copy_fold!(Safety);

#[doc(hidden)]
#[macro_export]
macro_rules! id_fold {
    ($t:ident) => {
        impl<I: Interner> $crate::fold::TypeFoldable<I> for $t<I> {
            fn try_fold_with<E>(
                self,
                _folder: &mut dyn ($crate::fold::FallibleTypeFolder<I, Error = E>),
                _outer_binder: DebruijnIndex,
            ) -> ::std::result::Result<Self, E> {
                Ok(self)
            }
        }
    };
}

id_fold!(ImplId);
id_fold!(AdtId);
id_fold!(TraitId);
id_fold!(AssocTypeId);
id_fold!(OpaqueTyId);
id_fold!(FnDefId);
id_fold!(ClosureId);
id_fold!(CoroutineId);
id_fold!(ForeignDefId);

impl<I: Interner> TypeSuperFoldable<I> for ProgramClauseData<I> {
    fn try_super_fold_with<E>(
        self,
        folder: &mut dyn FallibleTypeFolder<I, Error = E>,
        outer_binder: DebruijnIndex,
    ) -> ::std::result::Result<Self, E> {
        Ok(ProgramClauseData(
            self.0.try_fold_with(folder, outer_binder)?,
        ))
    }
}

impl<I: Interner> TypeSuperFoldable<I> for ProgramClause<I> {
    fn try_super_fold_with<E>(
        self,
        folder: &mut dyn FallibleTypeFolder<I, Error = E>,
        outer_binder: DebruijnIndex,
    ) -> ::std::result::Result<Self, E> {
        let clause = self.data(folder.interner()).clone();
        Ok(clause
            .try_super_fold_with(folder, outer_binder)?
            .intern(folder.interner()))
    }
}

impl<I: Interner> TypeFoldable<I> for PhantomData<I> {
    fn try_fold_with<E>(
        self,
        _folder: &mut dyn FallibleTypeFolder<I, Error = E>,
        _outer_binder: DebruijnIndex,
    ) -> ::std::result::Result<Self, E> {
        Ok(PhantomData)
    }
}
