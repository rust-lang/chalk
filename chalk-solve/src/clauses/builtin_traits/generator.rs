use crate::clauses::ClauseBuilder;
use crate::rust_ir::WellKnownTrait;
use crate::{Interner, RustIrDatabase, TraitRef};
use chalk_ir::cast::Cast;
use chalk_ir::{AliasTy, Floundered, Normalize, ProjectionTy, Substitution, Ty, TyKind};

/// Add implicit impls of the generator trait, i.e., add a clause that all generators implement
/// `Generator` and clauses for `Generator`'s associated types.
pub fn add_generator_program_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    self_ty: Ty<I>,
) -> Result<(), Floundered> {
    let interner = db.interner();

    match self_ty.kind(interner) {
        TyKind::Generator(id, substitution) => {
            let generator_datum = db.generator_datum(*id);
            let generator_io_datum = generator_datum
                .input_output
                .clone()
                .substitute(interner, &substitution);

            let trait_id = db.well_known_trait_id(WellKnownTrait::Generator).unwrap();
            let trait_datum = db.trait_datum(trait_id);
            assert_eq!(
                trait_datum.associated_ty_ids.len(),
                2,
                "Generator trait should have exactly two associated types, found {:?}",
                trait_datum.associated_ty_ids
            );

            let substitution = Substitution::from_iter(
                interner,
                &[
                    self_ty.cast(interner),
                    generator_io_datum.resume_type.cast(interner),
                ],
            );

            // generator: Generator<resume_type>
            builder.push_fact(TraitRef {
                trait_id,
                substitution: substitution.clone(),
            });

            // `Generator::Yield`
            let yield_id = trait_datum.associated_ty_ids[0];
            let yield_alias = AliasTy::Projection(ProjectionTy {
                associated_ty_id: yield_id,
                substitution: substitution.clone(),
            });
            builder.push_fact(Normalize {
                alias: yield_alias,
                ty: generator_io_datum.yield_type,
            });

            // `Generator::Return`
            let return_id = trait_datum.associated_ty_ids[1];
            let return_alias = AliasTy::Projection(ProjectionTy {
                associated_ty_id: return_id,
                substitution,
            });
            builder.push_fact(Normalize {
                alias: return_alias,
                ty: generator_io_datum.return_type,
            });

            Ok(())
        }

        // Generator trait is non-enumerable
        TyKind::InferenceVar(..) | TyKind::BoundVar(_) | TyKind::Alias(..) => Err(Floundered),
        _ => Ok(()),
    }
}
