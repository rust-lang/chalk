//!

use fallible::*;
use fold::{self, Fold, IdentityExistentialFolder, IdentityUniversalFolder, TypeFolder};
use ir::*;
use solve::infer::InferenceTable;

pub fn truncate<T>(
    infer: &mut InferenceTable,
    universe: UniverseIndex,
    max_size: usize,
    value: &T,
) -> Truncated<T::Result>
where
    T: Fold,
{
    debug_heading!("truncate(max_size={}, value={:?})", max_size, value);

    let mut truncater = Truncater::new(infer, universe, max_size);
    let value = value
        .fold_with(&mut truncater, 0)
        .expect("Truncater is infallible");
    debug!("truncate: overflow={} value={:?}", truncater.overflow, value);
    Truncated {
        overflow: truncater.overflow,
        value,
    }
}

/// Result from `truncate`.
pub struct Truncated<T> {
    /// If true, then `value` was truncated relative to the original
    /// (e.g., fresh inference variables were introduced). If false,
    /// then it is effectively a clone of the original.
    pub overflow: bool,

    /// Possibly truncate value.
    pub value: T,
}

struct Truncater<'infer> {
    infer: &'infer mut InferenceTable,
    universe: UniverseIndex,
    current_size: usize,
    max_size: usize,
    overflow: bool,
}

impl<'infer> Truncater<'infer> {
    fn new(infer: &'infer mut InferenceTable, universe: UniverseIndex, max_size: usize) -> Self {
        Truncater {
            infer,
            universe,
            current_size: 0,
            max_size,
            overflow: false,
        }
    }

    fn overflow(&mut self, pre_size: usize) -> Ty {
        self.overflow = true;
        self.current_size = pre_size + 1;
        self.infer.new_variable(self.universe).to_ty()
    }
}

impl<'infer> TypeFolder for Truncater<'infer> {
    fn fold_ty(&mut self, ty: &Ty, binders: usize) -> Fallible<Ty> {
        if let Some(normalized_ty) = self.infer.normalize_shallow(ty, binders) {
            return self.fold_ty(&normalized_ty, binders);
        }

        let pre_size = self.current_size;
        self.current_size += 1;

        let result = fold::super_fold_ty(self, ty, binders)?;

        // We wish to maintain the invariant that:
        //
        //     pre_size < self.max_size =>
        //         post_size <= self.max_size
        //
        // Presuming that `pre_size < self.max_size`, then the
        // invariant is in jeopardy if `post_size > self.max_size`.
        // To repair the situation, we replace the entire subtree with
        // a fresh existential variable (in the innermost universe).
        let post_size = self.current_size;
        let result = if pre_size < self.max_size && post_size > self.max_size {
            self.overflow(pre_size).up_shift(binders)
        } else {
            result
        };

        // When we get back to the first invocation, clear the counters.
        // We process each type independently.
        if pre_size == 0 {
            self.current_size = 0;
        }

        Ok(result)
    }

    fn fold_lifetime(&mut self, lifetime: &Lifetime, binders: usize) -> Fallible<Lifetime> {
        fold::super_fold_lifetime(self, lifetime, binders)
    }
}

impl<'infer> IdentityExistentialFolder for Truncater<'infer> {}

impl<'infer> IdentityUniversalFolder for Truncater<'infer> {}

#[test]
fn truncate_types() {
    let mut table = InferenceTable::new();

    let environment0 = &Environment::new();
    let environment1 = &environment0.new_universe();
    let u1 = environment1.universe;
    let environment2 = &environment1.new_universe();

    // Vec<Vec<Vec<Vec<T>>>>
    let ty0 = ty!(apply (item 0)
                  (apply (item 0)
                   (apply (item 0)
                    (apply (item 0)
                     (apply (skol 1))))));

    // test: no truncation with size 5
    let Truncated {
        overflow,
        value: ty_no_overflow,
    } = truncate(&mut table, u1, 5, &ty0);
    assert!(!overflow);
    assert_eq!(ty0, ty_no_overflow);

    // test: with size 3, truncates to `Vec<Vec<X>>`
    let ty_expect = ty!(apply (item 0)
                        (apply (item 0)
                         (var 0)));
    let Truncated {
        overflow,
        value: ty_overflow,
    } = truncate(&mut table, u1, 3, &ty0);
    assert!(overflow);
    assert_eq!(ty_expect, ty_overflow);

    // test: the `X` is in u1, hence should fail to unify with a skolemized value in U2.
    let ty_in_u2 = ty!(apply (item 0)
                       (apply (item 0)
                        (apply (skol 2))));
    table
        .unify(environment2, &ty_overflow, &ty_in_u2)
        .unwrap_err();
}

