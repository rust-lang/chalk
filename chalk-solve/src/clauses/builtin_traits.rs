use super::builder::ClauseBuilder;
use crate::Interner;
use crate::{TraitRef, WellKnownTrait};

/// For well known traits we have special hard-coded impls, either as an
/// optimization or to enforce special rules for correctness.
pub fn add_builtin_program_clauses<I: Interner>(
    well_known: WellKnownTrait,
    _trait_ref: &TraitRef<I>,
    _builder: &mut ClauseBuilder<'_, I>,
) {
    match well_known {
        WellKnownTrait::SizedTrait => { /* TODO */ }
    }
}
