use crate::clauses::ClauseBuilder;
use crate::infer::instantiate::IntoBindersAndValue;
use crate::rust_ir::{ClosureKind, WellKnownTrait};
use crate::{Interner, RustIrDatabase, TraitRef};
use chalk_ir::cast::Cast;
use chalk_ir::{
    AliasTy, ApplicationTy, Binders, Floundered, Normalize, ProjectionTy, Substitution, TraitId,
    Ty, TyData, TypeName, VariableKinds,
};

/// Handles clauses for FnOnce/FnMut/Fn.
/// If `self_ty` is a function, we push a clause of the form
/// `fn(A1, A2, ..., AN) -> O: FnTrait<(A1, A2, ..., AN)>`, where `FnTrait`
/// is the trait corresponding to `trait_id` (FnOnce/FnMut/Fn)
///
/// If `trait_id` is `FnOnce`, we also push a clause for the output type of the form:
/// `Normalize(<fn(A) -> B as FnOnce<(A,)>>::Output -> B)`
/// We do not add the usual `Implemented(fn(A) -> b as FnOnce<(A,)>` clause
/// as a condition, since we already called `push_fact` with it
pub fn add_fn_trait_program_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    well_known: WellKnownTrait,
    self_ty: Ty<I>,
) -> Result<(), Floundered> {
    let interner = db.interner();
    let trait_id = db.well_known_trait_id(well_known).unwrap();
    match self_ty.data(interner) {
        TyData::Apply(apply) => match apply.name {
            TypeName::FnDef(fn_def_id) => {
                let fn_def_datum = builder.db.fn_def_datum(fn_def_id);
                let bound = fn_def_datum
                    .binders
                    .substitute(builder.interner(), &apply.substitution);
                builder.push_binders(&bound.inputs_and_output, |builder, inputs_and_output| {
                    let interner = builder.interner();
                    let ty = ApplicationTy {
                        name: apply.name,
                        substitution: builder.substitution_in_scope(),
                    }
                    .intern(interner);

                    let substitution = {
                        let self_ty = ty.cast(interner);
                        let arguments = ApplicationTy {
                            name: TypeName::Tuple(inputs_and_output.argument_types.len()),
                            substitution: Substitution::from(
                                interner,
                                inputs_and_output
                                    .argument_types
                                    .iter()
                                    .cloned()
                                    .map(|ty| ty.cast(interner)),
                            ),
                        }
                        .intern(interner)
                        .cast(interner);
                        Substitution::from(interner, &[self_ty, arguments])
                    };
                    builder.push_fact(TraitRef {
                        trait_id,
                        substitution: substitution.clone(),
                    });

                    if let WellKnownTrait::FnOnce = well_known {
                        let trait_datum = db.trait_datum(trait_id);
                        let output_id = trait_datum.associated_ty_ids[0];
                        let alias = AliasTy::Projection(ProjectionTy {
                            associated_ty_id: output_id,
                            substitution,
                        });
                        let return_type = inputs_and_output.return_type;
                        builder.push_fact(Normalize {
                            alias,
                            ty: return_type,
                        });
                    }
                });
                Ok(())
            }
            TypeName::Closure(closure_id) => {
                let closure_datum = db.closure_datum(closure_id, apply.substitution.clone());
                let trait_matches = match well_known {
                    WellKnownTrait::Fn => matches!(closure_datum.kind, ClosureKind::Fn),
                    WellKnownTrait::FnMut => {
                        matches!(closure_datum.kind, ClosureKind::FnMut | ClosureKind::Fn)
                    }
                    WellKnownTrait::FnOnce => matches!(
                        closure_datum.kind,
                        ClosureKind::FnOnce | ClosureKind::FnMut | ClosureKind::Fn
                    ),
                    _ => false,
                };
                if trait_matches {
                    builder.push_binders(
                        &closure_datum.inputs_and_output,
                        |builder, inputs_and_output| {
                            let substitution = Substitution::from(
                                interner,
                                Some(self_ty.cast(interner)).into_iter().chain(
                                    inputs_and_output
                                        .argument_types
                                        .iter()
                                        .cloned()
                                        .map(|ty| ty.cast(interner)),
                                ),
                            );
                            builder.push_fact(TraitRef {
                                trait_id,
                                substitution: substitution.clone(),
                            });

                            if let WellKnownTrait::FnOnce = well_known {
                                let trait_datum = db.trait_datum(trait_id);
                                let output_id = trait_datum.associated_ty_ids[0];
                                let alias = AliasTy::Projection(ProjectionTy {
                                    associated_ty_id: output_id,
                                    substitution,
                                });
                                let return_type = inputs_and_output.return_type.clone();
                                builder.push_fact(Normalize {
                                    alias,
                                    ty: return_type,
                                });
                            }
                        },
                    );
                }
                Ok(())
            }
            _ => Ok(()),
        },
        TyData::Function(fn_val) => {
            let (binders, orig_sub) = fn_val.into_binders_and_value(interner);
            let bound_ref = Binders::new(VariableKinds::from(interner, binders), orig_sub);
            builder.push_binders(&bound_ref, |builder, orig_sub| {
                // The last parameter represents the function return type
                let (arg_sub, fn_output_ty) = orig_sub
                    .parameters(interner)
                    .split_at(orig_sub.len(interner) - 1);
                let arg_sub = Substitution::from(interner, arg_sub);
                let fn_output_ty = fn_output_ty[0].assert_ty_ref(interner);

                // We are constructing a reference to `FnOnce<Args>`, where
                // `Args` is a tuple of the function's argument types
                let tupled = Ty::new(
                    interner,
                    TyData::Apply(ApplicationTy {
                        name: TypeName::Tuple(arg_sub.len(interner)),
                        substitution: arg_sub.clone(),
                    }),
                );

                let tupled_sub = Substitution::from(interner, vec![self_ty.clone(), tupled]);
                // Given a function type `fn(A1, A2, ..., AN)`, construct a `TraitRef`
                // of the form `fn(A1, A2, ..., AN): FnOnce<(A1, A2, ..., AN)>`
                let new_trait_ref = TraitRef {
                    trait_id,
                    substitution: tupled_sub.clone(),
                };

                builder.push_fact(new_trait_ref.clone());

                if let Some(WellKnownTrait::FnOnce) = db.trait_datum(trait_id).well_known {
                    //The `Output` type is defined on the `FnOnce`
                    let fn_once = db.trait_datum(trait_id);
                    assert_eq!(fn_once.well_known, Some(WellKnownTrait::FnOnce));
                    let assoc_types = &fn_once.associated_ty_ids;
                    assert_eq!(
                        assoc_types.len(),
                        1,
                        "FnOnce trait should have exactly one associated type, found {:?}",
                        assoc_types
                    );

                    // Construct `Normalize(<fn(A) -> B as FnOnce<(A,)>>::Output -> B)`
                    let assoc_output_ty = assoc_types[0];
                    let proj_ty = ProjectionTy {
                        associated_ty_id: assoc_output_ty,
                        substitution: tupled_sub,
                    };
                    let normalize = Normalize {
                        alias: AliasTy::Projection(proj_ty),
                        ty: fn_output_ty.clone(),
                    };

                    builder.push_fact(normalize);
                }
            });
            Ok(())
        }
        // Function traits are non-enumerable
        TyData::InferenceVar(..) | TyData::Alias(..) => Err(Floundered),
        _ => Ok(()),
    }
}
