use super::builder::ClauseBuilder;
use crate::{Interner, RustIrDatabase, TraitRef, WellKnownTrait};
use chalk_ir::{Substitution, Ty, TyData};

mod clone;
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
        WellKnownTrait::CopyTrait => copy::add_copy_program_clauses(db, builder, trait_ref, ty),
        WellKnownTrait::CloneTrait => clone::add_clone_program_clauses(db, builder, trait_ref, ty),
        // Drop impls are provided explicitly
        WellKnownTrait::DropTrait => (),
    }
}

/// Given a trait ref `T0: Trait` and a list of types `U0..Un`, pushes a clause of the form
/// `Implemented(T0: Trait) :- Implemented(U0: Trait) .. Implemented(Un: Trait)`
pub fn needs_impl_for_tys<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    trait_ref: &TraitRef<I>,
    tys: impl Iterator<Item = Ty<I>>,
) {
    // The trait must take one parameter (a type)
    debug_assert_eq!(
        db.trait_datum(trait_ref.trait_id)
            .binders
            .len(db.interner()),
        1,
    );
    builder.push_clause(
        trait_ref.clone(),
        tys.map(|ty| TraitRef {
            trait_id: trait_ref.trait_id,
            substitution: Substitution::from1(db.interner(), ty),
        }),
    );
}
