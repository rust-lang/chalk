//! This module contains "rote and uninteresting" impls of `TypeVisitable` for
//! various types. In general, we prefer to derive `TypeVisitable`, but
//! sometimes that doesn't work for whatever reason.
//!
//! The more interesting impls of `TypeVisitable` remain in the `visit` module.

use crate::{
    try_break, AdtId, AssocTypeId, ClausePriority, ClosureId, Constraints, ControlFlow,
    CoroutineId, DebruijnIndex, FloatTy, FnDefId, ForeignDefId, GenericArg, Goals, ImplId, IntTy,
    Interner, Mutability, OpaqueTyId, PlaceholderIndex, ProgramClause, ProgramClauses,
    QuantifiedWhereClauses, QuantifierKind, Safety, Scalar, Substitution, TraitId,
    TypeSuperVisitable, TypeVisitable, TypeVisitor, UintTy, UniverseIndex,
};
use std::{marker::PhantomData, sync::Arc};

/// Convenience function to visit all the items in the iterator it.
pub fn visit_iter<'i, T, I, B>(
    it: impl Iterator<Item = T>,
    visitor: &mut dyn TypeVisitor<I, BreakTy = B>,
    outer_binder: DebruijnIndex,
) -> ControlFlow<B>
where
    T: TypeVisitable<I>,
    I: 'i + Interner,
{
    for e in it {
        try_break!(e.visit_with(visitor, outer_binder));
    }
    ControlFlow::Continue(())
}

impl<T: TypeVisitable<I>, I: Interner> TypeVisitable<I> for &T {
    fn visit_with<B>(
        &self,
        visitor: &mut dyn TypeVisitor<I, BreakTy = B>,
        outer_binder: DebruijnIndex,
    ) -> ControlFlow<B> {
        T::visit_with(self, visitor, outer_binder)
    }
}

impl<T: TypeVisitable<I>, I: Interner> TypeVisitable<I> for Vec<T> {
    fn visit_with<B>(
        &self,
        visitor: &mut dyn TypeVisitor<I, BreakTy = B>,
        outer_binder: DebruijnIndex,
    ) -> ControlFlow<B> {
        visit_iter(self.iter(), visitor, outer_binder)
    }
}

impl<T: TypeVisitable<I>, I: Interner> TypeVisitable<I> for &[T] {
    fn visit_with<B>(
        &self,
        visitor: &mut dyn TypeVisitor<I, BreakTy = B>,
        outer_binder: DebruijnIndex,
    ) -> ControlFlow<B> {
        visit_iter(self.iter(), visitor, outer_binder)
    }
}

impl<T: TypeVisitable<I>, I: Interner> TypeVisitable<I> for Box<T> {
    fn visit_with<B>(
        &self,
        visitor: &mut dyn TypeVisitor<I, BreakTy = B>,
        outer_binder: DebruijnIndex,
    ) -> ControlFlow<B> {
        T::visit_with(self, visitor, outer_binder)
    }
}

impl<T: TypeVisitable<I>, I: Interner> TypeVisitable<I> for Arc<T> {
    fn visit_with<B>(
        &self,
        visitor: &mut dyn TypeVisitor<I, BreakTy = B>,
        outer_binder: DebruijnIndex,
    ) -> ControlFlow<B> {
        T::visit_with(self, visitor, outer_binder)
    }
}

macro_rules! tuple_visit {
    ($($n:ident),*) => {
        impl<$($n: TypeVisitable<I>,)* I: Interner> TypeVisitable<I> for ($($n,)*) {
            fn visit_with<BT>(&self, visitor: &mut dyn TypeVisitor<I, BreakTy = BT>, outer_binder: DebruijnIndex) -> ControlFlow<BT> {
                #[allow(non_snake_case)]
                let &($(ref $n),*) = self;
                $(
                    try_break!($n.visit_with(visitor, outer_binder));
                )*
                ControlFlow::Continue(())
            }
        }
    }
}

tuple_visit!(A, B);
tuple_visit!(A, B, C);
tuple_visit!(A, B, C, D);
tuple_visit!(A, B, C, D, E);

impl<T: TypeVisitable<I>, I: Interner> TypeVisitable<I> for Option<T> {
    fn visit_with<B>(
        &self,
        visitor: &mut dyn TypeVisitor<I, BreakTy = B>,
        outer_binder: DebruijnIndex,
    ) -> ControlFlow<B> {
        match self {
            Some(e) => e.visit_with(visitor, outer_binder),
            None => ControlFlow::Continue(()),
        }
    }
}

