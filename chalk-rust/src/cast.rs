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
reflexive_impl!(WhereClauseGoal);

impl Cast<WhereClause> for TraitRef {
    fn cast(self) -> WhereClause {
        WhereClause::Implemented(self)
    }
}

impl Cast<WhereClauseGoal> for TraitRef {
    fn cast(self) -> WhereClauseGoal {
        WhereClauseGoal::Implemented(self)
    }
}

impl Cast<WhereClause> for Normalize {
    fn cast(self) -> WhereClause {
        WhereClause::Normalize(self)
    }
}

impl Cast<WhereClauseGoal> for Normalize {
    fn cast(self) -> WhereClauseGoal {
        WhereClauseGoal::Normalize(self)
    }
}

impl Cast<WhereClauseGoal> for WhereClause {
    fn cast(self) -> WhereClauseGoal {
        match self {
            WhereClause::Implemented(a) => a.cast(),
            WhereClause::Normalize(a) => a.cast(),
        }
    }
}

impl Cast<Goal> for TraitRef {
    fn cast(self) -> Goal {
        Goal::Leaf(self.cast())
    }
}

impl Cast<Goal> for WhereClause {
    fn cast(self) -> Goal {
        Goal::Leaf(self.cast())
    }
}

impl Cast<WhereClauseGoal> for Unify<Ty> {
    fn cast(self) -> WhereClauseGoal {
        WhereClauseGoal::UnifyTys(self)
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
map_impl!(impl[T: Cast<U>, U] Cast<Constrained<U>> for Constrained<T>);
map_impl!(impl[T: Cast<U>, U, E] Cast<Result<U, E>> for Result<T, E>);

impl<T, U> Cast<Vec<U>> for Vec<T>
    where T: Cast<U>
{
    fn cast(self) -> Vec<U> {
        self.into_iter().map(|v| v.cast()).collect()
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
