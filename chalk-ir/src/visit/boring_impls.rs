//! This module contains "rote and uninteresting" impls of `Visit` for
//! various types. In general, we prefer to derive `Visit`, but
//! sometimes that doesn't work for whatever reason.
//!
//! The more interesting impls of `Visit` remain in the `visit` module.

use crate::{
    AssocTypeId, ClausePriority, DebruijnIndex, FloatTy, Goals, ImplId, IntTy, Interner,
    OpaqueTyId, Parameter, ParameterKind, PlaceholderIndex, ProgramClause, ProgramClauseData,
    ProgramClauses, QuantifiedWhereClauses, QuantifierKind, Scalar, StructId, Substitution,
    SuperVisit, TraitId, UintTy, UniverseIndex, Visit, VisitResult, Visitor,
};
use chalk_engine::{context::Context, ExClause, FlounderedSubgoal, Literal};
use std::{marker::PhantomData, sync::Arc};

/// Convenience function to visit all the items in the iterator it.
pub fn visit_iter<'i, T, I, R>(
    it: impl Iterator<Item = T>,
    visitor: &mut dyn Visitor<'i, I, Result = R>,
    outer_binder: DebruijnIndex,
) -> R
where
    T: Visit<I>,
    I: 'i + Interner,
    R: VisitResult,
{
    let mut result = R::new();
    for e in it {
        result = result.combine(e.visit_with(visitor, outer_binder));
        if result.return_early() {
            return result;
        }
    }
    result
}

impl<T: Visit<I>, I: Interner> Visit<I> for &T {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        T::visit_with(self, visitor, outer_binder)
    }
}

impl<T: Visit<I>, I: Interner> Visit<I> for Vec<T> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        visit_iter(self.iter(), visitor, outer_binder)
    }
}

impl<T: Visit<I>, I: Interner> Visit<I> for &[T] {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        visit_iter(self.iter(), visitor, outer_binder)
    }
}

impl<T: Visit<I>, I: Interner> Visit<I> for Box<T> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        T::visit_with(self, visitor, outer_binder)
    }
}

impl<T: Visit<I>, I: Interner> Visit<I> for Arc<T> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        T::visit_with(self, visitor, outer_binder)
    }
}

macro_rules! tuple_visit {
    ($($n:ident),*) => {
        impl<$($n: Visit<I>,)* I: Interner> Visit<I> for ($($n,)*) {
            fn visit_with<'i, R: VisitResult>(&self, visitor: &mut dyn Visitor<'i, I, Result = R>, outer_binder: DebruijnIndex) -> R where I: 'i
            {
                #[allow(non_snake_case)]
                let &($(ref $n),*) = self;
                let mut result = R::new();
                $(
                    result = result.combine($n.visit_with(visitor, outer_binder));
                    if result.return_early() { return result; }
                )*
                result
            }
        }
    }
}

tuple_visit!(A, B);
tuple_visit!(A, B, C);
tuple_visit!(A, B, C, D);
tuple_visit!(A, B, C, D, E);

impl<T: Visit<I>, I: Interner> Visit<I> for Option<T> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        match self {
            Some(e) => e.visit_with(visitor, outer_binder),
            None => R::new(),
        }
    }
}

impl<I: Interner> Visit<I> for Parameter<I> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        let interner = visitor.interner();
        self.data(interner).visit_with(visitor, outer_binder)
    }
}

impl<I: Interner> Visit<I> for Substitution<I> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        let interner = visitor.interner();
        visit_iter(self.iter(interner), visitor, outer_binder)
    }
}

impl<I: Interner> Visit<I> for Goals<I> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        let interner = visitor.interner();
        visit_iter(self.iter(interner), visitor, outer_binder)
    }
}

#[macro_export]
macro_rules! const_visit {
    ($t:ty) => {
        impl<I: Interner> $crate::visit::Visit<I> for $t {
            fn visit_with<'i, R: VisitResult>(
                &self,
                _visitor: &mut dyn ($crate::visit::Visitor<'i, I, Result = R>),
                _outer_binder: DebruijnIndex,
            ) -> R
            where
                I: 'i,
            {
                R::new()
            }
        }
    };
}

