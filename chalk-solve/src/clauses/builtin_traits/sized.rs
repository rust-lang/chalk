use std::iter;

use crate::clauses::ClauseBuilder;
use crate::{Interner, RustIrDatabase, TraitRef};
use chalk_ir::{ApplicationTy, Substitution, TyData, TypeName};

pub fn add_sized_program_clauses<I: Interner>(
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
        // TODO(areredify)
        // when #368 lands, extend this to handle everything accordingly
        _ => return,
    };

    let struct_datum = db.struct_datum(struct_id);

    // Structs with no fields are always Sized
    if struct_datum.binders.map_ref(|b| b.fields.is_empty()).value {
        builder.push_fact(trait_ref.clone());
        return;
    }

    // To check if a struct type S<..> is Sized, we only have to look at its last field.
    // This is because the WF checks for structs require that all the other fields must be Sized.
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
