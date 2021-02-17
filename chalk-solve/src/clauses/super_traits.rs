use rustc_hash::FxHashSet;

use super::builder::ClauseBuilder;
use crate::RustIrDatabase;
use chalk_ir::{
    fold::shift::Shift, interner::Interner, Binders, BoundVar, DebruijnIndex, TraitId, TraitRef,
    WhereClause,
};

/// Generate `Implemented` clauses for `dyn Trait` and opaque types. We need to generate
/// `Implemented` clauses for all super traits, and for each trait we require
/// its where clauses. (See #203.)
pub(super) fn push_trait_super_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    trait_ref: TraitRef<I>,
) {
    let interner = db.interner();
    // Given`trait SuperTrait: WC`, which is a super trait
    // of `Trait` (including actually just being the same trait);
    // then we want to push
    // - for `dyn Trait`:
    //     `Implemented(dyn Trait: SuperTrait) :- WC`.
    // - for placeholder `!T` of `opaque type T: Trait = HiddenTy`:
    //     `Implemented(!T: SuperTrait) :- WC`

    let super_trait_refs =
        super_traits(db, trait_ref.trait_id).substitute(interner, &trait_ref.substitution);

    for q_super_trait_ref in super_trait_refs {
        builder.push_binders(q_super_trait_ref.clone(), |builder, super_trait_ref| {
            let trait_datum = db.trait_datum(super_trait_ref.trait_id);
            let wc = trait_datum
                .where_clauses()
                .cloned()
                .substitute(interner, &super_trait_ref.substitution);
            builder.push_clause(super_trait_ref, wc);
        });
    }
}

pub fn super_traits<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    trait_id: TraitId<I>,
) -> Binders<Vec<Binders<TraitRef<I>>>> {
    let interner = db.interner();
    let mut seen_traits = FxHashSet::default();
    let trait_datum = db.trait_datum(trait_id);
    let trait_ref = Binders::empty(
        db.interner(),
        TraitRef {
            trait_id,
            substitution: trait_datum
                .binders
                .identity_substitution(interner)
                .shifted_in(interner),
        },
    );
    let mut trait_refs = Vec::new();
    go(db, trait_ref, &mut seen_traits, &mut trait_refs);

    fn go<I: Interner>(
        db: &dyn RustIrDatabase<I>,
        trait_ref: Binders<TraitRef<I>>,
        seen_traits: &mut FxHashSet<TraitId<I>>,
        trait_refs: &mut Vec<Binders<TraitRef<I>>>,
    ) {
        let interner = db.interner();
        let trait_id = trait_ref.skip_binders().trait_id;
        // Avoid cycles
        if !seen_traits.insert(trait_id) {
            return;
        }
        trait_refs.push(trait_ref.clone());
        let trait_datum = db.trait_datum(trait_id);
        let super_trait_refs = trait_datum
            .binders
            .map_ref(|td| {
                td.where_clauses
                    .iter()
                    .filter_map(|qwc| {
                        qwc.as_ref().filter_map(|wc| match wc {
                            WhereClause::Implemented(tr) => {
                                let self_ty = tr.self_type_parameter(db.interner());

                                // We're looking for where clauses
                                // of the form `Self: Trait`. That's
                                // ^1.0 because we're one binder in.
                                if self_ty.bound_var(db.interner())
                                    != Some(BoundVar::new(DebruijnIndex::ONE, 0))
                                {
                                    return None;
                                }
                                Some(tr.clone())
                            }
                            WhereClause::AliasEq(_) => None,
                            WhereClause::LifetimeOutlives(..) => None,
                            WhereClause::TypeOutlives(..) => None,
                        })
                    })
                    .collect::<Vec<_>>()
            })
            // we skip binders on the trait_ref here and add them to the binders
            // on the trait ref in the loop below. We could probably avoid this if
            // we could turn the `Binders<Vec<>>` into a `Vec<Binders<>>` easily.
            .substitute(db.interner(), &trait_ref.skip_binders().substitution);
        for q_super_trait_ref in super_trait_refs {
            // So now we need to combine the binders of trait_ref with the
            // binders of super_trait_ref.
            let actual_binders = Binders::new(trait_ref.binders.clone(), q_super_trait_ref);
            let q_super_trait_ref = actual_binders.fuse_binders(interner);
            go(db, q_super_trait_ref, seen_traits, trait_refs);
        }
        seen_traits.remove(&trait_id);
    }

    Binders::new(trait_datum.binders.binders.clone(), trait_refs)
}
