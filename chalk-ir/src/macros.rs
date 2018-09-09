//! Useful macros for writing unit tests. They let you gin up dummy types and things.

#[macro_export]
macro_rules! ty {
    (apply $n:tt $($arg:tt)*) => {
        $crate::Ty::Apply(ApplicationTy {
            name: ty_name!($n),
            parameters: vec![$(arg!($arg)),*],
        })
    };

    (for_all $n:tt $t:tt) => {
        $crate::Ty::ForAll(Box::new(QuantifiedTy {
            num_binders: $n,
            ty: ty!($t),
        }))
    };

    (projection (item $n:tt) $($arg:tt)*) => {
        $crate::Ty::Projection(ProjectionTy {
            associated_ty_id: ItemId { index: $n },
            parameters: vec![$(arg!($arg)),*],
        })
    };

    (var $b:expr) => {
        $crate::Ty::Var($b)
    };

    (expr $b:expr) => {
        $b.clone()
    };

    (($($b:tt)*)) => {
        ty!($($b)*)
    };
}

#[macro_export]
macro_rules! arg {
    ((lifetime $b:tt)) => {
        $crate::ParameterKind::Lifetime(lifetime!($b))
    };

    ($arg:tt) => {
        $crate::ParameterKind::Ty(ty!($arg))
    };
}

#[macro_export]
macro_rules! lifetime {
    (var $b:expr) => {
        $crate::Lifetime::Var($b)
    };

    (skol $b:expr) => {
        $crate::Lifetime::ForAll(UniverseIndex { counter: $b })
    };

    (expr $b:expr) => {
        $b.clone()
    };

    (($($b:tt)*)) => {
        lifetime!($($b)*)
    };
}

#[macro_export]
macro_rules! ty_name {
    ((item $n:expr)) => { $crate::TypeName::ItemId(ItemId { index: $n }) };
    ((skol $n:expr)) => { $crate::TypeName::ForAll(UniverseIndex { counter: $n }) }
}
