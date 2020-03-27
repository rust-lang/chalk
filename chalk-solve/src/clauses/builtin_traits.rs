use crate::{TraitRef, WellKnownTrait};
use crate::Interner;
use super::builder::ClauseBuilder;

pub fn add_builtin_program_clauses<I: Interner>(
    well_known: WellKnownTrait,
    _trait_ref: &TraitRef<I>,
    _builder: &mut ClauseBuilder<I>,
) {
    match well_known {
        WellKnownTrait::SizedTrait => {
            /* TODO */
        }
    }
}

