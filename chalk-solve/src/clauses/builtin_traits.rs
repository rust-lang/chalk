use super::builder::ClauseBuilder;
use crate::Interner;
use crate::{TraitRef, WellKnownTrait};

pub fn add_builtin_program_clauses<I: Interner>(
    well_known: WellKnownTrait,
    _trait_ref: &TraitRef<I>,
    _builder: &mut ClauseBuilder<I>,
) {
    match well_known {
        WellKnownTrait::SizedTrait => { /* TODO */ }
    }
}
