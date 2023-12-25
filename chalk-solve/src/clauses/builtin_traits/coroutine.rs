use crate::clauses::ClauseBuilder;
use crate::rust_ir::WellKnownTrait;
use crate::{Interner, RustIrDatabase, TraitRef};
use chalk_ir::cast::Cast;
use chalk_ir::{AliasTy, Floundered, Normalize, ProjectionTy, Substitution, Ty, TyKind};

/// Add implicit impls of the coroutine trait, i.e., add a clause that all coroutines implement
/// `Coroutine` and clauses for `Coroutine`'s associated types.
pub fn add_coroutine_program_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    self_ty: Ty<I>,
) -> Result<(), Floundered> {
    let interner = db.interner();

    match self_ty.kind(interner) {
        TyKind::Coroutine(id, substitution) => {
            let coroutine_datum = db.coroutine_datum(*id);
            let coroutine_io_datum = coroutine_datum
                .input_output
                .clone()
                .substitute(interner, &substitution);

            let trait_id = db.well_known_trait_id(WellKnownTrait::Coroutine).unwrap();
            let trait_datum = db.trait_datum(trait_id);
            assert_eq!(
                trait_datum.associated_ty_ids.len(),
                2,
                "Coroutine trait should have exactly two associated types, found {:?}",
                trait_datum.associated_ty_ids
            );

            let substitution = Substitution::from_iter(
                interner,
                &[
                    self_ty.cast(interner),
                    coroutine_io_datum.resume_type.cast(interner),
                ],
            );

            // coroutine: Coroutine<resume_type>
            builder.push_fact(TraitRef {
                trait_id,
                substitution: substitution.clone(),
            });

            // `Coroutine::Yield`
            let yield_id = trait_datum.associated_ty_ids[0];
            let yield_alias = AliasTy::Projection(ProjectionTy {
                associated_ty_id: yield_id,
                substitution: substitution.clone(),
            });
            builder.push_fact(Normalize {
                alias: yield_alias,
                ty: coroutine_io_datum.yield_type,
            });

            // `Coroutine::Return`
            let return_id = trait_datum.associated_ty_ids[1];
            let return_alias = AliasTy::Projection(ProjectionTy {
                associated_ty_id: return_id,
                substitution,
            });
            builder.push_fact(Normalize {
                alias: return_alias,
                ty: coroutine_io_datum.return_type,
            });

            Ok(())
        }

        // Coroutine trait is non-enumerable
        TyKind::InferenceVar(..) | TyKind::BoundVar(_) | TyKind::Alias(..) => Err(Floundered),
        _ => Ok(()),
    }
}
