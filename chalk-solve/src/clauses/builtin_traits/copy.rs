use crate::clauses::builtin_traits::needs_impl_for_tys;
use crate::clauses::ClauseBuilder;
use crate::{Interner, RustIrDatabase, TraitRef};
use chalk_ir::{CanonicalVarKinds, Substitution, TyKind, TyVariableKind, VariableKind};
use std::iter;
use tracing::instrument;

fn push_tuple_copy_conditions<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    trait_ref: &TraitRef<I>,
    arity: usize,
    substitution: &Substitution<I>,
) {
    // Empty tuples are always Copy
    if arity == 0 {
        builder.push_fact(trait_ref.clone());
        return;
    }

    let interner = db.interner();

    needs_impl_for_tys(
        db,
        builder,
        trait_ref,
        substitution
            .iter(interner)
            .map(|param| param.assert_ty_ref(interner).clone()),
    );
}

#[instrument(skip(db, builder))]
pub fn add_copy_program_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    trait_ref: &TraitRef<I>,
    ty: &TyKind<I>,
    binders: &CanonicalVarKinds<I>,
) {
    match ty {
        TyKind::Tuple(arity, substitution) => {
            push_tuple_copy_conditions(db, builder, trait_ref, *arity, substitution)
        }
        TyKind::Array(ty, _) => {
            needs_impl_for_tys(db, builder, trait_ref, iter::once(ty.clone()));
        }
        TyKind::FnDef(_, _) => {
            builder.push_fact(trait_ref.clone());
        }
        TyKind::Closure(closure_id, substitution) => {
            let closure_fn_substitution = db.closure_fn_substitution(*closure_id, substitution);
            let upvars = db.closure_upvars(*closure_id, substitution);
            let upvars = upvars.substitute(db.interner(), &closure_fn_substitution);
            needs_impl_for_tys(db, builder, trait_ref, Some(upvars).into_iter());
        }

        // these impls are in libcore
        TyKind::Ref(_, _, _)
        | TyKind::Raw(_, _)
        | TyKind::Scalar(_)
        | TyKind::Never
        | TyKind::Str => {}

        TyKind::Adt(_, _)
        | TyKind::AssociatedType(_, _)
        | TyKind::Slice(_)
        | TyKind::OpaqueType(_, _)
        | TyKind::Foreign(_)
        | TyKind::Generator(_, _)
        | TyKind::GeneratorWitness(_, _)
        | TyKind::Error => {}

        TyKind::Function(_) => builder.push_fact(trait_ref.clone()),

        TyKind::InferenceVar(_, kind) => match kind {
            TyVariableKind::Integer | TyVariableKind::Float => builder.push_fact(trait_ref.clone()),
            TyVariableKind::General => {}
        },

        TyKind::BoundVar(bound_var) => {
            let var_kind = &binders.at(db.interner(), bound_var.index).kind;
            match var_kind {
                VariableKind::Ty(TyVariableKind::Integer)
                | VariableKind::Ty(TyVariableKind::Float) => builder.push_fact(trait_ref.clone()),
                VariableKind::Ty(_) | VariableKind::Const(_) | VariableKind::Lifetime => {}
            }
        }

        TyKind::Alias(_) | TyKind::Dyn(_) | TyKind::Placeholder(_) => {}
    };
}
