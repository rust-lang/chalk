use super::builder::ClauseBuilder;
use crate::{Interner, RustIrDatabase, TraitRef, WellKnownTrait};
use chalk_ir::TyData;

mod copy;
mod sized;

/// For well known traits we have special hard-coded impls, either as an
/// optimization or to enforce special rules for correctness.
pub fn add_builtin_program_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    well_known: WellKnownTrait,
    trait_ref: &TraitRef<I>,
    ty: &TyData<I>,
) {
    if let Some(force_impl) = db.force_impl_for(well_known, ty) {
        if force_impl {
            builder.push_fact(trait_ref.clone());
        }
        return;
    }

    match well_known {
        WellKnownTrait::SizedTrait => sized::add_sized_program_clauses(db, builder, trait_ref, ty),
        WellKnownTrait::CopyTrait | WellKnownTrait::CloneTrait => {
            copy::add_copy_clone_program_clauses(db, builder, trait_ref, ty)
        }
        // Drop impls are provided explicitly
        WellKnownTrait::DropTrait => (),
    }
}
