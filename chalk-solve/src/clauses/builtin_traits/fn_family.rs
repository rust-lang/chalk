use crate::clauses::ClauseBuilder;
use crate::rust_ir::{ClosureKind, FnDefInputsAndOutputDatum, WellKnownAssocType, WellKnownTrait};
use crate::{Interner, RustIrDatabase, TraitRef};
use chalk_ir::cast::Cast;
use chalk_ir::{
    AliasTy, Binders, Goal, Normalize, ProjectionTy, Safety, Substitution, TraitId, Ty, TyKind,
};

fn push_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    well_known: WellKnownTrait,
    trait_id: TraitId<I>,
    self_ty: Ty<I>,
    arg_sub: Substitution<I>,
    return_type: Ty<I>,
) {
    let interner = db.interner();
    let tupled = TyKind::Tuple(arg_sub.len(interner), arg_sub).intern(interner);
    let substitution =
        Substitution::from_iter(interner, &[self_ty.cast(interner), tupled.cast(interner)]);

    let is_async = matches!(
        well_known,
        WellKnownTrait::AsyncFnOnce | WellKnownTrait::AsyncFnMut | WellKnownTrait::AsyncFn
    );

    if !is_async {
        builder.push_fact(TraitRef {
            trait_id,
            substitution: substitution.clone(),
        });

        // The `Output` type is defined on the `FnOnce`
        if let WellKnownTrait::FnOnce = well_known {
            let trait_datum = db.trait_datum(trait_id);
            assert_eq!(
                trait_datum.associated_ty_ids.len(),
                1,
                "FnOnce trait should have exactly one associated type, found {:?}",
                trait_datum.associated_ty_ids
            );
            // Constructs the alias. For `Fn`, for example, this would look like
            // `Normalize(<fn(A) -> B as FnOnce<(A,)>>::Output -> B)`
            let output_id = trait_datum.associated_ty_ids[0];
            let alias = AliasTy::Projection(ProjectionTy {
                associated_ty_id: output_id,
                substitution,
            });
            builder.push_fact(Normalize {
                alias,
                ty: return_type,
            });
        }
    } else {
        let sync_counterpart = match well_known {
            WellKnownTrait::AsyncFnOnce => db.well_known_trait_id(WellKnownTrait::FnOnce).unwrap(),
            WellKnownTrait::AsyncFnMut => db.well_known_trait_id(WellKnownTrait::FnMut).unwrap(),
            WellKnownTrait::AsyncFn => db.well_known_trait_id(WellKnownTrait::Fn).unwrap(),
            _ => unreachable!(),
        };

        let future = db.well_known_trait_id(WellKnownTrait::Future).unwrap();
        let sync_counterpart = TraitRef {
            trait_id: sync_counterpart,
            substitution: substitution.clone(),
        };
        let output_is_future = TraitRef {
            trait_id: future,
            substitution: Substitution::from1(interner, return_type.clone()),
        };

        // This adds the following clause:
        // `F: AsyncFnX<Arg, Output = O>` :- `F: FnX<Arg, Output: Fut<Output = O>>`
        // Actually, the `<F as AsyncFnX>::Output = O` part is added in the if let expression below.
        builder.push_clause(
            TraitRef {
                trait_id,
                substitution: substitution.clone(),
            },
            [sync_counterpart.clone(), output_is_future.clone()],
        );

        if let WellKnownTrait::AsyncFnOnce = well_known {
            builder.push_bound_ty(|builder, ty| {
                let output_id = db
                    .well_known_assoc_type_id(WellKnownAssocType::AsyncFnOnceOutput)
                    .unwrap();
                let async_alias = AliasTy::Projection(ProjectionTy {
                    associated_ty_id: output_id,
                    substitution,
                });

                let trait_datum = db.trait_datum(future);
                assert_eq!(
                    trait_datum.associated_ty_ids.len(),
                    1,
                    "Future trait should have exactly one associated type, found {:?}",
                    trait_datum.associated_ty_ids
                );
                let output_id = trait_datum.associated_ty_ids[0];
                let future_alias = AliasTy::Projection(ProjectionTy {
                    associated_ty_id: output_id,
                    substitution: Substitution::from1(interner, return_type),
                });

                builder.push_clause(
                    Normalize {
                        alias: async_alias,
                        ty: ty.clone(),
                    },
                    [
                        sync_counterpart.cast::<Goal<_>>(interner),
                        output_is_future.cast(interner),
                        Normalize {
                            alias: future_alias,
                            ty,
                        }
                        .cast(interner),
                    ],
                );
            });
        }
    }
}

