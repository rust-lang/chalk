//! Useful macros for writing unit tests. They let you gin up dummy types and things.

#[cfg(test)]
macro_rules! ty {
    (apply $n:tt $($arg:tt)*) => {
        ::ir::Ty::Apply(ApplicationTy {
            name: ty_name!($n),
            parameters: vec![$(arg!($arg)),*],
        })
    };

    (for_all $n:tt $t:tt) => {
        ::ir::Ty::ForAll(Box::new(QuantifiedTy {
            num_binders: $n,
            ty: ty!($t),
        }))
    };

    (projection (item $n:tt) $($arg:tt)*) => {
        ::ir::Ty::Projection(ProjectionTy {
            associated_ty_id: ItemId { index: $n },
            parameters: vec![$(arg!($arg)),*],
        })
    };

    (var $b:expr) => {
        ::ir::Ty::Var($b)
    };

    (expr $b:expr) => {
        $b.clone()
    };

    (($($b:tt)*)) => {
        ty!($($b)*)
    };
}

#[cfg(test)]
macro_rules! arg {
    ((lifetime $b:tt)) => {
        ::ir::ParameterKind::Lifetime(lifetime!($b))
    };

    ($arg:tt) => {
        ::ir::ParameterKind::Ty(ty!($arg))
    };
}

#[cfg(test)]
macro_rules! lifetime {
    (var $b:expr) => {
        ::ir::Lifetime::Var($b)
    };

    (expr $b:expr) => {
        $b.clone()
    };

    (($($b:tt)*)) => {
        lifetime!($($b)*)
    };
}

#[cfg(test)]
macro_rules! ty_name {
    ((item $n:expr)) => { ::ir::TypeName::ItemId(ItemId { index: $n }) };
    ((skol $n:expr)) => { ::ir::TypeName::ForAll(UniverseIndex { counter: $n }) }
}

