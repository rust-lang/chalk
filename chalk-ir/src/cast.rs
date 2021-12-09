//! Upcasts, to avoid writing out wrapper types.

use crate::*;
use std::marker::PhantomData;

/// The `Cast` trait is used to make annoying upcasts between
/// logically equivalent types that imply wrappers. For example, one
/// could convert a `DomainGoal` into a `Goal` by doing:
///
/// ```ignore
/// let goal: Goal = domain_goal.cast();
/// ```
///
/// This is equivalent to the more explicit:
///
/// ```ignore
/// let goal: Goal = Goal::DomainGoal(domain_goal)
/// ```
///
/// Another useful trick is the `casted()` iterator adapter, which
/// casts each element in the iterator as it is produced (you must
/// have the `Caster` trait in scope for that).
///
/// # Invariant
///
/// `Cast` imposes a key invariant. You can only implement `T:
/// Cast<U>` if both `T` and `U` have the same semantic meaning. Also,
/// as part of this, they should always use the same set of free
/// variables (the `Canonical` implementation, for example, relies on
/// that).
///
/// # Iterators
///
/// If you import the `Caster` trait, you can also write `.casted()` on an
/// iterator chain to cast every instance within.
///
/// # Implementing Cast
///
/// Do not implement `Cast` directly. Instead, implement `CastTo`.
/// This split setup allows us to write `foo.cast::<T>()` to mean
/// "cast to T".
pub trait Cast: Sized {
    /// Cast a value to type `U` using `CastTo`.
    fn cast<U>(self, interner: U::Interner) -> U
    where
        Self: CastTo<U>,
        U: HasInterner,
    {
        self.cast_to(interner)
    }
}

impl<T> Cast for T {}

/// The "helper" trait for `cast` that actually implements the
/// transformations. You can also use this if you want to have
/// functions that take (e.g.) an `impl CastTo<Goal<_>>` or something
/// like that.
pub trait CastTo<T: HasInterner>: Sized {
    /// Cast a value to type `T`.
    fn cast_to(self, interner: T::Interner) -> T;
}

macro_rules! reflexive_impl {
    (for($($t:tt)*) $u:ty) => {
        impl<$($t)*> CastTo<$u> for $u {
            fn cast_to(self, _interner: <$u as HasInterner>::Interner) -> $u {
                self
            }
        }
    };
    ($u:ty) => {
        impl CastTo<$u> for $u {
            fn cast_to(self, interner: <$u as HasInterner>::Interner) -> $u {
                self
            }
        }
    };
}

reflexive_impl!(for(I: Interner) TyKind<I>);
reflexive_impl!(for(I: Interner) LifetimeData<I>);
reflexive_impl!(for(I: Interner) ConstData<I>);
reflexive_impl!(for(I: Interner) TraitRef<I>);
reflexive_impl!(for(I: Interner) DomainGoal<I>);
reflexive_impl!(for(I: Interner) Goal<I>);
reflexive_impl!(for(I: Interner) WhereClause<I>);
reflexive_impl!(for(I: Interner) ProgramClause<I>);
reflexive_impl!(for(I: Interner) QuantifiedWhereClause<I>);
reflexive_impl!(for(I: Interner) VariableKind<I>);
reflexive_impl!(for(I: Interner) VariableKinds<I>);
reflexive_impl!(for(I: Interner) CanonicalVarKind<I>);
reflexive_impl!(for(I: Interner) CanonicalVarKinds<I>);
reflexive_impl!(for(I: Interner) Constraint<I>);

impl<I: Interner> CastTo<WhereClause<I>> for TraitRef<I> {
    fn cast_to(self, _interner: I) -> WhereClause<I> {
        WhereClause::Implemented(self)
    }
}

impl<I: Interner> CastTo<WhereClause<I>> for AliasEq<I> {
    fn cast_to(self, _interner: I) -> WhereClause<I> {
        WhereClause::AliasEq(self)
    }
}

impl<I: Interner> CastTo<WhereClause<I>> for LifetimeOutlives<I> {
    fn cast_to(self, _interner: I) -> WhereClause<I> {
        WhereClause::LifetimeOutlives(self)
    }
}

impl<I: Interner> CastTo<WhereClause<I>> for TypeOutlives<I> {
    fn cast_to(self, _interner: I) -> WhereClause<I> {
        WhereClause::TypeOutlives(self)
    }
}

impl<T, I> CastTo<DomainGoal<I>> for T
where
    T: CastTo<WhereClause<I>>,
    I: Interner,
{
    fn cast_to(self, interner: I) -> DomainGoal<I> {
        DomainGoal::Holds(self.cast(interner))
    }
}

impl<T, I: Interner> CastTo<Goal<I>> for T
where
    T: CastTo<DomainGoal<I>>,
{
    fn cast_to(self, interner: I) -> Goal<I> {
        GoalData::DomainGoal(self.cast(interner)).intern(interner)
    }
}

impl<I: Interner> CastTo<DomainGoal<I>> for Normalize<I> {
    fn cast_to(self, _interner: I) -> DomainGoal<I> {
        DomainGoal::Normalize(self)
    }
}

impl<I: Interner> CastTo<DomainGoal<I>> for WellFormed<I> {
    fn cast_to(self, _interner: I) -> DomainGoal<I> {
        DomainGoal::WellFormed(self)
    }
}

impl<I: Interner> CastTo<DomainGoal<I>> for FromEnv<I> {
    fn cast_to(self, _interner: I) -> DomainGoal<I> {
        DomainGoal::FromEnv(self)
    }
}

impl<I: Interner> CastTo<Goal<I>> for EqGoal<I> {
    fn cast_to(self, interner: I) -> Goal<I> {
        GoalData::EqGoal(self).intern(interner)
    }
}

