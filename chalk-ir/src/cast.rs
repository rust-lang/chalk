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
    fn cast<U>(self) -> U
    where
        Self: CastTo<U>,
    {
        self.cast_to()
    }
}

impl<T> Cast for T {}

/// The "helper" trait for `cast` that actually implements the
/// transformations. You can also use this if you want to have
/// functions that take (e.g.) an `impl CastTo<Goal<_>>` or something
/// like that.
pub trait CastTo<T>: Sized {
    fn cast_to(self) -> T;
}

macro_rules! reflexive_impl {
    (for($($t:tt)*) $u:ty) => {
        impl<$($t)*> CastTo<$u> for $u {
            fn cast_to(self) -> $u {
                self
            }
        }
    };
    ($u:ty) => {
        impl CastTo<$u> for $u {
            fn cast_to(self) -> $u {
                self
            }
        }
    };
}

reflexive_impl!(for(I: Interner) TyData<I>);
reflexive_impl!(for(I: Interner) LifetimeData<I>);
reflexive_impl!(for(I: Interner) TraitRef<I>);
reflexive_impl!(for(I: Interner) DomainGoal<I>);
reflexive_impl!(for(I: Interner) Goal<I>);
reflexive_impl!(for(I: Interner) WhereClause<I>);

impl<I: Interner> CastTo<WhereClause<I>> for TraitRef<I> {
    fn cast_to(self) -> WhereClause<I> {
        WhereClause::Implemented(self)
    }
}

impl<I: Interner> CastTo<WhereClause<I>> for AliasEq<I> {
    fn cast_to(self) -> WhereClause<I> {
        WhereClause::AliasEq(self)
    }
}

impl<T, I> CastTo<DomainGoal<I>> for T
where
    T: CastTo<WhereClause<I>>,
    I: Interner,
{
    fn cast_to(self) -> DomainGoal<I> {
        DomainGoal::Holds(self.cast())
    }
}

impl<T, I: Interner> CastTo<Goal<I>> for T
where
    T: CastTo<DomainGoal<I>>,
{
    fn cast_to(self) -> Goal<I> {
        GoalData::DomainGoal(self.cast()).intern()
    }
}

impl<I: Interner> CastTo<DomainGoal<I>> for Normalize<I> {
    fn cast_to(self) -> DomainGoal<I> {
        DomainGoal::Normalize(self)
    }
}

impl<I: Interner> CastTo<DomainGoal<I>> for WellFormed<I> {
    fn cast_to(self) -> DomainGoal<I> {
        DomainGoal::WellFormed(self)
    }
}

impl<I: Interner> CastTo<DomainGoal<I>> for FromEnv<I> {
    fn cast_to(self) -> DomainGoal<I> {
        DomainGoal::FromEnv(self)
    }
}

impl<I: Interner> CastTo<Goal<I>> for EqGoal<I> {
    fn cast_to(self) -> Goal<I> {
        GoalData::EqGoal(self).intern()
    }
}

impl<T: CastTo<Goal<I>>, I: Interner> CastTo<Goal<I>> for Binders<T> {
    fn cast_to(self) -> Goal<I> {
        if self.binders.is_empty() {
            self.value.cast()
        } else {
            GoalData::Quantified(QuantifierKind::ForAll, self.map(|bound| bound.cast())).intern()
        }
    }
}

impl<I: Interner> CastTo<TyData<I>> for ApplicationTy<I> {
    fn cast_to(self) -> TyData<I> {
        TyData::Apply(self)
    }
}

impl<I: Interner> CastTo<TyData<I>> for AliasTy<I> {
    fn cast_to(self) -> TyData<I> {
        TyData::Alias(self)
    }
}

impl<I: Interner> CastTo<Parameter<I>> for Ty<I> {
    fn cast_to(self) -> Parameter<I> {
        Parameter::new(ParameterKind::Ty(self))
    }
}

