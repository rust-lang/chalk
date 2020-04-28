use std::iter;

use crate::clauses::builtin_traits::needs_impl_for_tys;
use crate::clauses::ClauseBuilder;
use crate::{Interner, RustIrDatabase, TraitRef};
use chalk_ir::{ApplicationTy, StructId, Substitution, TyData, TypeName};

fn push_struct_sized_conditions<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    trait_ref: &TraitRef<I>,
    struct_id: StructId<I>,
    substitution: &Substitution<I>,
) {
    let struct_datum = db.struct_datum(struct_id);

    // Structs with no fields are always Sized
    if struct_datum.binders.skip_binders().fields.is_empty() {
        builder.push_fact(trait_ref.clone());
        return;
    }

    let interner = db.interner();

    // To check if a struct type S<..> is Sized, we only have to look at its last field.
    // This is because the WF checks for structs require that all the other fields must be Sized.
    let last_field_ty = struct_datum
        .binders
        .map_ref(|b| b.fields.last().unwrap())
        .substitute(interner, substitution);

    needs_impl_for_tys(db, builder, trait_ref, iter::once(last_field_ty));
}

fn push_tuple_sized_conditions<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    trait_ref: &TraitRef<I>,
    arity: usize,
    substitution: &Substitution<I>,
) {
    // Empty tuples are always Sized
    if arity == 0 {
        builder.push_fact(trait_ref.clone());
        return;
    }

    let interner = db.interner();

    // To check if a tuple is Sized, we only have to look at its last element.
    // This is because the WF checks for tuples require that all the other elements must be Sized.
    let last_elem_ty = substitution
        .iter(interner)
        .last()
        .unwrap()
        .ty(interner)
        .unwrap()
        .clone();

    needs_impl_for_tys(db, builder, trait_ref, iter::once(last_elem_ty));
}

pub fn add_sized_program_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    trait_ref: &TraitRef<I>,
    ty: &TyData<I>,
) {
    match ty {
        TyData::Apply(ApplicationTy { name, substitution }) => match name {
            TypeName::Struct(struct_id) => {
                push_struct_sized_conditions(db, builder, trait_ref, *struct_id, substitution)
            }
            TypeName::Scalar(_) => builder.push_fact(trait_ref.clone()),
            TypeName::Tuple(arity) => {
                push_tuple_sized_conditions(db, builder, trait_ref, *arity, substitution)
            }
            _ => return,
        },
        TyData::Function(_) => builder.push_fact(trait_ref.clone()),
        // TODO(areredify)
        // when #368 lands, extend this to handle everything accordingly
        _ => return,
    }
}
