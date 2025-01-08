use itertools::{Either, Itertools};
use rustc_hash::FxHashSet;

use super::builder::ClauseBuilder;
use crate::{split::Split, RustIrDatabase};
use chalk_ir::{
    fold::shift::Shift, interner::Interner, AliasEq, AliasTy, Binders, BoundVar, DebruijnIndex,
    Normalize, ProjectionTy, TraitId, TraitRef, Ty, WhereClause,
};

/// Generate `Implemented` and `Normalize` clauses for `dyn Trait` and opaque types.
/// We need to generate those clauses for all super traits, and for each trait we
/// require its where clauses. (See #203)
pub(super) fn push_trait_super_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    trait_ref: TraitRef<I>,
) {
    let interner = db.interner();
    // Given `trait SuperTrait: WC`, which is a super trait
    // of `Trait` (including actually just being the same trait);
    // then we want to push
    // - for `dyn Trait`:
    //     `Implemented(dyn Trait: SuperTrait) :- WC`.
    // - for placeholder `!T` of `opaque type T: Trait = HiddenTy`:
    //     `Implemented(!T: SuperTrait) :- WC`
    //
    // When `SuperTrait` has `AliasEq` bounds like `trait SuperTrait: AnotherTrait<Assoc = Ty>`,
    // we also push
    // - for `dyn Trait`:
    //     `Normalize(<dyn Trait as AnotherTrait>::Assoc -> Ty) :- AssocWC, WC`
    // - for placeholder `!T` of `opaque type T: Trait = HiddenTy`:
    //     `Normalize(<!T as AnotherTrait>::Assoc -> Ty) :- AssocWC, WC`
    // where `WC` and `AssocWC` are the where clauses for `AnotherTrait` and `AnotherTrait::Assoc`
    // respectively.
    let (super_trait_refs, super_trait_proj) =
        super_traits(db, trait_ref.trait_id).substitute(interner, &trait_ref.substitution);

    for q_super_trait_ref in super_trait_refs {
        builder.push_binders(q_super_trait_ref, |builder, super_trait_ref| {
            let trait_datum = db.trait_datum(super_trait_ref.trait_id);
            let wc = trait_datum
                .where_clauses()
                .cloned()
                .substitute(interner, &super_trait_ref.substitution);
            builder.push_clause(super_trait_ref, wc);
        });
    }

    for q_super_trait_proj in super_trait_proj {
        builder.push_binders(q_super_trait_proj, |builder, (proj, ty)| {
            let assoc_ty_datum = db.associated_ty_data(proj.associated_ty_id);
            let trait_datum = db.trait_datum(assoc_ty_datum.trait_id);
            let assoc_wc = assoc_ty_datum
                .binders
                .map_ref(|b| &b.where_clauses)
                .into_iter()
                .map(|wc| wc.cloned().substitute(interner, &proj.substitution));

            let impl_params = db.trait_parameters_from_projection(&proj);
            let impl_wc = trait_datum
                .where_clauses()
                .into_iter()
                .map(|wc| wc.cloned().substitute(interner, impl_params));
            builder.push_clause(
                Normalize {
                    alias: AliasTy::Projection(proj.clone()),
                    ty,
                },
                impl_wc.chain(assoc_wc),
            );
        });
    }
}

/// Returns super-`TraitRef`s and super-`Projection`s that are quantified over the parameters of
/// `trait_id` and relevant higher-ranked lifetimes. The outer `Binders` is for the former and the
/// inner `Binders` is for the latter.
///
/// For example, given the following trait definitions and `C` as `trait_id`,
///
/// ```
/// trait A<'a, T> {}
/// trait B<'b, U> where Self: for<'x> A<'x, U> {}
/// trait C<'c, V> where Self: B<'c, V> {}
/// ```
///
/// returns the following quantified `TraitRef`s.
///
/// ```notrust
/// for<Self, 'c, V> {
///     for<'x> { Self: A<'x, V> }
///     for<> { Self: B<'c, V> }
///     for<> { Self: C<'c, V> }
/// }
/// ```
pub(crate) fn super_traits<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    trait_id: TraitId<I>,
) -> Binders<(
    Vec<Binders<TraitRef<I>>>,
    Vec<Binders<(ProjectionTy<I>, Ty<I>)>>,
)> {
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
    let mut aliases = Vec::new();
    go(
        db,
        trait_ref,
        &mut seen_traits,
        &mut trait_refs,
        &mut aliases,
    );

    fn go<I: Interner>(
        db: &dyn RustIrDatabase<I>,
        trait_ref: Binders<TraitRef<I>>,
        seen_traits: &mut FxHashSet<TraitId<I>>,
        trait_refs: &mut Vec<Binders<TraitRef<I>>>,
        aliases: &mut Vec<Binders<(ProjectionTy<I>, Ty<I>)>>,
    ) {
        let interner = db.interner();
        let trait_id = trait_ref.skip_binders().trait_id;
        // Avoid cycles
        if !seen_traits.insert(trait_id) {
            return;
        }
        trait_refs.push(trait_ref.clone());
        let trait_datum = db.trait_datum(trait_id);
        let (super_trait_refs, super_trait_projs): (Vec<_>, Vec<_>) = trait_datum
            .binders
            .map_ref(|td| {
                td.where_clauses
                    .iter()
                    .filter(|qwc| {
                        let trait_ref = match qwc.skip_binders() {
                            WhereClause::Implemented(tr) => tr.clone(),
                            WhereClause::AliasEq(AliasEq {
                                alias: AliasTy::Projection(p),
                                ..
                            }) => db.trait_ref_from_projection(p),
                            _ => return false,
                        };
                        // We're looking for where clauses of the form
                        // `Self: Trait` or `<Self as Trait>::Assoc`. `Self` is
                        // ^1.0 because we're one binder in.
                        trait_ref.self_type_parameter(interner).bound_var(interner)
                            == Some(BoundVar::new(DebruijnIndex::ONE, 0))
                    })
                    .cloned()
                    .partition_map(|qwc| {
                        let (value, binders) = qwc.into_value_and_skipped_binders();

                        match value {
                            WhereClause::Implemented(tr) => Either::Left(Binders::new(binders, tr)),
                            WhereClause::AliasEq(AliasEq {
                                alias: AliasTy::Projection(p),
                                ty,
                            }) => Either::Right(Binders::new(binders, (p, ty))),
                            _ => unreachable!(),
                        }
                    })
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
            go(db, q_super_trait_ref, seen_traits, trait_refs, aliases);
        }
        for q_super_trait_proj in super_trait_projs {
            let actual_binders = Binders::new(trait_ref.binders.clone(), q_super_trait_proj);
            let q_super_trait_proj = actual_binders.fuse_binders(interner);
            aliases.push(q_super_trait_proj);
        }
        seen_traits.remove(&trait_id);
    }

    Binders::new(trait_datum.binders.binders.clone(), (trait_refs, aliases))
}
