//! Useful macros for writing unit tests. They let you gin up dummy types and things.

#[macro_export]
macro_rules! ty {
    (apply $n:tt $($arg:tt)*) => {
        $crate::TyData::Apply(ApplicationTy {
            name: ty_name!($n),
            parameters: vec![$(arg!($arg)),*],
        }).intern()
    };

    (for_all $n:tt $t:tt) => {
        $crate::TyData::ForAll(Box::new(QuantifiedTy {
            num_binders: $n,
            ty: ty!($t),
        })).intern()
    };

    (projection (item $n:tt) $($arg:tt)*) => {
        $crate::TyData::Projection(ProjectionTy {
            associated_ty_id: TypeId(RawId { index: $n }),
            parameters: vec![$(arg!($arg)),*],
        }).intern()
    };

    (infer $b:expr) => {
        $crate::TyData::InferenceVar($crate::InferenceVar::from($b)).intern()
    };

    (bound $b:expr) => {
        $crate::TyData::BoundVar($b).intern()
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
