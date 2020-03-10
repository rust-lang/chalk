//!

use crate::infer::InferenceTable;
use chalk_engine::fallible::*;
use chalk_ir::fold::shift::Shift;
use chalk_ir::fold::{Fold, Folder, SuperFold};
use chalk_ir::interner::Interner;
use chalk_ir::*;
use std::fmt::Debug;

pub(crate) fn truncate<T, I>(
    interner: &I,
    infer: &mut InferenceTable<I>,
    max_size: usize,
    value: &T,
) -> Truncated<T::Result>
where
    I: Interner,
    T: Fold<I>,
    T::Result: Debug,
{
    debug_heading!("truncate(max_size={}, value={:?})", max_size, value);

    let mut truncater = Truncater::new(interner, infer, max_size);
    let value = value
        .fold_with(&mut truncater, 0)
        .expect("Truncater is infallible");
    debug!(
        "truncate: overflow={} value={:?}",
        truncater.overflow, value
    );
    Truncated {
        overflow: truncater.overflow,
        value,
    }
}

/// Result from `truncate`.
pub(crate) struct Truncated<T> {
    /// If true, then `value` was truncated relative to the original
    /// (e.g., fresh inference variables were introduced). If false,
    /// then it is effectively a clone of the original.
    pub(crate) overflow: bool,

    /// Possibly truncate value.
    pub(crate) value: T,
}

struct Truncater<'infer, 'i, I: Interner> {
    infer: &'infer mut InferenceTable<I>,
    current_size: usize,
    max_size: usize,
    overflow: bool,
    interner: &'i I,
}

impl<'infer, 'i, I: Interner> Truncater<'infer, 'i, I> {
    fn new(interner: &'i I, infer: &'infer mut InferenceTable<I>, max_size: usize) -> Self {
        Truncater {
            infer,
            current_size: 0,
            max_size,
            overflow: false,
            interner,
        }
    }

    fn overflow(&mut self, pre_size: usize) -> Ty<I> {
        self.overflow = true;
        self.current_size = pre_size + 1;
        let universe = self.infer.max_universe();
        self.infer.new_variable(universe).to_ty(self.interner())
    }
}

impl<'i, I: Interner> Folder<'i, I> for Truncater<'_, 'i, I>
where
    I: 'i,
{
    fn as_dyn(&mut self) -> &mut dyn Folder<'i, I> {
        self
    }

    fn fold_ty(&mut self, ty: &Ty<I>, binders: usize) -> Fallible<Ty<I>> {
        if let Some(normalized_ty) = self.infer.normalize_shallow(self.interner, ty) {
            return self.fold_ty(&normalized_ty, binders);
        }

        let pre_size = self.current_size;
        self.current_size += 1;

        let result = ty.super_fold_with(self, binders)?;

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
            self.overflow(pre_size).shifted_in(self.interner(), binders)
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

    fn fold_lifetime(&mut self, lifetime: &Lifetime<I>, binders: usize) -> Fallible<Lifetime<I>> {
        lifetime.super_fold_with(self, binders)
    }

    fn interner(&self) -> &'i I {
        self.interner
    }

    fn target_interner(&self) -> &'i I {
        self.interner()
    }
}

#[test]
fn truncate_types() {
    use chalk_ir::interner::ChalkIr;
    let interner = &ChalkIr;
    let mut table = InferenceTable::<chalk_ir::interner::ChalkIr>::new();
    let environment0 = &Environment::new();
    let _u1 = table.new_universe();

    // Vec<Vec<Vec<Vec<T>>>>
    let ty0 = ty!(apply (item 0)
                  (apply (item 0)
                   (apply (item 0)
                    (apply (item 0)
                     (placeholder 1)))));

    // test: no truncation with size 5
    let Truncated {
        overflow,
        value: ty_no_overflow,
    } = truncate(interner, &mut table, 5, &ty0);
    assert!(!overflow);
    assert_eq!(ty0, ty_no_overflow);

    // test: with size 3, truncates to `Vec<Vec<X>>`
    let ty_expect = ty!(apply (item 0)
                        (apply (item 0)
                         (infer 0)));
    let Truncated {
        overflow,
        value: ty_overflow,
    } = truncate(interner, &mut table, 3, &ty0);
    assert!(overflow);
    assert_eq!(ty_expect, ty_overflow);

    // test: the `X` is in u1, hence should fail to unify with a skolemized value in U2.
    let _u2 = table.new_universe();
    let ty_in_u2 = ty!(apply (item 0)
                       (apply (item 0)
                        (placeholder 2)));
    table
        .unify(interner, environment0, &ty_overflow, &ty_in_u2)
        .unwrap_err();
}

