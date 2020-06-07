use crate::clauses::ClauseBuilder;
use crate::infer::instantiate::IntoBindersAndValue;
use crate::rust_ir::WellKnownTrait;
use crate::{Interner, RustIrDatabase, TraitRef};
use chalk_ir::{
    AliasTy, ApplicationTy, Binders, Normalize, ProjectionTy, Substitution, TraitId, Ty, TyData,
    TypeName, VariableKinds,
};

/// Handles clauses for FnOnce/FnMut/Fn.
/// If `assoc_output` is `true`, we push a clause of the form
/// `Normalize(<fn(A) -> B as FnOnce<(A,)>>::Output -> B) :- Implemented(fn(A) -> B as FnOnce<(A,)>`
///
/// If `assoc_output` is `false`, we push a clause of the form
/// `Implemented(fn(A) -> B as FnOnce<(A,)>)`
pub fn add_fn_trait_program_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    trait_id: TraitId<I>,
    ty: &TyData<I>,
    assoc_output: bool,
) {
    match ty {
        TyData::Function(fn_val) => {
            let interner = db.interner();
            let (binders, orig_sub) = fn_val.into_binders_and_value(interner);
            // Take all of the arguments except for the last one, which
            // represents the return type
            let arg_sub = Substitution::from(
                interner,
                orig_sub.iter(interner).take(orig_sub.len(interner) - 1),
            );
            let fn_output_ty = orig_sub
                .at(interner, orig_sub.len(interner) - 1)
                .assert_ty_ref(interner);

            // We are constructing a reference to `FnOnce<Args>`, where
            // `Args` is a tuple of the function's argument types
            let tupled = Ty::new(
                interner,
                TyData::Apply(ApplicationTy {
                    name: TypeName::Tuple(arg_sub.len(interner)),
                    substitution: arg_sub.clone(),
                }),
            );

            let self_ty = Ty::new(interner, ty);

            let tupled_sub = Substitution::from(interner, vec![self_ty, tupled]);
            // Given a function type `fn(A1, A2, ..., AN)`, construct a `TraitRef`
            // of the form `fn(A1, A2, ..., AN): FnOnce<(A1, A2, ..., AN)>`
            let new_trait_ref = TraitRef {
                trait_id,
                substitution: tupled_sub.clone(),
            };

            // Functions types come with a binder, which we push so
            // that the `TraitRef` properly references any bound lifetimes
            // (e.g. `for<'a> fn(&'a u8): FnOnce<(&'b u8)>`)
            let bound_ref = Binders::new(VariableKinds::from(interner, binders), new_trait_ref);
            builder.push_binders(&bound_ref, |this, inner_trait| {
                if assoc_output {
                    //The `Output` type is defined on the `FnOnceTrait`
                    let fn_once = db.trait_datum(trait_id);
                    assert_eq!(fn_once.well_known, Some(WellKnownTrait::FnOnceTrait));
                    let assoc_types = &fn_once.associated_ty_ids;
                    if assoc_types.len() != 1 {
                        panic!(
                            "FnOnce trait should have exactly one associated type, found {:?}",
                            assoc_types
                        );
                    }

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

                    this.push_clause(normalize, std::iter::once(inner_trait));
                } else {
                    this.push_fact(inner_trait);
                }
            })
        }
        _ => {}
    }
}
