use super::{builder::ClauseBuilder, generalize};
use crate::RustIrDatabase;
use chalk_ir::{cast::Cast, interner::Interner, Ty, TyKind, WhereClause};

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
    let dyn_ty = match self_ty.kind(interner) {
        TyKind::Dyn(dyn_ty) => dyn_ty.clone(),
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

    builder.push_binders(generalized_dyn_ty, |builder, dyn_ty| {
        for exists_qwc in dyn_ty.bounds.map_ref(|r| r.iter(interner)) {
            // Replace the `T` from `exists<T> { .. }` with `self_ty`,
            // yielding clases like
            //
            // ```
            // forall<'a> { Implemented(dyn Fn(&u8): Fn<(&'a u8)>) }
            // ```
            let qwc = exists_qwc
                .cloned()
                .substitute(interner, &[self_ty.clone().cast(interner)]);

            builder.push_binders(qwc, |builder, bound| match &bound {
                // For the implemented traits, we need to elaborate super traits and add where clauses from the trait
                WhereClause::Implemented(trait_ref) => {
                    super::super_traits::push_trait_super_clauses(
                        builder.db,
                        builder,
                        trait_ref.clone(),
                    )
                }
                // FIXME: Associated item bindings are just taken as facts (?)
                WhereClause::AliasEq(_) => builder.push_fact(bound),
                WhereClause::LifetimeOutlives(..) => {}
                WhereClause::TypeOutlives(..) => {}
            });
        }
    });
}