impl<I: Interner> CastTo<Goal<I>> for SubtypeGoal<I> {
    fn cast_to(self, interner: I) -> Goal<I> {
        GoalData::SubtypeGoal(self).intern(interner)
    }
}

impl<I: Interner, T: HasInterner<Interner = I> + CastTo<Goal<I>>> CastTo<Goal<I>> for Binders<T> {
    fn cast_to(self, interner: I) -> Goal<I> {
        GoalData::Quantified(
            QuantifierKind::ForAll,
            self.map(|bound| bound.cast(interner)),
        )
        .intern(interner)
    }
}

impl<I: Interner> CastTo<TyKind<I>> for AliasTy<I> {
    fn cast_to(self, _interner: I) -> TyKind<I> {
        TyKind::Alias(self)
    }
}

impl<I: Interner> CastTo<GenericArg<I>> for Ty<I> {
    fn cast_to(self, interner: I) -> GenericArg<I> {
        GenericArg::new(interner, GenericArgData::Ty(self))
    }
}

impl<I: Interner> CastTo<GenericArg<I>> for Lifetime<I> {
    fn cast_to(self, interner: I) -> GenericArg<I> {
        GenericArg::new(interner, GenericArgData::Lifetime(self))
    }
}

impl<I: Interner> CastTo<GenericArg<I>> for Const<I> {
    fn cast_to(self, interner: I) -> GenericArg<I> {
        GenericArg::new(interner, GenericArgData::Const(self))
    }
}

impl<I: Interner> CastTo<GenericArg<I>> for GenericArg<I> {
    fn cast_to(self, _interner: I) -> GenericArg<I> {
        self
    }
}

impl<T, I> CastTo<ProgramClause<I>> for T
where
    T: CastTo<DomainGoal<I>>,
    I: Interner,
{
    fn cast_to(self, interner: I) -> ProgramClause<I> {
        let implication = ProgramClauseImplication {
            consequence: self.cast(interner),
            conditions: Goals::empty(interner),
            constraints: Constraints::empty(interner),
            priority: ClausePriority::High,
        };

        ProgramClauseData(Binders::empty(interner, implication.shifted_in(interner)))
            .intern(interner)
    }
}

impl<I, T> CastTo<ProgramClause<I>> for Binders<T>
where
    I: Interner,
    T: HasInterner<Interner = I> + CastTo<DomainGoal<I>>,
{
    fn cast_to(self, interner: I) -> ProgramClause<I> {
        ProgramClauseData(self.map(|bound| ProgramClauseImplication {
            consequence: bound.cast(interner),
            conditions: Goals::empty(interner),
            constraints: Constraints::empty(interner),
            priority: ClausePriority::High,
        }))
        .intern(interner)
    }
}

impl<T, U> CastTo<Option<U>> for Option<T>
where
    T: CastTo<U>,
    U: HasInterner,
{
    fn cast_to(self, interner: U::Interner) -> Option<U> {
        self.map(|v| v.cast(interner))
    }
}

impl<T, U, I> CastTo<InEnvironment<U>> for InEnvironment<T>
where
    T: HasInterner<Interner = I> + CastTo<U>,
    U: HasInterner<Interner = I>,
    I: Interner,
{
    fn cast_to(self, interner: U::Interner) -> InEnvironment<U> {
        self.map(|v| v.cast(interner))
    }
}

impl<T, U, E> CastTo<Result<U, E>> for Result<T, E>
where
    T: CastTo<U>,
    U: HasInterner,
{
    fn cast_to(self, interner: U::Interner) -> Result<U, E> {
        self.map(|v| v.cast(interner))
    }
}

impl<T> HasInterner for Option<T>
where
    T: HasInterner,
{
    type Interner = T::Interner;
}

impl<T, E> HasInterner for Result<T, E>
where
    T: HasInterner,
{
    type Interner = T::Interner;
}

impl<T, U> CastTo<Canonical<U>> for Canonical<T>
where
    T: CastTo<U> + HasInterner,
    U: HasInterner<Interner = T::Interner>,
{
    fn cast_to(self, interner: T::Interner) -> Canonical<U> {
        // Subtle point: It should be ok to re-use the binders here,
        // because `cast()` never introduces new inference variables,
        // nor changes the "substance" of the type we are working
        // with. It just introduces new wrapper types.
        Canonical {
            value: self.value.cast(interner),
            binders: self.binders.cast(interner),
        }
    }
}

impl<T, U> CastTo<Vec<U>> for Vec<T>
where
    T: CastTo<U> + HasInterner,
    U: HasInterner,
{
    fn cast_to(self, interner: U::Interner) -> Vec<U> {
        self.into_iter().casted(interner).collect()
    }
}

impl<T> CastTo<T> for &T
where
    T: Clone + HasInterner,
{
    fn cast_to(self, _interner: T::Interner) -> T {
        self.clone()
    }
}

/// An iterator that casts each element to some other type.
pub struct Casted<IT, U: HasInterner> {
    interner: U::Interner,
    iterator: IT,
    _cast: PhantomData<U>,
}

impl<IT: Iterator, U> Iterator for Casted<IT, U>
where
    IT::Item: CastTo<U>,
    U: HasInterner,
{
    type Item = U;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next().map(|item| item.cast_to(self.interner))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iterator.size_hint()
    }
}

/// An iterator adapter that casts each element we are iterating over
/// to some other type.
pub trait Caster: Iterator + Sized {
    /// Cast each element in this iterator.
    fn casted<U>(self, interner: U::Interner) -> Casted<Self, U>
    where
        Self::Item: CastTo<U>,
        U: HasInterner,
    {
        Casted {
            interner,
            iterator: self,
            _cast: PhantomData,
        }
    }
}

impl<I> Caster for I where I: Iterator {}
