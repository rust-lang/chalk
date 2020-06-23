use super::{builder::ClauseBuilder, generalize};
use crate::{Interner, RustIrDatabase, TraitRef, WellKnownTrait};
use chalk_ir::{Floundered, Substitution, Ty};

mod clone;
mod copy;
mod fn_family;
mod sized;
mod unsize;

/// For well known traits we have special hard-coded impls, either as an
/// optimization or to enforce special rules for correctness.
pub fn add_builtin_program_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    well_known: WellKnownTrait,
    trait_ref: &TraitRef<I>,
) -> Result<(), Floundered> {
    // If `trait_ref` contains bound vars, we want to universally quantify them.
    // `Generalize` collects them for us.
    let generalized = generalize::Generalize::apply(db.interner(), trait_ref);

    builder.push_binders(&generalized, |builder, trait_ref| {
        let self_ty = trait_ref.self_type_parameter(db.interner());
        let ty = self_ty.data(db.interner());
        if let Some(force_impl) = db.force_impl_for(well_known, ty) {
            if force_impl {
                builder.push_fact(trait_ref.clone());
            }
            return Ok(());
        }

        match well_known {
            WellKnownTrait::Sized => sized::add_sized_program_clauses(db, builder, &trait_ref, ty),
            WellKnownTrait::Copy => copy::add_copy_program_clauses(db, builder, &trait_ref, ty),
            WellKnownTrait::Clone => clone::add_clone_program_clauses(db, builder, &trait_ref, ty),
            WellKnownTrait::FnOnce | WellKnownTrait::FnMut | WellKnownTrait::Fn => {
                fn_family::add_fn_trait_program_clauses(db, builder, well_known, self_ty)?
            }
            WellKnownTrait::Unsize => {
                unsize::add_unsize_program_clauses(db, builder, &trait_ref, ty)
            }
            // Drop impls are provided explicitly
            WellKnownTrait::Drop => (),
        }
        Ok(())
    })
}

/// Like `add_builtin_program_clauses`, but for `DomainGoal::Normalize` involving
/// a projection (e.g. `<fn(u8) as FnOnce<(u8,)>>::Output`)
pub fn add_builtin_assoc_program_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    well_known: WellKnownTrait,
    self_ty: Ty<I>,
) -> Result<(), Floundered> {
    match well_known {
        WellKnownTrait::FnOnce => {
            // If `self_ty` contains bound vars, we want to universally quantify them.
            // `Generalize` collects them for us.
            let generalized = generalize::Generalize::apply(db.interner(), &self_ty);

            builder.push_binders(&generalized, |builder, self_ty| {
                fn_family::add_fn_trait_program_clauses(db, builder, well_known, self_ty)?;
                Ok(())
            })
        }
        _ => Ok(()),
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
