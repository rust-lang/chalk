use std::iter;

use crate::clauses::builtin_traits::needs_impl_for_tys;
use crate::clauses::ClauseBuilder;
use crate::rust_ir::AdtKind;
use crate::{Interner, RustIrDatabase, TraitRef};
use chalk_ir::{
    AdtId, ApplicationTy, CanonicalVarKinds, Substitution, TyData, TyKind, TypeName, VariableKind,
};

fn push_adt_sized_conditions<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    trait_ref: &TraitRef<I>,
    adt_id: AdtId<I>,
    substitution: &Substitution<I>,
) {
    let adt_datum = db.adt_datum(adt_id);

    // WF ensures that all enums are Sized, so we only have to consider structs.
    if adt_datum.kind != AdtKind::Struct {
        builder.push_fact(trait_ref.clone());
        return;
    }

    let interner = db.interner();

    // To check if a struct S<..> is Sized, we only have to look at its last field.
    // This is because the WF checks for ADTs require that all the other fields must be Sized.
    let last_field_ty = adt_datum
        .binders
        .map_ref(|b| &b.variants)
        .substitute(interner, substitution)
        .into_iter()
        .take(1) // We have a struct so we're guaranteed one variant
        .flat_map(|mut v| v.fields.pop());

    needs_impl_for_tys(db, builder, trait_ref, last_field_ty);
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
    binders: &CanonicalVarKinds<I>,
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

            TypeName::AssociatedType(_)
            | TypeName::Slice
            | TypeName::OpaqueType(_)
            | TypeName::Str
            | TypeName::Foreign(_)
            | TypeName::Error => {}
        },

        TyData::Function(_)
        | TyData::InferenceVar(_, TyKind::Float)
        | TyData::InferenceVar(_, TyKind::Integer) => builder.push_fact(trait_ref.clone()),

        TyData::BoundVar(bound_var) => {
            let var_kind = &binders.at(db.interner(), bound_var.index).kind;
            match var_kind {
                VariableKind::Ty(TyKind::Integer) | VariableKind::Ty(TyKind::Float) => {
                    builder.push_fact(trait_ref.clone())
                }
                VariableKind::Ty(_) | VariableKind::Const(_) | VariableKind::Lifetime => {}
            }
        }

        TyData::InferenceVar(_, TyKind::General)
        | TyData::Placeholder(_)
        | TyData::Dyn(_)
        | TyData::Alias(_) => {}
    }
}