#[test]
fn truncate_multiple_types() {
    let mut table = InferenceTable::new();

    let environment0 = &Environment::new();
    let environment1 = &environment0.new_universe();
    let u1 = environment1.universe;

    // Vec<Vec<Vec<Vec<T>>>>
    let ty0 = ty!(apply (item 0)
                  (apply (item 0)
                   (apply (item 0)
                    (apply (item 0)
                     (apply (skol 1))))));

    // test: no truncation with size 5
    let ty0_3 = vec![ty0.clone(), ty0.clone(), ty0.clone()];
    let Truncated {
        overflow,
        value: ty_no_overflow,
    } = truncate(&mut table, u1, 5, &ty0_3);
    assert!(!overflow);
    assert_eq!(ty0_3, ty_no_overflow);

    // test: no truncation with size 6
    let ty0_3 = vec![ty0.clone(), ty0.clone(), ty0.clone()];
    let Truncated {
        overflow,
        value: ty_no_overflow,
    } = truncate(&mut table, u1, 6, &ty0_3);
    assert!(!overflow);
    assert_eq!(ty0_3, ty_no_overflow);

    // test: truncation of all types evenly with size 3
    let ty0_3 = vec![ty0.clone(), ty0.clone(), ty0.clone()];
    let Truncated {
        overflow,
        value: ty_overflow,
    } = truncate(&mut table, u1, 3, &ty0_3);
    assert!(overflow);
    assert_eq!(
        vec![
            ty!(apply (item 0) (apply (item 0) (var 0))),
            ty!(apply (item 0) (apply (item 0) (var 1))),
            ty!(apply (item 0) (apply (item 0) (var 2))),
        ],
        ty_overflow
    );
}

#[test]
fn truncate_normalizes() {
    let mut table = InferenceTable::new();

    let environment0 = &Environment::new();
    let environment1 = &environment0.new_universe();
    let u1 = environment1.universe;

    // ty0 = Vec<Vec<X>>
    let v0 = table.new_variable(u1);
    let ty0 = ty!(apply (item 0)
                  (apply (item 0)
                   (var 0)));

    // ty1 = Vec<Vec<T>>
    let ty1 = ty!(apply (item 0)
                  (apply (item 0)
                   (apply (skol 1))));

    // test: truncating *before* unifying has no effect
    assert!(!truncate(&mut table, u1, 3, &ty0).overflow);

    // unify X and ty1
    table.unify(environment1, &v0.to_ty(), &ty1).unwrap();

    // test: truncating *after* triggers
    let Truncated {
        overflow,
        value: ty_overflow,
    } = truncate(&mut table, u1, 3, &ty0);
    assert!(overflow);
    assert_eq!(
        ty!(apply (item 0)
            (apply (item 0)
             (var 1))),
        ty_overflow);
}

#[test]
fn truncate_normalizes_under_binders() {
    let mut table = InferenceTable::new();

    let u0 = UniverseIndex::root();

    // v0 = X
    let _v0 = table.new_variable(u0);

    // ty0 = for<'a> Vec<Vec<X>>
    let ty0 = ty!(for_all 1
                  (apply (item 0)
                   (apply (item 0)
                    (var 1))));

    // the index in `(var 1)` should be adjusted to account for binders
    assert!(!truncate(&mut table, u0, 4, &ty0).overflow);
}

