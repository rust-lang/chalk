use crate::clauses::ClauseBuilder;
use crate::rust_ir::WellKnownTrait;
use crate::{Interner, RustIrDatabase, TraitRef};
use chalk_ir::{Floundered, Substitution, Ty, TyKind};

/// Add implicit impl for the `Tuple` trait for all tuples
pub fn add_tuple_program_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    self_ty: Ty<I>,
) -> Result<(), Floundered> {
    let interner = db.interner();

    match self_ty.kind(interner) {
        TyKind::Tuple(..) => {
            let trait_id = db.well_known_trait_id(WellKnownTrait::Tuple).unwrap();

            builder.push_fact(TraitRef {
                trait_id,
                substitution: Substitution::from1(interner, self_ty),
            });

            Ok(())
        }

        // Tuple trait is non-enumerable
        TyKind::InferenceVar(..) | TyKind::BoundVar(_) | TyKind::Alias(..) => Err(Floundered),
        _ => Ok(()),
    }
}
