//! This module contains "rote and uninteresting" impls of `Fold` for
//! various types. In general, we prefer to derive `Fold`, but
//! sometimes that doesn't work for whatever reason.
//!
//! The more interesting impls of `Fold` remain in the `fold` module.

use crate::interner::TargetInterner;
use crate::*;
use chalk_engine::context::Context;
use chalk_engine::{ExClause, FlounderedSubgoal, Literal};
use std::marker::PhantomData;
use std::sync::Arc;

impl<'a, T: Fold<I, TI>, I: Interner, TI: TargetInterner<I>> Fold<I, TI> for &'a T {
    type Result = T::Result;
    fn fold_with(&self, folder: &mut dyn Folder<I, TI>, binders: usize) -> Fallible<Self::Result> {
        (**self).fold_with(folder, binders)
    }
}

impl<T: Fold<I, TI>, I: Interner, TI: TargetInterner<I>> Fold<I, TI> for Vec<T> {
    type Result = Vec<T::Result>;
    fn fold_with(&self, folder: &mut dyn Folder<I, TI>, binders: usize) -> Fallible<Self::Result> {
        self.iter().map(|e| e.fold_with(folder, binders)).collect()
    }
}

impl<T: Fold<I, TI>, I: Interner, TI: TargetInterner<I>> Fold<I, TI> for Box<T> {
    type Result = Box<T::Result>;
    fn fold_with(&self, folder: &mut dyn Folder<I, TI>, binders: usize) -> Fallible<Self::Result> {
        Ok(Box::new((**self).fold_with(folder, binders)?))
    }
}

impl<T: Fold<I, TI>, I: Interner, TI: TargetInterner<I>> Fold<I, TI> for Arc<T> {
    type Result = Arc<T::Result>;
    fn fold_with(&self, folder: &mut dyn Folder<I, TI>, binders: usize) -> Fallible<Self::Result> {
        Ok(Arc::new((**self).fold_with(folder, binders)?))
    }
}

