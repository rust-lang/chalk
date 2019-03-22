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
            associated_ty_id: TypeId(RawId { index: $n }),
            parameters: vec![$(arg!($arg)),*],
        })
    };

    (infer $b:expr) => {
        $crate::Ty::InferenceVar($crate::InferenceVar::from($b))
    };

    (bound $b:expr) => {
        $crate::Ty::BoundVar($b)
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
        $crate::Parameter($crate::ParameterKind::Lifetime(lifetime!($b)))
    };

    ($arg:tt) => {
        $crate::Parameter($crate::ParameterKind::Ty(ty!($arg)))
    };
}

#[macro_export]
macro_rules! lifetime {
    (infer $b:expr) => {
        $crate::Lifetime::InferenceVar($crate::InferenceVar::from($b))
    };

    (bound $b:expr) => {
        $crate::Lifetime::BoundVar($b)
    };

    (placeholder $b:expr) => {
        $crate::Lifetime::Placeholder(PlaceholderIndex { ui: UniverseIndex { counter: $b }, idx: 0})
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
    ((item $n:expr)) => {
        $crate::TypeName::TypeKindId(TypeKindId::TypeId(TypeId(RawId { index: $n })))
    };
    ((placeholder $n:expr)) => {
        $crate::TypeName::Placeholder(PlaceholderIndex {
            ui: UniverseIndex { counter: $n },
            idx: 0,
        })
    };
}
