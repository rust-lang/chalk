use ir::*;
use solve::Solution;
use solve::environment::InEnvironment;

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
reflexive_impl!(WhereClause);

impl Cast<WhereClause> for TraitRef {
    fn cast(self) -> WhereClause {
        WhereClause::Implemented(self)
    }
}

impl Cast<WhereClause> for NormalizeTo {
    fn cast(self) -> WhereClause {
        WhereClause::NormalizeTo(self)
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
map_impl!(impl[T: Cast<U>, U] Cast<Quantified<U>> for Quantified<T>);
map_impl!(impl[T: Cast<U>, U] Cast<Solution<U>> for Solution<T>);
map_impl!(impl[T: Cast<U>, U] Cast<InEnvironment<U>> for InEnvironment<T>);
map_impl!(impl[T: Cast<U>, U, E] Cast<Result<U, E>> for Result<T, E>);

impl<T, U> Cast<Vec<U>> for Vec<T>
    where T: Cast<U>
{
    fn cast(self) -> Vec<U> {
        self.into_iter().map(|v| v.cast()).collect()
    }
}
