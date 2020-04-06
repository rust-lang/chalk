use std::iter;

use super::builder::ClauseBuilder;
use crate::{Interner, RustIrDatabase, TraitRef, WellKnownTrait};
use chalk_ir::{ApplicationTy, Substitution, TyData, TypeName};

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
        WellKnownTrait::SizedTrait => add_sized_program_clauses(db, builder, trait_ref, ty),
        WellKnownTrait::CopyTrait => { /* TODO */ }
        WellKnownTrait::CloneTrait => { /* TODO */ }
    }
}

fn add_sized_program_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    trait_ref: &TraitRef<I>,
    ty: &TyData<I>,
) {
    let interner = db.interner();

    let (struct_id, substitution) = match ty {
        TyData::Apply(ApplicationTy {
            name: TypeName::Struct(struct_id),
            substitution,
        }) => (*struct_id, substitution),
        _ => return,
    };

    let struct_datum = db.struct_datum(struct_id);

    if struct_datum.binders.map_ref(|b| b.fields.is_empty()).value {
        builder.push_fact(trait_ref.clone());
        return;
    }

    let last_field_ty = struct_datum
        .binders
        .map_ref(|b| b.fields.last().unwrap())
        .substitute(interner, substitution)
        .clone();

    let last_field_sized_goal = TraitRef {
        trait_id: trait_ref.trait_id,
        substitution: Substitution::from1(interner, last_field_ty),
    };
    builder.push_clause(trait_ref.clone(), iter::once(last_field_sized_goal));
}
