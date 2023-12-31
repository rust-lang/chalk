use crate::clauses::ClauseBuilder;
use crate::{Interner, RustIrDatabase, TraitRef, WellKnownTrait};
use chalk_ir::{
    AliasTy, Floundered, Normalize, ProjectionTy, Substitution, Ty, TyKind, TyVariableKind,
};

pub fn add_discriminant_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    self_ty: Ty<I>,
) -> Result<(), Floundered> {
    let interner = db.interner();

    let can_determine_discriminant = match self_ty.data(interner).kind {
        TyKind::Adt(..)
        | TyKind::Array(..)
        | TyKind::Tuple(..)
        | TyKind::Slice(..)
        | TyKind::Raw(..)
        | TyKind::Ref(..)
        | TyKind::Scalar(_)
        | TyKind::Str
        | TyKind::Never
        | TyKind::FnDef(..)
        | TyKind::Coroutine(..)
        | TyKind::Closure(..)
        | TyKind::CoroutineWitness(..)
        | TyKind::Foreign(_)
        | TyKind::Dyn(_)
        | TyKind::Function(..)
        | TyKind::InferenceVar(_, TyVariableKind::Integer)
        | TyKind::InferenceVar(_, TyVariableKind::Float) => true,
        TyKind::OpaqueType(..)
        | TyKind::Alias(_)
        | TyKind::BoundVar(_)
        | TyKind::Placeholder(_)
        | TyKind::AssociatedType(..)
        | TyKind::Error
        | TyKind::InferenceVar(..) => false,
    };

    let trait_id = db
        .well_known_trait_id(WellKnownTrait::DiscriminantKind)
        .unwrap();
    let trait_datum = db.trait_datum(trait_id);

    let associated_ty_id = trait_datum.associated_ty_ids[0];
    let substitution = Substitution::from1(interner, self_ty.clone());

    let trait_ref = TraitRef {
        trait_id,
        substitution: substitution.clone(),
    };

    builder.push_fact(trait_ref);

    if !can_determine_discriminant {
        return Ok(());
    }

    let disc_ty = db.discriminant_type(self_ty);

    let normalize = Normalize {
        alias: AliasTy::Projection(ProjectionTy {
            associated_ty_id,
            substitution,
        }),
        ty: disc_ty,
    };

    builder.push_fact(normalize);

    Ok(())
}
