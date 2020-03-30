use chalk_ir::TyData;
use super::builder::ClauseBuilder;
use crate::{Interner, RustIrDatabase, TraitRef, WellKnownTrait};

/// For well known traits we have special hard-coded impls, either as an
/// optimization or to enforce special rules for correctness.
pub fn add_builtin_program_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    well_known: WellKnownTrait,
    trait_ref: &TraitRef<I>,
    ty: &TyData<I>,
) {
    if db.force_impl_for(well_known, ty) {
        builder.push_fact(trait_ref.clone());
    }

    match well_known {
        WellKnownTrait::SizedTrait => { /* TODO */ }
    }
}
