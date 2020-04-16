//! Useful macros for writing unit tests. They let you gin up dummy types and things.

#[macro_export]
macro_rules! ty {
    (apply $n:tt $($arg:tt)*) => {
        $crate::TyData::Apply(ApplicationTy {
            name: ty_name!($n),
            substitution: $crate::Substitution::from(&chalk_ir::interner::ChalkIr, vec![$(arg!($arg)),*] as Vec<$crate::Parameter<_>>),
        }).intern(&chalk_ir::interner::ChalkIr)
    };

    (function $n:tt $($arg:tt)*) => {
        $crate::TyData::Function(Fn {
            num_binders: $n,
            substitution: $crate::Substitution::from(&chalk_ir::interner::ChalkIr, vec![$(arg!($arg)),*] as Vec<$crate::Parameter<_>>),
        }).intern(&chalk_ir::interner::ChalkIr)
    };

    (placeholder $n:expr) => {
        $crate::TyData::Placeholder(PlaceholderIndex {
            ui: UniverseIndex { counter: $n },
            idx: 0,
        }).intern(&chalk_ir::interner::ChalkIr)
    };

    (projection (item $n:tt) $($arg:tt)*) => {
            chalk_ir::AliasTy::Projection(chalk_ir::ProjectionTy  {
            associated_ty_id: AssocTypeId(chalk_ir::interner::RawId { index: $n }),
            substitution: $crate::Substitution::from(&chalk_ir::interner::ChalkIr, vec![$(arg!($arg)),*] as Vec<$crate::Parameter<_>>),
        }).intern(&chalk_ir::interner::ChalkIr)
    };

    (infer $b:expr) => {
        $crate::TyData::InferenceVar($crate::InferenceVar::from($b)).intern(&chalk_ir::interner::ChalkIr)
    };

    (bound $d:tt $b:tt) => {
        $crate::TyData::BoundVar($crate::BoundVar::new($crate::DebruijnIndex::new($d), $b)).intern(&chalk_ir::interner::ChalkIr)
    };

    (bound $b:expr) => {
        $crate::TyData::BoundVar($crate::BoundVar::new($crate::DebruijnIndex::INNERMOST, $b)).intern(&chalk_ir::interner::ChalkIr)
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
        $crate::Parameter::new(
            &chalk_ir::interner::ChalkIr,
            $crate::ParameterKind::Lifetime(lifetime!($b)),
        )
    };

    ($arg:tt) => {
        $crate::Parameter::new(
            &chalk_ir::interner::ChalkIr,
            $crate::ParameterKind::Ty(ty!($arg)),
        )
    };
}

#[macro_export]
macro_rules! lifetime {
    (infer $b:expr) => {
        $crate::LifetimeData::InferenceVar($crate::InferenceVar::from($b)).intern(&chalk_ir::interner::ChalkIr)
    };

    (bound $d:tt $b:tt) => {
        $crate::LifetimeData::BoundVar($crate::BoundVar::new($crate::DebruijnIndex::new($d), $b)).intern(&chalk_ir::interner::ChalkIr)
    };

    (bound $b:expr) => {
        $crate::LifetimeData::BoundVar($crate::BoundVar::new($crate::DebruijnIndex::INNERMOST, $b)).intern(&chalk_ir::interner::ChalkIr)
    };

    (placeholder $b:expr) => {
        $crate::LifetimeData::Placeholder(PlaceholderIndex { ui: UniverseIndex { counter: $b }, idx: 0}).intern(&chalk_ir::interner::ChalkIr)
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