const_visit!(UniverseIndex);
const_visit!(usize);
const_visit!(PlaceholderIndex);
const_visit!(QuantifierKind);
const_visit!(DebruijnIndex);
const_visit!(chalk_engine::TableIndex);
const_visit!(chalk_engine::TimeStamp);
const_visit!(ClausePriority);
const_visit!(());
const_visit!(Scalar);
const_visit!(UintTy);
const_visit!(IntTy);
const_visit!(FloatTy);

#[macro_export]
macro_rules! id_visit {
    ($t:ident) => {
        impl<I: Interner> $crate::visit::Visit<I> for $t<I> {
            fn visit_with<'i, R: VisitResult>(
                &self,
                _visitor: &mut dyn ($crate::visit::Visitor<'i, I, Result = R>),
                _outer_binder: DebruijnIndex,
            ) -> R
            where
                I: 'i,
            {
                R::new()
            }
        }
    };
}

id_visit!(ImplId);
id_visit!(StructId);
id_visit!(TraitId);
id_visit!(OpaqueTyId);
id_visit!(AssocTypeId);

impl<I: Interner> SuperVisit<I> for ProgramClause<I> {
    fn super_visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        let interner = visitor.interner();

        match self.data(interner) {
            ProgramClauseData::Implies(pci) => pci.visit_with(visitor, outer_binder),
            ProgramClauseData::ForAll(pci) => pci.visit_with(visitor, outer_binder),
        }
    }
}

impl<I: Interner> Visit<I> for ProgramClauses<I> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        let interner = visitor.interner();

        visit_iter(self.iter(interner), visitor, outer_binder)
    }
}

impl<I: Interner> Visit<I> for QuantifiedWhereClauses<I> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        let interner = visitor.interner();

        visit_iter(self.iter(interner), visitor, outer_binder)
    }
}

impl<I: Interner> Visit<I> for PhantomData<I> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        _visitor: &mut dyn Visitor<'i, I, Result = R>,
        _outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        R::new()
    }
}

impl<I: Interner, T, L> Visit<I> for ParameterKind<T, L>
where
    T: Visit<I>,
    L: Visit<I>,
{
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        match self {
            ParameterKind::Ty(a) => a.visit_with(visitor, outer_binder),
            ParameterKind::Lifetime(a) => a.visit_with(visitor, outer_binder),
        }
    }
}

impl<C: Context, I: Interner> Visit<I> for ExClause<C>
where
    C: Context,
    C::Substitution: Visit<I>,
    C::RegionConstraint: Visit<I>,
    C::CanonicalConstrainedSubst: Visit<I>,
    C::GoalInEnvironment: Visit<I>,
{
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        let ExClause {
            subst,
            ambiguous: _,
            constraints,
            subgoals,
            delayed_subgoals,
            answer_time,
            floundered_subgoals,
        } = self;

        R::new()
            .and_then(|| subst.visit_with(visitor, outer_binder))
            .and_then(|| constraints.visit_with(visitor, outer_binder))
            .and_then(|| constraints.visit_with(visitor, outer_binder))
            .and_then(|| subgoals.visit_with(visitor, outer_binder))
            .and_then(|| delayed_subgoals.visit_with(visitor, outer_binder))
            .and_then(|| answer_time.visit_with(visitor, outer_binder))
            .and_then(|| floundered_subgoals.visit_with(visitor, outer_binder))
    }
}

impl<C: Context, I: Interner> Visit<I> for FlounderedSubgoal<C>
where
    C: Context,
    C::Substitution: Visit<I>,
    C::RegionConstraint: Visit<I>,
    C::CanonicalConstrainedSubst: Visit<I>,
    C::GoalInEnvironment: Visit<I>,
{
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        let FlounderedSubgoal {
            floundered_literal,
            floundered_time,
        } = self;

        R::new()
            .and_then(|| floundered_literal.visit_with(visitor, outer_binder))
            .and_then(|| floundered_time.visit_with(visitor, outer_binder))
    }
}

impl<C: Context, I: Interner> Visit<I> for Literal<C>
where
    C: Context,
    C::GoalInEnvironment: Visit<I>,
{
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        match self {
            Literal::Positive(goal) => goal.visit_with(visitor, outer_binder),
            Literal::Negative(goal) => goal.visit_with(visitor, outer_binder),
        }
    }
}
