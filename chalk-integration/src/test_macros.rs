//! Useful macros for writing unit tests. They let you gin up dummy types and things.

#[macro_export]
macro_rules! ty {
    (apply (item $n:expr) $($arg:tt)*) => {
        chalk_ir::TyKind::Adt(
            chalk_ir::AdtId(chalk_integration::interner::RawId {
                index: $n,

            }),
            chalk_ir::Substitution::from_iter(
                chalk_integration::interner::ChalkIr,
                vec![$(arg!($arg)),*] as Vec<chalk_ir::GenericArg<_>>
            ),
        )
        .intern(chalk_integration::interner::ChalkIr)
    };

    (function $n:tt $($arg:tt)*) => {
        chalk_ir::TyKind::Function(chalk_ir::FnPointer {
            num_binders: $n,
            substitution: chalk_ir::FnSubst(chalk_ir::Substitution::from_iter(
                chalk_integration::interner::ChalkIr,
                vec![$(arg!($arg)),*] as Vec<chalk_ir::GenericArg<_>>
            )),
            sig: chalk_ir::FnSig {
                safety: chalk_ir::Safety::Safe,
                abi: <chalk_integration::interner::ChalkIr as chalk_ir::interner::Interner>::FnAbi::Rust,
                variadic: false,
            }
        }).intern(chalk_integration::interner::ChalkIr)
    };

    (placeholder $n:expr) => {
        chalk_ir::TyKind::Placeholder(PlaceholderIndex {
            ui: UniverseIndex { counter: $n },
            idx: 0,
        }).intern(chalk_integration::interner::ChalkIr)
    };

    (projection (item $n:tt) $($arg:tt)*) => {
            chalk_ir::AliasTy::Projection(chalk_ir::ProjectionTy  {
            associated_ty_id: AssocTypeId(chalk_integration::interner::RawId { index: $n }),
            substitution: chalk_ir::Substitution::from_iter(
                chalk_integration::interner::ChalkIr,
                vec![$(arg!($arg)),*] as Vec<chalk_ir::GenericArg<_>>
            ),
        }).intern(chalk_integration::interner::ChalkIr)
    };

    (infer $b:expr) => {
        chalk_ir::TyKind::InferenceVar(chalk_ir::InferenceVar::from($b), chalk_ir::TyVariableKind::General)
            .intern(chalk_integration::interner::ChalkIr)
    };

    (bound $d:tt $b:tt) => {
        chalk_ir::TyKind::BoundVar(chalk_ir::BoundVar::new(chalk_ir::DebruijnIndex::new($d), $b))
            .intern(chalk_integration::interner::ChalkIr)
    };

    (bound $b:expr) => {
        chalk_ir::TyKind::BoundVar(chalk_ir::BoundVar::new(chalk_ir::DebruijnIndex::INNERMOST, $b))
            .intern(chalk_integration::interner::ChalkIr)
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
        chalk_ir::GenericArg::new(
            chalk_integration::interner::ChalkIr,
            chalk_ir::GenericArgData::Lifetime(lifetime!($b)),
        )
    };

    ($arg:tt) => {
        chalk_ir::GenericArg::new(
            chalk_integration::interner::ChalkIr,
            chalk_ir::GenericArgData::Ty(ty!($arg)),
        )
    };
}

#[macro_export]
macro_rules! lifetime {
    (infer $b:expr) => {
        chalk_ir::LifetimeData::InferenceVar(chalk_ir::InferenceVar::from($b))
            .intern(chalk_integration::interner::ChalkIr)
    };

    (bound $d:tt $b:tt) => {
        chalk_ir::LifetimeData::BoundVar(chalk_ir::BoundVar::new(chalk_ir::DebruijnIndex::new($d), $b))
            .intern(chalk_integration::interner::ChalkIr)
    };

    (bound $b:expr) => {
        chalk_ir::LifetimeData::BoundVar(chalk_ir::BoundVar::new(chalk_ir::DebruijnIndex::INNERMOST, $b))
            .intern(chalk_integration::interner::ChalkIr)
    };

    (placeholder $b:expr) => {
        chalk_ir::LifetimeData::Placeholder(PlaceholderIndex { ui: UniverseIndex { counter: $b }, idx: 0})
            .intern(chalk_integration::interner::ChalkIr)
    };

    (expr $b:expr) => {
        $b.clone()
    };

    (($($b:tt)*)) => {
        lifetime!($($b)*)
    };
}

#[macro_export]
macro_rules! empty_substitution {
    () => {
        chalk_ir::Substitution::empty(chalk_integration::interner::ChalkIr)
    };
}
