use crate::clauses::ClauseBuilder;
use crate::infer::instantiate::IntoBindersAndValue;
use crate::rust_ir::{ClosureKind, FnDefInputsAndOutputDatum, WellKnownTrait};
use crate::{Interner, RustIrDatabase, TraitRef};
use chalk_ir::cast::Cast;
use chalk_ir::{
    AliasTy, Binders, Floundered, Normalize, ProjectionTy, Safety, Substitution, TraitId, Ty,
    TyKind, VariableKinds,
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
}

fn push_clauses_for_apply<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    well_known: WellKnownTrait,
    trait_id: TraitId<I>,
    self_ty: Ty<I>,
    inputs_and_output: &Binders<FnDefInputsAndOutputDatum<I>>,
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

    match self_ty.kind(interner) {
        TyKind::FnDef(fn_def_id, substitution) => {
            let fn_def_datum = builder.db.fn_def_datum(*fn_def_id);
            if fn_def_datum.sig.safety == Safety::Safe && !fn_def_datum.sig.variadic {
                let bound = fn_def_datum
                    .binders
                    .substitute(builder.interner(), &substitution);
                push_clauses_for_apply(
                    db,
                    builder,
                    well_known,
                    trait_id,
                    self_ty,
                    &bound.inputs_and_output,
                );
            }
            Ok(())
        }
        TyKind::Closure(closure_id, substitution) => {
            let closure_kind = db.closure_kind(*closure_id, &substitution);
            let trait_matches = match (well_known, closure_kind) {
                (WellKnownTrait::Fn, ClosureKind::Fn) => true,
                (WellKnownTrait::FnMut, ClosureKind::FnMut)
                | (WellKnownTrait::FnMut, ClosureKind::Fn) => true,
                (WellKnownTrait::FnOnce, _) => true,
                _ => false,
            };
            if !trait_matches {
                return Ok(());
            }
            let closure_inputs_and_output =
                db.closure_inputs_and_output(*closure_id, &substitution);
            push_clauses_for_apply(
                db,
                builder,
                well_known,
                trait_id,
                self_ty,
                &closure_inputs_and_output,
            );
            Ok(())
        }
        TyKind::Function(fn_val) if fn_val.sig.safety == Safety::Safe && !fn_val.sig.variadic => {
            let (binders, orig_sub) = fn_val.into_binders_and_value(interner);
            let bound_ref = Binders::new(VariableKinds::from_iter(interner, binders), orig_sub);
            builder.push_binders(&bound_ref, |builder, orig_sub| {
                // The last parameter represents the function return type
                let (arg_sub, fn_output_ty) = orig_sub
                    .as_slice(interner)
                    .split_at(orig_sub.len(interner) - 1);
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
            Ok(())
        }
        // Function traits are non-enumerable
        TyKind::InferenceVar(..) | TyKind::Alias(..) => Err(Floundered),
        _ => Ok(()),
    }
}
