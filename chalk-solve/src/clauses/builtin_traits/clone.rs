use crate::clauses::ClauseBuilder;
use crate::{Interner, RustIrDatabase, TraitRef};
use chalk_ir::{CanonicalVarKinds, Floundered, TyKind};

use super::copy::add_copy_program_clauses;

pub fn add_clone_program_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    trait_ref: TraitRef<I>,
    ty: TyKind<I>,
    binders: &CanonicalVarKinds<I>,
) -> Result<(), Floundered> {
    // Implement Clone for types that automatically implement Copy
    add_copy_program_clauses(db, builder, trait_ref, ty, binders)
}
