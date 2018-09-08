use chalk_engine::fallible::*;
use ir::fold::{DefaultTypeFolder, ExistentialFolder, Fold, UniversalFolder};
use ir::fold::shift::Shift;
use ir::*;
use std::collections::HashMap;

use super::{InferenceTable, InferenceVariable};
use super::canonicalize::Canonicalized;

impl InferenceTable {
    /// Converts `value` into a "negation" value -- meaning one that,
    /// if we can find any answer to it, then the negation fails. For
    /// goals that do not contain any free variables, then this is a
    /// no-op operation.
    ///
    /// If `value` contains any existential variables that have not
    /// yet been assigned a value, then this function will return
    /// `None`, indicating that we cannot prove negation for this goal
    /// yet.  This follows the approach in Clark's original
    /// negation-as-failure paper [1], where negative goals are only
    /// permitted if they contain no free (existential) variables.
    ///
    /// [1] http://www.doc.ic.ac.uk/~klc/NegAsFailure.pdf
    ///
    /// Restricting free existential variables is done because the
    /// semantics of such queries is not what you expect: it basically
    /// treats the existential as a universal. For example, consider:
    ///
    /// ```rust,ignore
    /// struct Vec<T> {}
    /// struct i32 {}
    /// struct u32 {}
    /// trait Foo {}
    /// impl Foo for Vec<u32> {}
    /// ```
    ///
    /// If we ask `exists<T> { not { Vec<T>: Foo } }`, what should happen?
    /// If we allow negative queries to be definitively answered even when
    /// they contain free variables, we will get a definitive *no* to the
    /// entire goal! From a logical perspective, that's just wrong: there
    /// does exists a `T` such that `not { Vec<T>: Foo }`, namely `i32`. The
    /// problem is that the proof search procedure is actually trying to
    /// prove something stronger, that there is *no* such `T`.
    ///
    /// An additional complication arises around free universal
    /// variables.  Consider a query like `not { !0 = !1 }`, where
    /// `!0` and `!1` represent universally quantified types (i.e.,
    /// `TypeName::ForAll`). If we just tried to prove `!0 = !1`, we
    /// would get false, because those types cannot be unified -- this
    /// would then allow us to conclude that `not { !0 = !1 }`, i.e.,
    /// `forall<X, Y> { not { X = Y } }`, but this is clearly not true
    /// -- what if X were to be equal to Y?
    ///
    /// Interestingly, the semantics of existential variables turns
    /// out to be exactly what we want here. So, in addition to
    /// forbidding existential variables in the original query, the
    /// `negated` query also converts all universals *into*
    /// existentials. Hence `negated` applies to `!0 = !1` would yield
    /// `exists<X,Y> { X = Y }` (note that a canonical, i.e. closed,
    /// result is returned). Naturally this has a solution, and hence
    /// `not { !0 = !1 }` fails, as we expect.
    ///
    /// (One could imagine converting free existentials into
    /// universals, rather than forbidding them altogether. This would
    /// be conveivable, but overly strict. For example, the goal
    /// `exists<T> { not { ?T: Clone }, ?T = Vec<i32> }` would come
    /// back as false, when clearly this is true. This is because we
    /// would wind up proving that `?T: Clone` can *never* be
    /// satisfied (which is false), when we only really care about
    /// `?T: Clone` in the case where `?T = Vec<i32>`. The current
    /// version would delay processing the negative goal (i.e., return
    /// `None`) until the second unification has occurred.)
    crate fn invert<T>(&mut self, value: &T) -> Option<T::Result>
    where
        T: Fold<Result = T>,
    {
        let Canonicalized {
            free_vars,
            quantified,
            ..
        } = self.canonicalize(&value);

        // If the original contains free existential variables, give up.
        if !free_vars.is_empty() {
            return None;
        }

        // If this contains free universal variables, replace them with existentials.
        assert!(quantified.binders.is_empty());
        let inverted = quantified
            .value
            .fold_with(&mut Inverter::new(self), 0)
            .unwrap();
        Some(inverted)
    }
}

struct Inverter<'q> {
    table: &'q mut InferenceTable,
    inverted_ty: HashMap<UniverseIndex, InferenceVariable>,
    inverted_lifetime: HashMap<UniverseIndex, InferenceVariable>,
}

impl<'q> Inverter<'q> {
    fn new(table: &'q mut InferenceTable) -> Self {
        Inverter {
            table,
            inverted_ty: HashMap::new(),
            inverted_lifetime: HashMap::new(),
        }
    }
}

impl<'q> DefaultTypeFolder for Inverter<'q> {}

impl<'q> UniversalFolder for Inverter<'q> {
    fn fold_free_universal_ty(&mut self, universe: UniverseIndex, binders: usize) -> Fallible<Ty> {
        let table = &mut self.table;
        Ok(
            self.inverted_ty
                .entry(universe)
                .or_insert_with(|| table.new_variable(universe))
                .to_ty()
                .shifted_in(binders),
        )
    }

    fn fold_free_universal_lifetime(
        &mut self,
        universe: UniverseIndex,
        binders: usize,
    ) -> Fallible<Lifetime> {
        let table = &mut self.table;
        Ok(
            self.inverted_lifetime
                .entry(universe)
                .or_insert_with(|| table.new_variable(universe))
                .to_lifetime()
                .shifted_in(binders),
        )
    }
}

impl<'q> ExistentialFolder for Inverter<'q> {
    fn fold_free_existential_ty(&mut self, _depth: usize, _binders: usize) -> Fallible<Ty> {
        panic!("should not be any existentials")
    }

    fn fold_free_existential_lifetime(
        &mut self,
        _depth: usize,
        _binders: usize,
    ) -> Fallible<Lifetime> {
        panic!("should not be any existentials")
    }
}
