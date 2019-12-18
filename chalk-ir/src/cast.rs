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

reflexive_impl!(for(TF: TypeFamily) TyData<TF>);
reflexive_impl!(for(TF: TypeFamily) LifetimeData<TF>);
reflexive_impl!(for(TF: TypeFamily) TraitRef<TF>);
reflexive_impl!(for(TF: TypeFamily) LeafGoal<TF>);
reflexive_impl!(for(TF: TypeFamily) DomainGoal<TF>);
reflexive_impl!(for(TF: TypeFamily) Goal<TF>);
reflexive_impl!(for(TF: TypeFamily) WhereClause<TF>);

impl<TF: TypeFamily> CastTo<WhereClause<TF>> for TraitRef<TF> {
    fn cast_to(self) -> WhereClause<TF> {
        WhereClause::Implemented(self)
    }
}

impl<TF: TypeFamily> CastTo<WhereClause<TF>> for ProjectionEq<TF> {
    fn cast_to(self) -> WhereClause<TF> {
        WhereClause::ProjectionEq(self)
    }
}

impl<T, TF> CastTo<DomainGoal<TF>> for T
where
    T: CastTo<WhereClause<TF>>,
    TF: TypeFamily,
{
    fn cast_to(self) -> DomainGoal<TF> {
        DomainGoal::Holds(self.cast())
    }
}

impl<T, TF: TypeFamily> CastTo<LeafGoal<TF>> for T
where
    T: CastTo<DomainGoal<TF>>,
{
    fn cast_to(self) -> LeafGoal<TF> {
        LeafGoal::DomainGoal(self.cast())
    }
}

impl<T, TF: TypeFamily> CastTo<Goal<TF>> for T
where
    T: CastTo<LeafGoal<TF>>,
{
    fn cast_to(self) -> Goal<TF> {
        Goal::Leaf(self.cast())
    }
}

impl<TF: TypeFamily> CastTo<DomainGoal<TF>> for Normalize<TF> {
    fn cast_to(self) -> DomainGoal<TF> {
        DomainGoal::Normalize(self)
    }
}

impl<TF: TypeFamily> CastTo<DomainGoal<TF>> for WellFormed<TF> {
    fn cast_to(self) -> DomainGoal<TF> {
        DomainGoal::WellFormed(self)
    }
}

impl<TF: TypeFamily> CastTo<DomainGoal<TF>> for FromEnv<TF> {
    fn cast_to(self) -> DomainGoal<TF> {
        DomainGoal::FromEnv(self)
    }
}

impl<TF: TypeFamily> CastTo<LeafGoal<TF>> for EqGoal<TF> {
    fn cast_to(self) -> LeafGoal<TF> {
        LeafGoal::EqGoal(self)
    }
}

impl<T: CastTo<Goal<TF>>, TF: TypeFamily> CastTo<Goal<TF>> for Binders<T> {
    fn cast_to(self) -> Goal<TF> {
        if self.binders.is_empty() {
            self.value.cast()
        } else {
            Goal::Quantified(
                QuantifierKind::ForAll,
                self.map(|bound| Box::new(bound.cast())),
            )
        }
    }
}

impl<TF: TypeFamily> CastTo<TyData<TF>> for ApplicationTy<TF> {
    fn cast_to(self) -> TyData<TF> {
        TyData::Apply(self)
    }
}

impl<TF: TypeFamily> CastTo<TyData<TF>> for ProjectionTy<TF> {
    fn cast_to(self) -> TyData<TF> {
        TyData::Projection(self)
    }
}

impl<TF: TypeFamily> CastTo<Parameter<TF>> for Ty<TF> {
    fn cast_to(self) -> Parameter<TF> {
        Parameter(ParameterKind::Ty(self))
    }
}

impl<TF: TypeFamily> CastTo<Parameter<TF>> for Lifetime<TF> {
    fn cast_to(self) -> Parameter<TF> {
        Parameter(ParameterKind::Lifetime(self))
    }
}

impl<T, TF> CastTo<ProgramClause<TF>> for T
where
    T: CastTo<DomainGoal<TF>>,
    TF: TypeFamily,
{
    fn cast_to(self) -> ProgramClause<TF> {
        ProgramClause::Implies(ProgramClauseImplication {
            consequence: self.cast(),
            conditions: vec![],
        })
    }
}

impl<T, TF> CastTo<ProgramClause<TF>> for Binders<T>
where
    T: CastTo<DomainGoal<TF>>,
    TF: TypeFamily,
{
    fn cast_to(self) -> ProgramClause<TF> {
        if self.binders.is_empty() {
            self.value.cast::<ProgramClause<TF>>()
        } else {
            ProgramClause::ForAll(self.map(|bound| ProgramClauseImplication {
                consequence: bound.cast(),
                conditions: vec![],
            }))
        }
    }
}

impl<TF: TypeFamily> CastTo<ProgramClause<TF>> for ProgramClauseImplication<TF> {
    fn cast_to(self) -> ProgramClause<TF> {
        ProgramClause::Implies(self)
    }
}

impl<TF: TypeFamily> CastTo<ProgramClause<TF>> for Binders<ProgramClauseImplication<TF>> {
    fn cast_to(self) -> ProgramClause<TF> {
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
    T: HasTypeFamily<TypeFamily = TF> + CastTo<U>,
    U: HasTypeFamily<TypeFamily = TF>,
    TF: TypeFamily,
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

impl<TF: TypeFamily> CastTo<TypeKindId<TF>> for StructId<TF> {
    fn cast_to(self) -> TypeKindId<TF> {
        TypeKindId::StructId(self)
    }
}

impl<TF> CastTo<TypeName<TF>> for StructId<TF>
where
    TF: TypeFamily,
{
    fn cast_to(self) -> TypeName<TF> {
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