impl<I: Interner> CastTo<Parameter<I>> for Lifetime<I> {
    fn cast_to(self) -> Parameter<I> {
        Parameter::new(ParameterKind::Lifetime(self))
    }
}

impl<I: Interner> CastTo<Parameter<I>> for Parameter<I> {
    fn cast_to(self) -> Parameter<I> {
        self
    }
}

impl<T, I> CastTo<ProgramClause<I>> for T
where
    T: CastTo<DomainGoal<I>>,
    I: Interner,
{
    fn cast_to(self) -> ProgramClause<I> {
        ProgramClause::Implies(ProgramClauseImplication {
            consequence: self.cast(),
            conditions: Goals::new(),
        })
    }
}

impl<T, I> CastTo<ProgramClause<I>> for Binders<T>
where
    T: CastTo<DomainGoal<I>>,
    I: Interner,
{
    fn cast_to(self) -> ProgramClause<I> {
        if self.binders.is_empty() {
            self.value.cast::<ProgramClause<I>>()
        } else {
            ProgramClause::ForAll(self.map(|bound| ProgramClauseImplication {
                consequence: bound.cast(),
                conditions: Goals::new(),
            }))
        }
    }
}

impl<I: Interner> CastTo<ProgramClause<I>> for ProgramClauseImplication<I> {
    fn cast_to(self) -> ProgramClause<I> {
        ProgramClause::Implies(self)
    }
}

impl<I: Interner> CastTo<ProgramClause<I>> for Binders<ProgramClauseImplication<I>> {
    fn cast_to(self) -> ProgramClause<I> {
        ProgramClause::ForAll(self)
    }
}

macro_rules! map_impl {
    (impl[$($t:tt)*] CastTo<$b:ty> for $a:ty) => {
        impl<$($t)*> CastTo<$b> for $a {
            fn cast_to(self) -> $b {
                self.map(|v| v.cast())
            }
        }
    }
}

map_impl!(impl[T: CastTo<U>, U] CastTo<Option<U>> for Option<T>);
map_impl!(impl[
    T: HasInterner<Interner = I> + CastTo<U>,
    U: HasInterner<Interner = I>,
    I: Interner,
] CastTo<InEnvironment<U>> for InEnvironment<T>);
map_impl!(impl[T: CastTo<U>, U, E] CastTo<Result<U, E>> for Result<T, E>);

impl<T, U> CastTo<Canonical<U>> for Canonical<T>
where
    T: CastTo<U>,
{
    fn cast_to(self) -> Canonical<U> {
        // Subtle point: It should be ok to re-use the binders here,
        // because `cast()` never introduces new inference variables,
        // nor changes the "substance" of the type we are working
        // with. It just introduces new wrapper types.
        Canonical {
            value: self.value.cast(),
            binders: self.binders,
        }
    }
}

impl<T, U> CastTo<Vec<U>> for Vec<T>
where
    T: CastTo<U>,
{
    fn cast_to(self) -> Vec<U> {
        self.into_iter().casted().collect()
    }
}

impl<I> CastTo<TypeName<I>> for StructId<I>
where
    I: Interner,
{
    fn cast_to(self) -> TypeName<I> {
        TypeName::Struct(self)
    }
}

impl<T> CastTo<T> for &T
where
    T: Clone,
{
    fn cast_to(self) -> T {
        self.clone()
    }
}

pub struct Casted<I, U> {
    iterator: I,
    _cast: PhantomData<U>,
}

impl<I: Iterator, U> Iterator for Casted<I, U>
where
    I::Item: CastTo<U>,
{
    type Item = U;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next().map(|item| item.cast_to())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iterator.size_hint()
    }
}

/// An iterator adapter that casts each element we are iterating over
/// to some other type.
pub trait Caster: Iterator + Sized {
    fn casted<U>(self) -> Casted<Self, U>
    where
        Self::Item: CastTo<U>,
    {
        Casted {
            iterator: self,
            _cast: PhantomData,
        }
    }
}

impl<I> Caster for I where I: Iterator {}
