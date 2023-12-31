use crate::clauses::ClauseBuilder;
use crate::rust_ir::WellKnownTrait;
use crate::{Interner, RustIrDatabase, TraitRef};
use chalk_ir::{
    AliasTy, Floundered, Normalize, ProjectionTy, Substitution, Ty, TyKind, TyVariableKind,
};

use super::last_field_of_struct;

fn push_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    self_ty: Ty<I>,
    metadata: Ty<I>,
) {
    let interner = db.interner();
    let trait_id = db.well_known_trait_id(WellKnownTrait::Pointee).unwrap();
    let substitution = Substitution::from1(interner, self_ty);
    let trait_datum = db.trait_datum(trait_id);
    assert_eq!(
        trait_datum.associated_ty_ids.len(),
        1,
        "Pointee trait should have exactly one associated type, found {:?}",
        trait_datum.associated_ty_ids
    );
    let metadata_id = trait_datum.associated_ty_ids[0];
    let alias = AliasTy::Projection(ProjectionTy {
        associated_ty_id: metadata_id,
        substitution,
    });
    builder.push_fact(Normalize {
        alias,
        ty: metadata,
    });
}

/// Add implicit impl for the `Pointee` trait for all types
pub fn add_pointee_program_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    self_ty: Ty<I>,
) -> Result<(), Floundered> {
    let interner = db.interner();
    let trait_id = db.well_known_trait_id(WellKnownTrait::Pointee).unwrap();
    let substitution = Substitution::from1(interner, self_ty.clone());
    builder.push_fact(TraitRef {
        trait_id,
        substitution: substitution.clone(),
    });
    match self_ty.kind(interner) {
        TyKind::Str | TyKind::Slice(_) => push_clauses(
            db,
            builder,
            self_ty.clone(),
            TyKind::Scalar(chalk_ir::Scalar::Uint(chalk_ir::UintTy::Usize)).intern(interner),
        ),
        TyKind::Array(_, _)
        | TyKind::Never
        | TyKind::Closure(_, _)
        | TyKind::FnDef(_, _)
        | TyKind::Scalar(_)
        | TyKind::Raw(_, _)
        | TyKind::Function(_)
        | TyKind::InferenceVar(_, TyVariableKind::Float)
        | TyKind::InferenceVar(_, TyVariableKind::Integer)
        | TyKind::Coroutine(_, _)
        | TyKind::CoroutineWitness(_, _)
        | TyKind::Ref(_, _, _) => push_clauses(
            db,
            builder,
            self_ty,
            TyKind::Tuple(0, Substitution::empty(interner)).intern(interner),
        ),
        TyKind::Adt(id, subst) => {
            if let Some(last_field_ty) = last_field_of_struct(db, *id, subst) {
                push_for_last_field(last_field_ty, db, builder, self_ty);
            } else {
                push_clauses(
                    db,
                    builder,
                    self_ty,
                    TyKind::Tuple(0, Substitution::empty(interner)).intern(interner),
                );
            }
        }
        TyKind::Tuple(_, subst) => {
            let last_field_ty = subst
                .iter(interner)
                .rev()
                .next()
                .and_then(|x| x.ty(interner))
                .cloned();
            if let Some(last_field_ty) = last_field_ty {
                push_for_last_field(last_field_ty, db, builder, self_ty);
            } else {
                push_clauses(
                    db,
                    builder,
                    self_ty,
                    TyKind::Tuple(0, Substitution::empty(interner)).intern(interner),
                );
            }
        }
        TyKind::BoundVar(_)
        | TyKind::AssociatedType(_, _)
        | TyKind::OpaqueType(_, _)
        | TyKind::Foreign(_)
        | TyKind::Error
        | TyKind::Placeholder(_)
        | TyKind::Alias(_) => (),
        TyKind::Dyn(_) => {
            // FIXME: We should add a `Normalize(<dyn Trait as Pointee>::Metadata -> DynMetadata<dyn Trait>)` here, but
            // since chalk doesn't have the concept of lang item structs yet, we can't.
        }
        TyKind::InferenceVar(_, TyVariableKind::General) => return Err(Floundered),
    }
    Ok(())
}

fn push_for_last_field<I: Interner>(
    last_field_ty: Ty<I>,
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    self_ty: Ty<I>,
) {
    let interner = db.interner();
    let _ = add_pointee_program_clauses(db, builder, last_field_ty.clone());
    let trait_id = db.well_known_trait_id(WellKnownTrait::Pointee).unwrap();
    let trait_datum = db.trait_datum(trait_id);
    assert_eq!(
        trait_datum.associated_ty_ids.len(),
        1,
        "Pointee trait should have exactly one associated type, found {:?}",
        trait_datum.associated_ty_ids
    );
    let metadata_id = trait_datum.associated_ty_ids[0];
    let alias_last_field = AliasTy::Projection(ProjectionTy {
        associated_ty_id: metadata_id,
        substitution: Substitution::from1(interner, last_field_ty),
    });
    let alias_self = AliasTy::Projection(ProjectionTy {
        associated_ty_id: metadata_id,
        substitution: Substitution::from1(interner, self_ty),
    });
    builder.push_fact(Normalize {
        alias: alias_self,
        ty: TyKind::Alias(alias_last_field).intern(interner),
    });
}