fn push_clauses_for_apply<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    well_known: WellKnownTrait,
    trait_id: TraitId<I>,
    self_ty: Ty<I>,
    inputs_and_output: Binders<FnDefInputsAndOutputDatum<I>>,
) {
    let interner = db.interner();
    builder.push_binders(inputs_and_output, |builder, inputs_and_output| {
        let arg_sub = inputs_and_output
            .argument_types
            .iter()
            .cloned()
            .map(|ty| ty.cast(interner));
        let arg_sub = Substitution::from_iter(interner, arg_sub);
        let output_ty = inputs_and_output.return_type;

        push_clauses(
            db, builder, well_known, trait_id, self_ty, arg_sub, output_ty,
        );
    });
}

/// Handles clauses for FnOnce/FnMut/Fn and AsyncFnOnce/AsyncFnMut/AsyncFn.
/// For sync traits, `self_ty` is a function, we push a clause of the form
/// `fn(A1, A2, ..., AN) -> O: FnTrait<(A1, A2, ..., AN)>`, where `FnTrait`
/// is the trait corresponding to `trait_id` (FnOnce/FnMut/Fn)
///
/// If `trait_id` is `FnOnce`, we also push a clause for the output type of the form:
/// `Normalize(<fn(A) -> B as FnOnce<(A,)>>::Output -> B)`
/// We do not add the usual `Implemented(fn(A) -> b as FnOnce<(A,)>` clause
/// as a condition, since we already called `push_fact` with it
///
/// For async traits, we push a clause of the form
/// `F: AsyncFnX<Arg, Output = O>` :- `F: FnX<Arg, Output: Fut<Output = O>>`,
/// which corresponds to the implementation
/// `impl<F, Arg, Fut, O> AsyncFn<A> for F where F: Fn<Arg, Output = Fut>, Fut: Future<Output = O>`.
pub fn add_fn_trait_program_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    well_known: WellKnownTrait,
    self_ty: Ty<I>,
) {
    let interner = db.interner();
    let trait_id = db.well_known_trait_id(well_known).unwrap();

    match self_ty.kind(interner) {
        TyKind::FnDef(fn_def_id, substitution) => {
            let fn_def_datum = builder.db.fn_def_datum(*fn_def_id);
            if fn_def_datum.sig.safety == Safety::Safe && !fn_def_datum.sig.variadic {
                let bound = fn_def_datum
                    .binders
                    .clone()
                    .substitute(builder.interner(), &substitution);
                push_clauses_for_apply(
                    db,
                    builder,
                    well_known,
                    trait_id,
                    self_ty,
                    bound.inputs_and_output,
                );
            }
        }
        TyKind::Closure(closure_id, substitution) => {
            let closure_kind = db.closure_kind(*closure_id, substitution);
            let trait_matches = matches!(
                (well_known, closure_kind),
                (
                    WellKnownTrait::Fn | WellKnownTrait::AsyncFn,
                    ClosureKind::Fn
                ) | (
                    WellKnownTrait::FnMut | WellKnownTrait::AsyncFnMut,
                    ClosureKind::FnMut | ClosureKind::Fn
                ) | (WellKnownTrait::FnOnce | WellKnownTrait::AsyncFnOnce, _)
            );
            if !trait_matches {
                return;
            }
            let closure_inputs_and_output = db.closure_inputs_and_output(*closure_id, substitution);
            push_clauses_for_apply(
                db,
                builder,
                well_known,
                trait_id,
                self_ty,
                closure_inputs_and_output,
            );
        }
        TyKind::Function(fn_val) if fn_val.sig.safety == Safety::Safe && !fn_val.sig.variadic => {
            let bound_ref = fn_val.clone().into_binders(interner);
            builder.push_binders(bound_ref, |builder, orig_sub| {
                // The last parameter represents the function return type
                let (arg_sub, fn_output_ty) = orig_sub
                    .0
                    .as_slice(interner)
                    .split_at(orig_sub.0.len(interner) - 1);
                let arg_sub = Substitution::from_iter(interner, arg_sub);
                let output_ty = fn_output_ty[0].assert_ty_ref(interner).clone();

                push_clauses(
                    db,
                    builder,
                    well_known,
                    trait_id,
                    self_ty.clone(),
                    arg_sub,
                    output_ty,
                );
            });
        }
        _ => {}
    }
}
