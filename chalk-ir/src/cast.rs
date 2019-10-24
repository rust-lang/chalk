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
pub trait Cast<T>: Sized {
    fn cast(self) -> T;
}

macro_rules! reflexive_impl {
    (for($($t:tt)*) $u:ty) => {
        impl<$($t)*> Cast<$u> for $u {
            fn cast(self) -> $u {
                self
            }
        }
    };
    ($u:ty) => {
        impl Cast<$u> for $u {
            fn cast(self) -> $u {
                self
            }
        }
    };
}

reflexive_impl!(for(TF: TypeFamily) TraitRef<TF>);
reflexive_impl!(for(TF: TypeFamily) LeafGoal<TF>);
reflexive_impl!(for(TF: TypeFamily) DomainGoal<TF>);
reflexive_impl!(for(TF: TypeFamily) WhereClause<TF>);

impl<TF: TypeFamily> Cast<WhereClause<TF>> for TraitRef<TF> {
    fn cast(self) -> WhereClause<TF> {
        WhereClause::Implemented(self)
    }
}

impl<TF: TypeFamily> Cast<WhereClause<TF>> for ProjectionEq<TF> {
    fn cast(self) -> WhereClause<TF> {
        WhereClause::ProjectionEq(self)
    }
}

impl<T, TF> Cast<DomainGoal<TF>> for T
where
    T: Cast<WhereClause<TF>>,
    TF: TypeFamily,
{
    fn cast(self) -> DomainGoal<TF> {
        DomainGoal::Holds(self.cast())
    }
}

impl<T, TF: TypeFamily> Cast<LeafGoal<TF>> for T
where
    T: Cast<DomainGoal<TF>>,
{
    fn cast(self) -> LeafGoal<TF> {
        LeafGoal::DomainGoal(self.cast())
    }
}

impl<T, TF: TypeFamily> Cast<Goal<TF>> for T
where
    T: Cast<LeafGoal<TF>>,
{
    fn cast(self) -> Goal<TF> {
        Goal::Leaf(self.cast())
    }
}

impl<TF: TypeFamily> Cast<DomainGoal<TF>> for Normalize<TF> {
    fn cast(self) -> DomainGoal<TF> {
        DomainGoal::Normalize(self)
    }
}

impl<TF: TypeFamily> Cast<DomainGoal<TF>> for WellFormed<TF> {
    fn cast(self) -> DomainGoal<TF> {
        DomainGoal::WellFormed(self)
    }
}

impl<TF: TypeFamily> Cast<DomainGoal<TF>> for FromEnv<TF> {
    fn cast(self) -> DomainGoal<TF> {
        DomainGoal::FromEnv(self)
    }
}

impl<TF: TypeFamily> Cast<LeafGoal<TF>> for EqGoal<TF> {
    fn cast(self) -> LeafGoal<TF> {
        LeafGoal::EqGoal(self)
    }
}

impl<T: Cast<Goal<TF>>, TF: TypeFamily> Cast<Goal<TF>> for Binders<T> {
    fn cast(self) -> Goal<TF> {
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

impl<TF: TypeFamily> Cast<Ty<TF>> for ApplicationTy<TF> {
    fn cast(self) -> Ty<TF> {
        Ty::Apply(self)
    }
}

impl<TF: TypeFamily> Cast<Ty<TF>> for ProjectionTy<TF> {
    fn cast(self) -> Ty<TF> {
        Ty::Projection(self)
    }
}

impl<T, TF> Cast<ProgramClause<TF>> for T
where
    T: Cast<DomainGoal<TF>>,
    TF: TypeFamily,
{
    fn cast(self) -> ProgramClause<TF> {
        ProgramClause::Implies(ProgramClauseImplication {
            consequence: self.cast(),
            conditions: vec![],
        })
    }
}

impl<T, TF> Cast<ProgramClause<TF>> for Binders<T>
where
    T: Cast<DomainGoal<TF>>,
    TF: TypeFamily,
{
    fn cast(self) -> ProgramClause<TF> {
        if self.binders.is_empty() {
            Cast::<ProgramClause<TF>>::cast(self.value)
        } else {
            ProgramClause::ForAll(self.map(|bound| ProgramClauseImplication {
                consequence: bound.cast(),
                conditions: vec![],
            }))
        }
    }
}

impl<TF: TypeFamily> Cast<ProgramClause<TF>> for ProgramClauseImplication<TF> {
    fn cast(self) -> ProgramClause<TF> {
        ProgramClause::Implies(self)
    }
}

impl<TF: TypeFamily> Cast<ProgramClause<TF>> for Binders<ProgramClauseImplication<TF>> {
    fn cast(self) -> ProgramClause<TF> {
        ProgramClause::ForAll(self)
    }
}

macro_rules! map_impl {
    (impl[$($t:tt)*] Cast<$b:ty> for $a:ty) => {
        impl<$($t)*> Cast<$b> for $a {
            fn cast(self) -> $b {
                self.map(|v| v.cast())
            }
        }
    }
}

map_impl!(impl[T: Cast<U>, U] Cast<Option<U>> for Option<T>);
map_impl!(impl[
    T: HasTypeFamily<TypeFamily = TF> + Cast<U>,
    U: HasTypeFamily<TypeFamily = TF>,
    TF: TypeFamily,
] Cast<InEnvironment<U>> for InEnvironment<T>);
map_impl!(impl[T: Cast<U>, U, E] Cast<Result<U, E>> for Result<T, E>);

impl<T, U> Cast<Canonical<U>> for Canonical<T>
where
    T: Cast<U>,
{
    fn cast(self) -> Canonical<U> {
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

impl<T, U> Cast<Vec<U>> for Vec<T>
where
    T: Cast<U>,
{
    fn cast(self) -> Vec<U> {
        self.into_iter().casted().collect()
    }
}

pub struct Casted<I, U> {
    iterator: I,
    _cast: PhantomData<U>,
}

impl<I: Iterator, U> Iterator for Casted<I, U>
where
    I::Item: Cast<U>,
{
    type Item = U;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next().map(|item| item.cast())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iterator.size_hint()
    }
}

/// An iterator adapter that casts each element we are iterating over
/// to some other type.
pub trait Caster<U>: Sized {
    fn casted(self) -> Casted<Self, U>;
}

impl<I: Iterator, U> Caster<U> for I {
    fn casted(self) -> Casted<Self, U> {
        Casted {
            iterator: self,
            _cast: PhantomData,
        }
    }
}