impl<I: Interner> TypeVisitable<I> for GenericArg<I> {
    fn visit_with<B>(
        &self,
        visitor: &mut dyn TypeVisitor<I, BreakTy = B>,
        outer_binder: DebruijnIndex,
    ) -> ControlFlow<B> {
        let interner = visitor.interner();
        self.data(interner).visit_with(visitor, outer_binder)
    }
}

impl<I: Interner> TypeVisitable<I> for Substitution<I> {
    fn visit_with<B>(
        &self,
        visitor: &mut dyn TypeVisitor<I, BreakTy = B>,
        outer_binder: DebruijnIndex,
    ) -> ControlFlow<B> {
        let interner = visitor.interner();
        visit_iter(self.iter(interner), visitor, outer_binder)
    }
}

impl<I: Interner> TypeVisitable<I> for Goals<I> {
    fn visit_with<B>(
        &self,
        visitor: &mut dyn TypeVisitor<I, BreakTy = B>,
        outer_binder: DebruijnIndex,
    ) -> ControlFlow<B> {
        let interner = visitor.interner();
        visit_iter(self.iter(interner), visitor, outer_binder)
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! const_visit {
    ($t:ty) => {
        impl<I: Interner> $crate::visit::TypeVisitable<I> for $t {
            fn visit_with<B>(
                &self,
                _visitor: &mut dyn ($crate::visit::TypeVisitor<I, BreakTy = B>),
                _outer_binder: DebruijnIndex,
            ) -> ControlFlow<B> {
                ControlFlow::Continue(())
            }
        }
    };
}

const_visit!(bool);
const_visit!(usize);
const_visit!(UniverseIndex);
const_visit!(PlaceholderIndex);
const_visit!(QuantifierKind);
const_visit!(DebruijnIndex);
const_visit!(ClausePriority);
const_visit!(());
const_visit!(Scalar);
const_visit!(UintTy);
const_visit!(IntTy);
const_visit!(FloatTy);
const_visit!(Mutability);
const_visit!(Safety);

#[doc(hidden)]
#[macro_export]
macro_rules! id_visit {
    ($t:ident) => {
        impl<I: Interner> $crate::visit::TypeVisitable<I> for $t<I> {
            fn visit_with<B>(
                &self,
                _visitor: &mut dyn ($crate::visit::TypeVisitor<I, BreakTy = B>),
                _outer_binder: DebruijnIndex,
            ) -> ControlFlow<B> {
                ControlFlow::Continue(())
            }
        }
    };
}

id_visit!(ImplId);
id_visit!(AdtId);
id_visit!(TraitId);
id_visit!(OpaqueTyId);
id_visit!(AssocTypeId);
id_visit!(FnDefId);
id_visit!(ClosureId);
id_visit!(CoroutineId);
id_visit!(ForeignDefId);

impl<I: Interner> TypeSuperVisitable<I> for ProgramClause<I> {
    fn super_visit_with<B>(
        &self,
        visitor: &mut dyn TypeVisitor<I, BreakTy = B>,
        outer_binder: DebruijnIndex,
    ) -> ControlFlow<B> {
        let interner = visitor.interner();

        self.data(interner).0.visit_with(visitor, outer_binder)
    }
}

impl<I: Interner> TypeVisitable<I> for ProgramClauses<I> {
    fn visit_with<B>(
        &self,
        visitor: &mut dyn TypeVisitor<I, BreakTy = B>,
        outer_binder: DebruijnIndex,
    ) -> ControlFlow<B> {
        let interner = visitor.interner();

        visit_iter(self.iter(interner), visitor, outer_binder)
    }
}

impl<I: Interner> TypeVisitable<I> for Constraints<I> {
    fn visit_with<B>(
        &self,
        visitor: &mut dyn TypeVisitor<I, BreakTy = B>,
        outer_binder: DebruijnIndex,
    ) -> ControlFlow<B> {
        let interner = visitor.interner();

        visit_iter(self.iter(interner), visitor, outer_binder)
    }
}

impl<I: Interner> TypeVisitable<I> for QuantifiedWhereClauses<I> {
    fn visit_with<B>(
        &self,
        visitor: &mut dyn TypeVisitor<I, BreakTy = B>,
        outer_binder: DebruijnIndex,
    ) -> ControlFlow<B> {
        let interner = visitor.interner();

        visit_iter(self.iter(interner), visitor, outer_binder)
    }
}

impl<I: Interner> TypeVisitable<I> for PhantomData<I> {
    fn visit_with<B>(
        &self,
        _visitor: &mut dyn TypeVisitor<I, BreakTy = B>,
        _outer_binder: DebruijnIndex,
    ) -> ControlFlow<B> {
        ControlFlow::Continue(())
    }
}
