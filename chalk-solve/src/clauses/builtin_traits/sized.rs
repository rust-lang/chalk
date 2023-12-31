use std::iter;

use crate::clauses::builtin_traits::needs_impl_for_tys;
use crate::clauses::ClauseBuilder;
use crate::{Interner, RustIrDatabase, TraitRef};
use chalk_ir::{
    AdtId, CanonicalVarKinds, Floundered, Substitution, TyKind, TyVariableKind, VariableKind,
};

use super::last_field_of_struct;

fn push_adt_sized_conditions<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    trait_ref: TraitRef<I>,
    adt_id: AdtId<I>,
    substitution: &Substitution<I>,
) {
    // We only need to check last field of the struct here. Rest of the fields and cases are handled in WF.
    let last_field_ty = last_field_of_struct(db, adt_id, substitution).into_iter();
    needs_impl_for_tys(db, builder, trait_ref, last_field_ty);
}

fn push_tuple_sized_conditions<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    trait_ref: TraitRef<I>,
    arity: usize,
    substitution: &Substitution<I>,
) {
    // Empty tuples are always Sized
    if arity == 0 {
        builder.push_fact(trait_ref);
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
    trait_ref: TraitRef<I>,
    ty: TyKind<I>,
    binders: &CanonicalVarKinds<I>,
) -> Result<(), Floundered> {
    match ty {
        TyKind::Adt(adt_id, ref substitution) => {
            push_adt_sized_conditions(db, builder, trait_ref, adt_id, substitution)
        }
        TyKind::Tuple(arity, ref substitution) => {
            push_tuple_sized_conditions(db, builder, trait_ref, arity, substitution)
        }
        TyKind::Array(_, _)
        | TyKind::Never
        | TyKind::Closure(_, _)
        | TyKind::FnDef(_, _)
        | TyKind::Scalar(_)
        | TyKind::Raw(_, _)
        | TyKind::Coroutine(_, _)
        | TyKind::CoroutineWitness(_, _)
        | TyKind::Ref(_, _, _) => builder.push_fact(trait_ref),

        TyKind::AssociatedType(_, _)
        | TyKind::Slice(_)
        | TyKind::OpaqueType(_, _)
        | TyKind::Str
        | TyKind::Foreign(_)
        | TyKind::Error => {}

        TyKind::Function(_)
        | TyKind::InferenceVar(_, TyVariableKind::Float)
        | TyKind::InferenceVar(_, TyVariableKind::Integer) => builder.push_fact(trait_ref),

        TyKind::BoundVar(bound_var) => {
            let var_kind = &binders.at(db.interner(), bound_var.index).kind;
            match var_kind {
                VariableKind::Ty(TyVariableKind::Integer)
                | VariableKind::Ty(TyVariableKind::Float) => builder.push_fact(trait_ref),

                // Don't know enough
                VariableKind::Ty(TyVariableKind::General) => return Err(Floundered),

                VariableKind::Const(_) | VariableKind::Lifetime => {}
            }
        }

        // We don't know enough here
        TyKind::InferenceVar(_, TyVariableKind::General) => return Err(Floundered),

        // These would be handled elsewhere
        TyKind::Placeholder(_) | TyKind::Dyn(_) | TyKind::Alias(_) => {}
    }
    Ok(())
}
