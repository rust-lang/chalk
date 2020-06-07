use crate::clauses::ClauseBuilder;
use crate::infer::instantiate::IntoBindersAndValue;
use crate::{Interner, RustIrDatabase, TraitRef};
use chalk_ir::{ApplicationTy, Binders, Substitution, Ty, TyData, TypeName, VariableKinds};

// Handles clauses for FnOnce/FnMut/Fn
pub fn add_fn_trait_program_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    trait_ref: &TraitRef<I>,
    ty: &TyData<I>,
) {
    match ty {
        TyData::Function(fn_val) => {
            let interner = db.interner();
            let (binders, sub) = fn_val.into_binders_and_value(interner);

            // We are constructing a reference to `FnOnce<Args>`, where
            // `Args` is a tuple of the function's argument types
            let tupled = Ty::new(
                interner,
                TyData::Apply(ApplicationTy {
                    name: TypeName::Tuple(sub.len(interner)),
                    substitution: sub.clone(),
                }),
            );

            let self_ty = Ty::new(interner, ty);

            let tupled_sub = Substitution::from(interner, vec![self_ty, tupled]);
            // Given a function type `fn(A1, A2, ..., AN)`, construct a `TraitRef`
            // of the form `fn(A1, A2, ..., AN): FnOnce<(A1, A2, ..., AN)>`
            let new_trait_ref = TraitRef {
                trait_id: trait_ref.trait_id,
                substitution: tupled_sub,
            };

            // Functions types come with a binder, which we push so
            // that the `TraitRef` properly references any bound lifetimes
            // (e.g. `for<'a> fn(&'a u8): FnOnce<(&'b u8)>`)
            let bound_ref = Binders::new(VariableKinds::from(interner, binders), new_trait_ref);
            builder.push_binders(&bound_ref, |this, inner_trait| {
                this.push_fact(inner_trait);
            })
        }
        _ => {}
    }
}
