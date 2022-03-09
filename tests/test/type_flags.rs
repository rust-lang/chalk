use chalk_integration::interner::ChalkIr;
use chalk_integration::{empty_substitution, lifetime, ty};
use chalk_ir::cast::Cast;
use chalk_ir::{PlaceholderIndex, TyKind, TypeFlags, UniverseIndex};

#[test]
fn placeholder_ty_flags_correct() {
    let placeholder_ty = ty!(placeholder 0);
    assert_eq!(
        placeholder_ty.data(ChalkIr).flags,
        TypeFlags::HAS_TY_PLACEHOLDER
    );
}

#[test]
fn opaque_ty_flags_correct() {
    let opaque_ty = TyKind::Alias(chalk_ir::AliasTy::Opaque(chalk_ir::OpaqueTy {
        opaque_ty_id: chalk_ir::OpaqueTyId(chalk_integration::interner::RawId { index: 0 }),
        substitution: chalk_ir::Substitution::from_iter(
            ChalkIr,
            Some(
                chalk_ir::ConstData {
                    ty: TyKind::Placeholder(PlaceholderIndex {
                        ui: chalk_ir::UniverseIndex::ROOT,
                        idx: 0,
                    })
                    .intern(ChalkIr),
                    value: chalk_ir::ConstValue::InferenceVar(chalk_ir::InferenceVar::from(0)),
                }
                .intern(ChalkIr)
                .cast(ChalkIr),
            ),
        ),
    }))
    .intern(ChalkIr);
    assert_eq!(
        opaque_ty.data(ChalkIr).flags,
        TypeFlags::HAS_TY_OPAQUE
            | TypeFlags::HAS_CT_INFER
            | TypeFlags::STILL_FURTHER_SPECIALIZABLE
            | TypeFlags::HAS_TY_PLACEHOLDER
    );
}

#[test]
fn dyn_ty_flags_correct() {
    let internal_ty = TyKind::Scalar(chalk_ir::Scalar::Bool).intern(ChalkIr);
    let projection_ty = chalk_ir::ProjectionTy {
        associated_ty_id: chalk_ir::AssocTypeId(chalk_integration::interner::RawId { index: 0 }),
        substitution: empty_substitution!(),
    };
    let bounds = chalk_ir::Binders::<chalk_ir::QuantifiedWhereClauses<ChalkIr>>::empty(
        ChalkIr,
        chalk_ir::QuantifiedWhereClauses::from_iter(
            ChalkIr,
            vec![chalk_ir::Binders::<chalk_ir::WhereClause<ChalkIr>>::empty(
                ChalkIr,
                chalk_ir::WhereClause::AliasEq(chalk_ir::AliasEq {
                    ty: internal_ty,
                    alias: chalk_ir::AliasTy::Projection(projection_ty),
                }),
            )],
        ),
    );
    let dyn_ty = chalk_ir::DynTy {
        lifetime: lifetime!(placeholder 5),
        bounds,
    };
    let ty = TyKind::Dyn(dyn_ty).intern(ChalkIr);
    assert_eq!(
        ty.data(ChalkIr).flags,
        TypeFlags::HAS_TY_PROJECTION
            | TypeFlags::HAS_RE_PLACEHOLDER
            | TypeFlags::HAS_FREE_LOCAL_REGIONS
            | TypeFlags::HAS_FREE_REGIONS
    );
}

#[test]
fn flagless_ty_has_no_flags() {
    let ty = TyKind::Str.intern(ChalkIr);
    assert_eq!(ty.data(ChalkIr).flags, TypeFlags::empty());

    let fn_ty = TyKind::Function(chalk_ir::FnPointer {
        num_binders: 0,
        substitution: chalk_ir::FnSubst(empty_substitution!()),
        sig: chalk_ir::FnSig {
            abi: chalk_integration::interner::ChalkFnAbi::Rust,
            safety: chalk_ir::Safety::Safe,
            variadic: false,
        },
    })
    .intern(ChalkIr);
    assert_eq!(fn_ty.data(ChalkIr).flags, TypeFlags::empty());
}

#[test]
fn static_and_bound_lifetimes() {
    let substitutions = chalk_ir::Substitution::from_iter(
        ChalkIr,
        vec![
            chalk_ir::GenericArgData::Lifetime(chalk_ir::LifetimeData::Static.intern(ChalkIr))
                .intern(ChalkIr),
            chalk_ir::GenericArgData::Lifetime(lifetime!(bound 5)).intern(ChalkIr),
        ],
    );

    let ty = TyKind::Adt(
        chalk_ir::AdtId(chalk_integration::interner::RawId { index: 0 }),
        substitutions,
    )
    .intern(ChalkIr);

    assert_eq!(
        ty.data(ChalkIr).flags,
        TypeFlags::HAS_FREE_REGIONS | TypeFlags::HAS_RE_LATE_BOUND
    );
}
