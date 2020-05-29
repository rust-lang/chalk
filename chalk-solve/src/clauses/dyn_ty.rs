use rustc_hash::FxHashSet;

use super::{builder::ClauseBuilder, generalize};
use crate::RustIrDatabase;
use chalk_ir::{
    cast::Cast, fold::shift::Shift, interner::Interner, Binders, BoundVar, DebruijnIndex, TraitId,
    TraitRef, Ty, TyData, WhereClause,
};

/// If the self type `S` of an `Implemented` goal is a `dyn trait` type, we wish
/// to generate program-clauses that indicates that it implements its own
/// traits. For example, a `dyn Write` type implements `Write` and so on.
///
/// To see how this works, consider as an example the type `dyn Fn(&u8)`. This
/// is really shorthand for `dyn for<'a> Fn<(&'a u8), Output = ()>`, and we
/// represent that type as something like this:
///
/// ```ignore
/// dyn(exists<T> {
///     forall<'a> { Implemented(T: Fn<'a>) },
///     forall<'a> { AliasEq(<T as Fn<'a>>::Output, ()) },
/// })
/// ```
///
/// so what we will do is to generate one program clause for each of the
/// conditions. Thus we get two program clauses:
///
/// ```ignore
/// forall<'a> { Implemented(dyn Fn(&u8): Fn<(&'a u8)>) }
/// ```
///
/// and
///
/// ```ignore
/// forall<'a> { AliasEq(<dyn Fn(&u8) as Fn<'a>>::Output, ()) },
/// ```
pub(super) fn build_dyn_self_ty_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    self_ty: Ty<I>,
) {
    let interner = db.interner();
    let dyn_ty = match self_ty.data(interner) {
        TyData::Dyn(dyn_ty) => dyn_ty,
        _ => return,
    };
    let generalized_dyn_ty = generalize::Generalize::apply(db.interner(), dyn_ty);

    // Here, `self_ty` is the `dyn Fn(&u8)`, and `dyn_ty` is the `exists<T> { ..
    // }` clauses shown above.

    // Turn free BoundVars in the type into new existentials. E.g.
    // we might get some `dyn Foo<?X>`, and we don't want to return
    // a clause with a free variable. We can instead return a
    // slightly more general clause by basically turning this into
    // `exists<A> dyn Foo<A>`.

    builder.push_binders(&generalized_dyn_ty, |builder, dyn_ty| {
        for exists_qwc in dyn_ty.bounds.map_ref(|r| r.iter(interner)) {
            // Replace the `T` from `exists<T> { .. }` with `self_ty`,
            // yielding clases like
            //
            // ```
            // forall<'a> { Implemented(dyn Fn(&u8): Fn<(&'a u8)>) }
            // ```
            let qwc = exists_qwc.substitute(interner, &[self_ty.clone().cast(interner)]);

            builder.push_binders(&qwc, |builder, wc| match &wc {
                // For the implemented traits, we need to elaborate super traits and add where clauses from the trait
                WhereClause::Implemented(trait_ref) => {
                    push_dyn_ty_impl_clauses(db, builder, trait_ref.clone())
                }
                // Associated item bindings are just taken as facts (?)
                WhereClause::AliasEq(_) => builder.push_fact(wc),
                WhereClause::LifetimeOutlives(..) => {}
            });
        }
    });
}

/// Generate `Implemented` clauses for a `dyn Trait` type. We need to generate
/// `Implemented` clauses for all super traits, and for each trait we require
/// its where clauses. (See #203.)
fn push_dyn_ty_impl_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    trait_ref: TraitRef<I>,
) {
    let interner = db.interner();
    // We have some `dyn Trait`, and some `trait SuperTrait: WC`
    // which is a super trait of `Trait` (including actually
    // just being the same trait); then we want to push
    // `Implemented(dyn Trait: SuperTrait) :- WC`.

    let super_trait_refs =
        super_traits(db, trait_ref.trait_id).substitute(interner, &trait_ref.substitution);

    for q_super_trait_ref in super_trait_refs {
        builder.push_binders(&q_super_trait_ref, |builder, super_trait_ref| {
            let trait_datum = db.trait_datum(super_trait_ref.trait_id);
            let wc = trait_datum
                .where_clauses()
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
