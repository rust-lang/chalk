use std::iter;

use crate::clauses::builtin_traits::needs_impl_for_tys;
use crate::clauses::ClauseBuilder;
use crate::{Interner, RustIrDatabase, TraitRef};
use chalk_ir::{AdtId, ApplicationTy, Substitution, TyData, TypeName};

fn push_adt_sized_conditions<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    trait_ref: &TraitRef<I>,
    adt_id: AdtId<I>,
    substitution: &Substitution<I>,
) {
    let adt_datum = db.adt_datum(adt_id);

    // ADTs with no fields are always Sized
    if adt_datum.binders.skip_binders().fields.is_empty() {
        builder.push_fact(trait_ref.clone());
        return;
    }

    let interner = db.interner();

    // To check if an ADT type S<..> is Sized, we only have to look at its last field.
    // This is because the WF checks for ADTs require that all the other fields must be Sized.
    let last_field_ty = adt_datum
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
            TypeName::Adt(adt_id) => {
                push_adt_sized_conditions(db, builder, trait_ref, *adt_id, substitution)
            }
            TypeName::Tuple(arity) => {
                push_tuple_sized_conditions(db, builder, trait_ref, *arity, substitution)
            }
            TypeName::Array
            | TypeName::Never
            | TypeName::Closure(_)
            | TypeName::FnDef(_)
            | TypeName::Scalar(_)
            | TypeName::Raw(_)
            | TypeName::Ref(_) => builder.push_fact(trait_ref.clone()),
            _ => return,
        },
        TyData::Function(_) => builder.push_fact(trait_ref.clone()),
        // TODO(areredify)
        // when #368 lands, extend this to handle everything accordingly
        _ => return,
    }
}
