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

reflexive_impl!(TraitRef);
reflexive_impl!(LeafGoal);
reflexive_impl!(DomainGoal);
reflexive_impl!(WhereClause);

impl Cast<WhereClause> for TraitRef {
    fn cast(self) -> WhereClause {
        WhereClause::Implemented(self)
    }
}

impl Cast<WhereClause> for ProjectionEq {
    fn cast(self) -> WhereClause {
        WhereClause::ProjectionEq(self)
    }
}

impl<T> Cast<DomainGoal> for T
where
    T: Cast<WhereClause>,
{
    fn cast(self) -> DomainGoal {
        DomainGoal::Holds(self.cast())
    }
}

impl<T> Cast<LeafGoal> for T
where
    T: Cast<DomainGoal>,
{
    fn cast(self) -> LeafGoal {
        LeafGoal::DomainGoal(self.cast())
    }
}

impl<T> Cast<Goal> for T
where
    T: Cast<LeafGoal>,
{
    fn cast(self) -> Goal {
        Goal::Leaf(self.cast())
    }
}

impl Cast<DomainGoal> for Normalize {
    fn cast(self) -> DomainGoal {
        DomainGoal::Normalize(self)
    }
}

impl Cast<DomainGoal> for UnselectedNormalize {
    fn cast(self) -> DomainGoal {
        DomainGoal::UnselectedNormalize(self)
    }
}

impl Cast<DomainGoal> for WellFormed {
    fn cast(self) -> DomainGoal {
        DomainGoal::WellFormed(self)
    }
}

impl Cast<DomainGoal> for FromEnv {
    fn cast(self) -> DomainGoal {
        DomainGoal::FromEnv(self)
    }
}

impl Cast<LeafGoal> for EqGoal {
    fn cast(self) -> LeafGoal {
        LeafGoal::EqGoal(self)
    }
}

impl<T: Cast<Goal>> Cast<Goal> for Binders<T> {
    fn cast(self) -> Goal {
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

impl Cast<Ty> for ApplicationTy {
    fn cast(self) -> Ty {
        Ty::Apply(self)
    }
}

impl Cast<Ty> for ProjectionTy {
    fn cast(self) -> Ty {
        Ty::Projection(self)
    }
}

impl Cast<Parameter> for Ty {
    fn cast(self) -> Parameter {
        ParameterKind::Ty(self)
    }
}

impl Cast<Parameter> for Lifetime {
    fn cast(self) -> Parameter {
        ParameterKind::Lifetime(self)
    }
}

impl<T> Cast<ProgramClause> for T
where
    T: Cast<DomainGoal>,
{
    fn cast(self) -> ProgramClause {
        ProgramClause::Implies(ProgramClauseImplication {
            consequence: self.cast(),
            conditions: vec![],
        })
    }
}

impl<T: Cast<DomainGoal>> Cast<ProgramClause> for Binders<T> {
    fn cast(self) -> ProgramClause {
        if self.binders.is_empty() {
            Cast::<ProgramClause>::cast(self.value)
        } else {
            ProgramClause::ForAll(self.map(|bound| ProgramClauseImplication {
                consequence: bound.cast(),
                conditions: vec![],
            }))
        }
    }
}

impl Cast<ProgramClause> for ProgramClauseImplication {
    fn cast(self) -> ProgramClause {
        ProgramClause::Implies(self)
    }
}

impl Cast<ProgramClause> for Binders<ProgramClauseImplication> {
    fn cast(self) -> ProgramClause {
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
map_impl!(impl[T: Cast<U>, U] Cast<InEnvironment<U>> for InEnvironment<T>);
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
