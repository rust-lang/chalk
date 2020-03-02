//! Useful macros for writing unit tests. They let you gin up dummy types and things.

#[macro_export]
macro_rules! ty {
    (apply $n:tt $($arg:tt)*) => {
        $crate::TyData::Apply(ApplicationTy {
            name: ty_name!($n),
            substitution: $crate::Substitution::from(vec![$(arg!($arg)),*] as Vec<$crate::Parameter<_>>),
        }).intern()
    };

    (function $n:tt $($arg:tt)*) => {
        $crate::TyData::Function(Fn {
            num_binders: $n,
            parameters: vec![$(arg!($arg)),*],
        }).intern()
    };

    (placeholder $n:expr) => {
        $crate::TyData::Placeholder(PlaceholderIndex {
            ui: UniverseIndex { counter: $n },
            idx: 0,
        }).intern()
    };

    (alias (item $n:tt) $($arg:tt)*) => {
        $crate::TyData::Alias(AliasTy {
            associated_ty_id: AssocTypeId(chalk_ir::interner::RawId { index: $n }),
            substitution: $crate::Substitution::from(vec![$(arg!($arg)),*] as Vec<$crate::Parameter<_>>),
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
        $crate::Parameter::new($crate::ParameterKind::Lifetime(lifetime!($b)))
    };

    ($arg:tt) => {
        $crate::Parameter::new($crate::ParameterKind::Ty(ty!($arg)))
    };
}

#[macro_export]
macro_rules! lifetime {
    (infer $b:expr) => {
        $crate::LifetimeData::InferenceVar($crate::InferenceVar::from($b)).intern()
    };

    (bound $b:expr) => {
        $crate::LifetimeData::BoundVar($b).intern()
    };

    (placeholder $b:expr) => {
        $crate::LifetimeData::Placeholder(PlaceholderIndex { ui: UniverseIndex { counter: $b }, idx: 0}).intern()
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
        $crate::TypeName::Struct(StructId(chalk_ir::interner::RawId { index: $n }))
    };
}
