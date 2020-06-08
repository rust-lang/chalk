use crate::clauses::ClauseBuilder;
use crate::infer::instantiate::IntoBindersAndValue;
use crate::rust_ir::WellKnownTrait;
use crate::{Interner, RustIrDatabase, TraitRef};
use chalk_ir::{
    AliasTy, ApplicationTy, Binders, Normalize, ProjectionTy, Substitution, TraitId, Ty, TyData,
    TypeName, VariableKinds,
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
    trait_id: TraitId<I>,
    self_ty: Ty<I>,
) {
    let interner = db.interner();
    match self_ty.data(interner) {
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
        }
        _ => {}
    }
}
