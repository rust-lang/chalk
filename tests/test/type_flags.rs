use chalk_integration::{empty_substitution, lifetime, ty};
use chalk_ir::{PlaceholderIndex, Ty, TyKind, UniverseIndex};

#[test]
fn placeholder_ty_flags_correct() {
    let placeholder_ty = ty!(placeholder 0);
    assert_eq!(
        placeholder_ty
            .data(&chalk_integration::interner::ChalkIr)
            .flags,
        chalk_ir::TypeFlags::HAS_TY_PLACEHOLDER
    );
}

#[test]
fn opaque_ty_flags_correct() {
    let x: Vec<chalk_ir::GenericArg<chalk_integration::interner::ChalkIr>> =
        vec![chalk_ir::GenericArg::new(
            &chalk_integration::interner::ChalkIr,
            chalk_ir::GenericArgData::Const(chalk_ir::Const::new(
                &chalk_integration::interner::ChalkIr,
                chalk_ir::ConstData {
                    ty: Ty::new(
                        &chalk_integration::interner::ChalkIr,
                        TyKind::Placeholder(PlaceholderIndex {
                            ui: chalk_ir::UniverseIndex::ROOT,
                            idx: 0,
                        }),
                    ),
                    value: chalk_ir::ConstValue::InferenceVar(chalk_ir::InferenceVar::from(0)),
                },
            )),
        )];
    let opaque_ty_kind = TyKind::OpaqueType(
        chalk_ir::OpaqueTyId {
            0: chalk_integration::interner::RawId { index: 0 },
        },
        chalk_ir::Substitution::from_iter(&chalk_integration::interner::ChalkIr, x),
    );
    let opaque_ty = Ty::new(&chalk_integration::interner::ChalkIr, opaque_ty_kind);
    assert_eq!(
        opaque_ty.data(&chalk_integration::interner::ChalkIr).flags,
        chalk_ir::TypeFlags::HAS_TY_OPAQUE
            | chalk_ir::TypeFlags::HAS_CT_INFER
            | chalk_ir::TypeFlags::STILL_FURTHER_SPECIALIZABLE
            | chalk_ir::TypeFlags::HAS_TY_PLACEHOLDER
    );
}

#[test]
fn dyn_ty_flags_correct() {
    let internal_ty = Ty::new(
        &chalk_integration::interner::ChalkIr,
        chalk_ir::TyKind::Scalar(chalk_ir::Scalar::Bool),
    );
    let projection_ty = chalk_ir::ProjectionTy {
        associated_ty_id: chalk_ir::AssocTypeId {
            0: chalk_integration::interner::RawId { index: 0 },
        },
        substitution: empty_substitution!(),
    };
    let bounds = chalk_ir::Binders::<
        chalk_ir::QuantifiedWhereClauses<chalk_integration::interner::ChalkIr>,
    >::empty(
        &chalk_integration::interner::ChalkIr,
        chalk_ir::QuantifiedWhereClauses::from_iter(
            &chalk_integration::interner::ChalkIr,
            vec![chalk_ir::Binders::<
                chalk_ir::WhereClause<chalk_integration::interner::ChalkIr>,
            >::empty(
                &chalk_integration::interner::ChalkIr,
                chalk_ir::WhereClause::AliasEq(chalk_ir::AliasEq {
                    ty: internal_ty,
                    alias: chalk_ir::AliasTy::Projection(projection_ty),
                }),
            )],
        ),
    );
    let dyn_ty = chalk_ir::DynTy {
        lifetime: lifetime!(placeholder 5),
        bounds: bounds,
    };
    let ty = chalk_ir::Ty::new(&chalk_integration::interner::ChalkIr, TyKind::Dyn(dyn_ty));
    assert_eq!(
        ty.data(&chalk_integration::interner::ChalkIr).flags,
        chalk_ir::TypeFlags::HAS_TY_PROJECTION
            | chalk_ir::TypeFlags::HAS_RE_PLACEHOLDER
            | chalk_ir::TypeFlags::HAS_FREE_LOCAL_REGIONS
            | chalk_ir::TypeFlags::HAS_FREE_REGIONS
    );
}

#[test]
fn flagless_ty_has_no_flags() {
    let ty = Ty::new(&chalk_integration::interner::ChalkIr, TyKind::Str);
    assert_eq!(
        ty.data(&chalk_integration::interner::ChalkIr).flags,
        chalk_ir::TypeFlags::empty()
    );

    let fn_ty = Ty::new(
        &chalk_integration::interner::ChalkIr,
        TyKind::Function(chalk_ir::FnPointer {
            num_binders: 0,
            substitution: empty_substitution!(),
            sig: chalk_ir::FnSig {
                abi: chalk_integration::interner::ChalkFnAbi::Rust,
                safety: chalk_ir::Safety::Safe,
                variadic: false,
            },
        }),
    );
    assert_eq!(
        fn_ty.data(&chalk_integration::interner::ChalkIr).flags,
        chalk_ir::TypeFlags::empty()
    );
}

#[test]
fn flagless_lifetime_contributes_no_flags() {
    let substitutions = chalk_ir::Substitution::from_iter(
        &chalk_integration::interner::ChalkIr,
        vec![
            chalk_ir::GenericArg::new(
                &chalk_integration::interner::ChalkIr,
                chalk_ir::GenericArgData::Lifetime(chalk_ir::Lifetime::new(
                    &chalk_integration::interner::ChalkIr,
                    chalk_ir::LifetimeData::Static,
                )),
            ),
            chalk_ir::GenericArg::new(
                &chalk_integration::interner::ChalkIr,
                chalk_ir::GenericArgData::Lifetime(lifetime!(bound 5)),
            ),
        ],
    );

    let ty = Ty::new(
        &chalk_integration::interner::ChalkIr,
        TyKind::Adt(
            chalk_ir::AdtId {
                0: chalk_integration::interner::RawId { index: 0 },
            },
            substitutions,
        ),
    );

    assert_eq!(
        ty.data(&chalk_integration::interner::ChalkIr).flags,
        chalk_ir::TypeFlags::empty()
    );
}
