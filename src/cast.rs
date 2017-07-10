use ir::*;
use std::marker::PhantomData;

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

reflexive_impl!(PolarizedTraitRef);
reflexive_impl!(LeafGoal);
reflexive_impl!(DomainGoal);

impl Cast<DomainGoal> for PolarizedTraitRef {
    fn cast(self) -> DomainGoal {
        DomainGoal::Implemented(self)
    }
}

impl Cast<LeafGoal> for PolarizedTraitRef {
    fn cast(self) -> LeafGoal {
        LeafGoal::DomainGoal(self.cast())
    }
}

impl Cast<DomainGoal> for Normalize {
    fn cast(self) -> DomainGoal {
        DomainGoal::Normalize(self)
    }
}

impl Cast<LeafGoal> for Normalize {
    fn cast(self) -> LeafGoal {
        LeafGoal::DomainGoal(self.cast())
    }
}

impl Cast<DomainGoal> for WellFormed {
    fn cast(self) -> DomainGoal {
        DomainGoal::WellFormed(self)
    }
}

impl Cast<LeafGoal> for WellFormed {
    fn cast(self) -> LeafGoal {
        LeafGoal::DomainGoal(self.cast())
    }
}

impl Cast<Goal> for WellFormed {
    fn cast(self) -> Goal {
        let wcg: LeafGoal = self.cast();
        wcg.cast()
    }
}

impl Cast<Goal> for Normalize {
    fn cast(self) -> Goal {
        let wcg: LeafGoal = self.cast();
        wcg.cast()
    }
}

impl Cast<LeafGoal> for DomainGoal {
    fn cast(self) -> LeafGoal {
        LeafGoal::DomainGoal(self)
    }
}

impl Cast<Goal> for PolarizedTraitRef {
    fn cast(self) -> Goal {
        Goal::Leaf(self.cast())
    }
}

impl Cast<Goal> for DomainGoal {
    fn cast(self) -> Goal {
        Goal::Leaf(self.cast())
    }
}

impl Cast<Goal> for LeafGoal {
    fn cast(self) -> Goal {
        Goal::Leaf(self)
    }
}

impl Cast<LeafGoal> for EqGoal {
    fn cast(self) -> LeafGoal {
        LeafGoal::EqGoal(self)
    }
}

impl Cast<Ty> for ApplicationTy {
    fn cast(self) -> Ty {
        Ty::Apply(self)
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
map_impl!(impl[T: Cast<U>, U] Cast<Canonical<U>> for Canonical<T>);
map_impl!(impl[T: Cast<U>, U] Cast<InEnvironment<U>> for InEnvironment<T>);
map_impl!(impl[T: Cast<U>, U, E] Cast<Result<U, E>> for Result<T, E>);

impl<T, U> Cast<Vec<U>> for Vec<T>
    where T: Cast<U>
{
    fn cast(self) -> Vec<U> {
        self.into_iter().casted().collect()
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

pub struct Casted<I, U> {
    iterator: I,
    _cast: PhantomData<U>,
}

impl<I: Iterator, U> Iterator for Casted<I, U> where I::Item: Cast<U> {
    type Item = U;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next().map(|item| item.cast())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iterator.size_hint()
    }
}

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