macro_rules! tuple_fold {
    ($($n:ident),*) => {
        impl<$($n: Fold<I, TI>,)* I: Interner, TI: TargetInterner<I>> Fold<I, TI> for ($($n,)*) {
            type Result = ($($n::Result,)*);
            fn fold_with(&self, folder: &mut dyn Folder<I, TI>, binders: usize) -> Fallible<Self::Result> {
                #[allow(non_snake_case)]
                let &($(ref $n),*) = self;
                Ok(($($n.fold_with(folder, binders)?,)*))
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
    fn fold_with(&self, folder: &mut dyn Folder<I, TI>, binders: usize) -> Fallible<Self::Result> {
        match self {
            None => Ok(None),
            Some(e) => Ok(Some(e.fold_with(folder, binders)?)),
        }
    }
}

impl<I: Interner, TI: TargetInterner<I>> Fold<I, TI> for Parameter<I> {
    type Result = Parameter<TI>;
    fn fold_with(&self, folder: &mut dyn Folder<I, TI>, binders: usize) -> Fallible<Self::Result> {
        let data = self.data().fold_with(folder, binders)?;
        Ok(Parameter::new(folder.target_interner(), data))
    }
}

impl<I: Interner, TI: TargetInterner<I>> Fold<I, TI> for Substitution<I> {
    type Result = Substitution<TI>;
    fn fold_with(&self, folder: &mut dyn Folder<I, TI>, binders: usize) -> Fallible<Self::Result> {
        let folded = self.iter().map(|p| p.fold_with(folder, binders));
        Ok(Substitution::from_fallible(
            folder.target_interner(),
            folded,
        )?)
    }
}

impl<I: Interner, TI: TargetInterner<I>> Fold<I, TI> for Goals<I> {
    type Result = Goals<TI>;
    fn fold_with(&self, folder: &mut dyn Folder<I, TI>, binders: usize) -> Fallible<Self::Result> {
        let folded = self.iter().map(|p| p.fold_with(folder, binders));
        Ok(Goals::from_fallible(folder.target_interner(), folded)?)
    }
}

#[macro_export]
macro_rules! copy_fold {
    ($t:ty) => {
        impl<I: Interner, TI: TargetInterner<I>> $crate::fold::Fold<I, TI> for $t {
            type Result = Self;
            fn fold_with(
                &self,
                _folder: &mut dyn ($crate::fold::Folder<I, TI>),
                _binders: usize,
            ) -> ::chalk_engine::fallible::Fallible<Self::Result> {
                Ok(*self)
            }
        }
    };
}

copy_fold!(UniverseIndex);
copy_fold!(usize);
copy_fold!(PlaceholderIndex);
copy_fold!(QuantifierKind);
copy_fold!(chalk_engine::TableIndex);
copy_fold!(chalk_engine::TimeStamp);
copy_fold!(());

#[macro_export]
macro_rules! id_fold {
    ($t:ident) => {
        impl<I: Interner, TI: TargetInterner<I>> $crate::fold::Fold<I, TI> for $t<I> {
            type Result = $t<TI>;
            fn fold_with(
                &self,
                _folder: &mut dyn ($crate::fold::Folder<I, TI>),
                _binders: usize,
            ) -> ::chalk_engine::fallible::Fallible<Self::Result> {
                let $t(def_id_tf) = *self;
                let def_id_ttf = TI::transfer_def_id(def_id_tf);
                Ok($t(def_id_ttf))
            }
        }
    };
}

id_fold!(ImplId);
id_fold!(StructId);
id_fold!(TraitId);
id_fold!(AssocTypeId);

impl<I: Interner, TI: TargetInterner<I>> SuperFold<I, TI> for ProgramClause<I> {
    fn super_fold_with(
        &self,
        folder: &mut dyn Folder<I, TI>,
        binders: usize,
    ) -> ::chalk_engine::fallible::Fallible<Self::Result> {
        match self {
            ProgramClause::Implies(pci) => {
                Ok(ProgramClause::Implies(pci.fold_with(folder, binders)?))
            }
            ProgramClause::ForAll(pci) => {
                Ok(ProgramClause::ForAll(pci.fold_with(folder, binders)?))
            }
        }
    }
}

impl<I: Interner, TI: TargetInterner<I>> Fold<I, TI> for PhantomData<I> {
    type Result = PhantomData<TI>;

    fn fold_with(
        &self,
        _folder: &mut dyn Folder<I, TI>,
        _binders: usize,
    ) -> ::chalk_engine::fallible::Fallible<Self::Result> {
        Ok(PhantomData)
    }
}

impl<I: Interner, TI: TargetInterner<I>, T, L> Fold<I, TI> for ParameterKind<T, L>
where
    T: Fold<I, TI>,
    L: Fold<I, TI>,
{
    type Result = ParameterKind<T::Result, L::Result>;

    fn fold_with(&self, folder: &mut dyn Folder<I, TI>, binders: usize) -> Fallible<Self::Result> {
        match self {
            ParameterKind::Ty(a) => Ok(ParameterKind::Ty(a.fold_with(folder, binders)?)),
            ParameterKind::Lifetime(a) => {
                Ok(ParameterKind::Lifetime(a.fold_with(folder, binders)?))
            }
        }
    }
}

impl<C: Context, I: Interner, TI: TargetInterner<I>> Fold<I, TI> for ExClause<C>
where
    C: Context,
    C::Substitution: Fold<I, TI, Result = C::Substitution>,
    C::RegionConstraint: Fold<I, TI, Result = C::RegionConstraint>,
    C::CanonicalConstrainedSubst: Fold<I, TI, Result = C::CanonicalConstrainedSubst>,
    C::GoalInEnvironment: Fold<I, TI, Result = C::GoalInEnvironment>,
{
    type Result = ExClause<C>;

    fn fold_with(&self, folder: &mut dyn Folder<I, TI>, binders: usize) -> Fallible<Self::Result> {
        let ExClause {
            subst,
            ambiguous,
            constraints,
            subgoals,
            delayed_subgoals,
            answer_time,
            floundered_subgoals,
        } = self;
        Ok(ExClause {
            subst: subst.fold_with(folder, binders)?,
            ambiguous: *ambiguous,
            constraints: constraints.fold_with(folder, binders)?,
            subgoals: subgoals.fold_with(folder, binders)?,
            delayed_subgoals: delayed_subgoals.fold_with(folder, binders)?,
            answer_time: answer_time.fold_with(folder, binders)?,
            floundered_subgoals: floundered_subgoals.fold_with(folder, binders)?,
        })
    }
}

impl<C: Context, I: Interner, TI: TargetInterner<I>> Fold<I, TI> for FlounderedSubgoal<C>
where
    C: Context,
    C::Substitution: Fold<I, TI, Result = C::Substitution>,
    C::RegionConstraint: Fold<I, TI, Result = C::RegionConstraint>,
    C::CanonicalConstrainedSubst: Fold<I, TI, Result = C::CanonicalConstrainedSubst>,
    C::GoalInEnvironment: Fold<I, TI, Result = C::GoalInEnvironment>,
{
    type Result = FlounderedSubgoal<C>;

    fn fold_with(&self, folder: &mut dyn Folder<I, TI>, binders: usize) -> Fallible<Self::Result> {
        let FlounderedSubgoal {
            floundered_literal,
            floundered_time,
        } = self;
        Ok(FlounderedSubgoal {
            floundered_literal: floundered_literal.fold_with(folder, binders)?,
            floundered_time: floundered_time.fold_with(folder, binders)?,
        })
    }
}

impl<C: Context, I: Interner, TI: TargetInterner<I>> Fold<I, TI> for Literal<C>
where
    C: Context,
    C::GoalInEnvironment: Fold<I, TI, Result = C::GoalInEnvironment>,
{
    type Result = Literal<C>;

    fn fold_with(&self, folder: &mut dyn Folder<I, TI>, binders: usize) -> Fallible<Self::Result> {
        match self {
            Literal::Positive(goal) => Ok(Literal::Positive(goal.fold_with(folder, binders)?)),
            Literal::Negative(goal) => Ok(Literal::Negative(goal.fold_with(folder, binders)?)),
        }
    }
}
