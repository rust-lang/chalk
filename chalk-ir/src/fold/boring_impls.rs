//! This module contains "rote and uninteresting" impls of `Fold` for
//! various types. In general, we prefer to derive `Fold`, but
//! sometimes that doesn't work for whatever reason.
//!
//! The more interesting impls of `Fold` remain in the `fold` module.

use crate::family::TargetTypeFamily;
use crate::*;
use chalk_engine::context::Context;
use chalk_engine::{ExClause, FlounderedSubgoal, Literal};
use std::marker::PhantomData;
use std::sync::Arc;

impl<'a, T: Fold<TF, TTF>, TF: TypeFamily, TTF: TargetTypeFamily<TF>> Fold<TF, TTF> for &'a T {
    type Result = T::Result;
    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        (**self).fold_with(folder, binders)
    }
}

impl<T: Fold<TF, TTF>, TF: TypeFamily, TTF: TargetTypeFamily<TF>> Fold<TF, TTF> for Vec<T> {
    type Result = Vec<T::Result>;
    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        self.iter().map(|e| e.fold_with(folder, binders)).collect()
    }
}

impl<T: Fold<TF, TTF>, TF: TypeFamily, TTF: TargetTypeFamily<TF>> Fold<TF, TTF> for Box<T> {
    type Result = Box<T::Result>;
    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        Ok(Box::new((**self).fold_with(folder, binders)?))
    }
}

impl<T: Fold<TF, TTF>, TF: TypeFamily, TTF: TargetTypeFamily<TF>> Fold<TF, TTF> for Arc<T> {
    type Result = Arc<T::Result>;
    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        Ok(Arc::new((**self).fold_with(folder, binders)?))
    }
}

macro_rules! tuple_fold {
    ($($n:ident),*) => {
        impl<$($n: Fold<TF, TTF>,)* TF: TypeFamily, TTF: TargetTypeFamily<TF>> Fold<TF, TTF> for ($($n,)*) {
            type Result = ($($n::Result,)*);
            fn fold_with(&self, folder: &mut dyn Folder<TF, TTF>, binders: usize) -> Fallible<Self::Result> {
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

impl<T: Fold<TF, TTF>, TF: TypeFamily, TTF: TargetTypeFamily<TF>> Fold<TF, TTF> for Option<T> {
    type Result = Option<T::Result>;
    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        match self {
            None => Ok(None),
            Some(e) => Ok(Some(e.fold_with(folder, binders)?)),
        }
    }
}
impl<TF: TypeFamily, TTF: TargetTypeFamily<TF>> Fold<TF, TTF> for Parameter<TF> {
    type Result = Parameter<TTF>;
    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        let data = self.data().fold_with(folder, binders)?;
        Ok(Parameter::new(data))
    }
}

#[macro_export]
macro_rules! copy_fold {
    ($t:ty) => {
        impl<TF: TypeFamily, TTF: TargetTypeFamily<TF>> $crate::fold::Fold<TF, TTF> for $t {
            type Result = Self;
            fn fold_with(
                &self,
                _folder: &mut dyn ($crate::fold::Folder<TF, TTF>),
                _binders: usize,
            ) -> ::chalk_engine::fallible::Fallible<Self::Result> {
                Ok(*self)
            }
        }
    };
}

copy_fold!(Identifier);
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
        impl<TF: TypeFamily, TTF: TargetTypeFamily<TF>> $crate::fold::Fold<TF, TTF> for $t<TF> {
            type Result = $t<TTF>;
            fn fold_with(
                &self,
                _folder: &mut dyn ($crate::fold::Folder<TF, TTF>),
                _binders: usize,
            ) -> ::chalk_engine::fallible::Fallible<Self::Result> {
                let $t(def_id_tf) = *self;
                let def_id_ttf = TTF::transfer_def_id(def_id_tf);
                Ok($t(def_id_ttf))
            }
        }
    };
}

id_fold!(ImplId);
id_fold!(StructId);
id_fold!(TraitId);
id_fold!(AssocTypeId);

impl<TF: TypeFamily, TTF: TargetTypeFamily<TF>> Fold<TF, TTF> for PhantomData<TF> {
    type Result = PhantomData<TTF>;

    fn fold_with(
        &self,
        _folder: &mut dyn Folder<TF, TTF>,
        _binders: usize,
    ) -> ::chalk_engine::fallible::Fallible<Self::Result> {
        Ok(PhantomData)
    }
}

impl<TF: TypeFamily, TTF: TargetTypeFamily<TF>, T, L> Fold<TF, TTF> for ParameterKind<T, L>
where
    T: Fold<TF, TTF>,
    L: Fold<TF, TTF>,
{
    type Result = ParameterKind<T::Result, L::Result>;

    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        match self {
            ParameterKind::Ty(a) => Ok(ParameterKind::Ty(a.fold_with(folder, binders)?)),
            ParameterKind::Lifetime(a) => {
                Ok(ParameterKind::Lifetime(a.fold_with(folder, binders)?))
            }
        }
    }
}

impl<C: Context, TF: TypeFamily, TTF: TargetTypeFamily<TF>> Fold<TF, TTF> for ExClause<C>
where
    C: Context,
    C::Substitution: Fold<TF, TTF, Result = C::Substitution>,
    C::RegionConstraint: Fold<TF, TTF, Result = C::RegionConstraint>,
    C::CanonicalConstrainedSubst: Fold<TF, TTF, Result = C::CanonicalConstrainedSubst>,
    C::GoalInEnvironment: Fold<TF, TTF, Result = C::GoalInEnvironment>,
{
    type Result = ExClause<C>;

    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        let ExClause {
            subst,
            ambiguous,
            constraints,
            subgoals,
            answer_time,
            floundered_subgoals,
        } = self;
        Ok(ExClause {
            subst: subst.fold_with(folder, binders)?,
            ambiguous: *ambiguous,
            constraints: constraints.fold_with(folder, binders)?,
            subgoals: subgoals.fold_with(folder, binders)?,
            answer_time: answer_time.fold_with(folder, binders)?,
            floundered_subgoals: floundered_subgoals.fold_with(folder, binders)?,
        })
    }
}

impl<C: Context, TF: TypeFamily, TTF: TargetTypeFamily<TF>> Fold<TF, TTF> for FlounderedSubgoal<C>
where
    C: Context,
    C::Substitution: Fold<TF, TTF, Result = C::Substitution>,
    C::RegionConstraint: Fold<TF, TTF, Result = C::RegionConstraint>,
    C::CanonicalConstrainedSubst: Fold<TF, TTF, Result = C::CanonicalConstrainedSubst>,
    C::GoalInEnvironment: Fold<TF, TTF, Result = C::GoalInEnvironment>,
{
    type Result = FlounderedSubgoal<C>;

    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
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

impl<C: Context, TF: TypeFamily, TTF: TargetTypeFamily<TF>> Fold<TF, TTF> for Literal<C>
where
    C: Context,
    C::GoalInEnvironment: Fold<TF, TTF, Result = C::GoalInEnvironment>,
{
    type Result = Literal<C>;

    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        match self {
            Literal::Positive(goal) => Ok(Literal::Positive(goal.fold_with(folder, binders)?)),
            Literal::Negative(goal) => Ok(Literal::Negative(goal.fold_with(folder, binders)?)),
        }
    }
}