#[test]
fn truncate_multiple_types() {
    use chalk_ir::interner::ChalkIr;
    let interner = &ChalkIr;
    let mut table = InferenceTable::<chalk_ir::interner::ChalkIr>::new();
    let _u1 = table.new_universe();

    // Vec<Vec<Vec<Vec<T>>>>
    let ty0 = ty!(apply (item 0)
                  (apply (item 0)
                   (apply (item 0)
                    (apply (item 0)
                     (placeholder 1)))));

    // test: no truncation with size 5
    let ty0_3 = vec![ty0.clone(), ty0.clone(), ty0.clone()];
    let Truncated {
        overflow,
        value: ty_no_overflow,
    } = truncate(interner, &mut table, 5, &ty0_3);
    assert!(!overflow);
    assert_eq!(ty0_3, ty_no_overflow);

    // test: no truncation with size 6
    let ty0_3 = vec![ty0.clone(), ty0.clone(), ty0.clone()];
    let Truncated {
        overflow,
        value: ty_no_overflow,
    } = truncate(interner, &mut table, 6, &ty0_3);
    assert!(!overflow);
    assert_eq!(ty0_3, ty_no_overflow);

    // test: truncation of all types evenly with size 3
    let ty0_3 = vec![ty0.clone(), ty0.clone(), ty0.clone()];
    let Truncated {
        overflow,
        value: ty_overflow,
    } = truncate(interner, &mut table, 3, &ty0_3);
    assert!(overflow);
    assert_eq!(
        vec![
            ty!(apply (item 0) (apply (item 0) (infer 0))),
            ty!(apply (item 0) (apply (item 0) (infer 1))),
            ty!(apply (item 0) (apply (item 0) (infer 2))),
        ],
        ty_overflow
    );
}

#[test]
fn truncate_normalizes() {
    use chalk_ir::interner::ChalkIr;
    let interner = &ChalkIr;
    let mut table = InferenceTable::<chalk_ir::interner::ChalkIr>::new();

    let environment0 = &Environment::new();
    let u1 = table.new_universe();

    // ty0 = Vec<Vec<X>>
    let v0 = table.new_variable(u1);
    let ty0 = ty!(apply (item 0)
                  (apply (item 0)
                   (infer 0)));

    // ty1 = Vec<Vec<T>>
    let ty1 = ty!(apply (item 0)
                  (apply (item 0)
                   (placeholder 1)));

    // test: truncating *before* unifying has no effect
    assert!(!truncate(interner, &mut table, 3, &ty0).overflow);

    // unify X and ty1
    table
        .unify(interner, environment0, &v0.to_ty(interner), &ty1)
        .unwrap();

    // test: truncating *after* triggers
    let Truncated {
        overflow,
        value: ty_overflow,
    } = truncate(interner, &mut table, 3, &ty0);
    assert!(overflow);
    assert_eq!(
        ty!(apply (item 0)
            (apply (item 0)
             (infer 1))),
        ty_overflow
    );
}

#[test]
fn truncate_normalizes_under_binders() {
    use chalk_ir::interner::ChalkIr;
    let interner = &ChalkIr;
    let mut table = InferenceTable::<chalk_ir::interner::ChalkIr>::new();

    let u0 = UniverseIndex::ROOT;

    // v0 = X
    let _v0 = table.new_variable(u0);

    // ty0 = for<'a> Vec<Vec<X>>
    let ty0 = ty!(function 1
                  (apply (item 0)
                   (apply (item 0)
                    (infer 0))));

    assert!(!truncate(interner, &mut table, 4, &ty0).overflow);
}
