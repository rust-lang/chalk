//! Visitor helpers

use crate::{BoundVar, DebruijnIndex, Interner, Visit, VisitResult, Visitor};

/// Visitor extensions.
pub trait VisitExt<I: Interner>: Visit<I> {
    /// Check whether there are free (non-bound) variables.
    fn has_free_vars(&self, interner: &I) -> bool {
        self.visit_with(
            &mut FindFreeVarsVisitor { interner },
            DebruijnIndex::INNERMOST,
        )
        .to_bool()
    }
}

impl<T, I: Interner> VisitExt<I> for T where T: Visit<I> {}

/// Helper visitor for finding a specific value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct FindAny {
    pub found: bool,
}

impl FindAny {
    /// Visitor has found the value.
    pub const FOUND: FindAny = FindAny { found: true };

    /// Checks whether the value has been found.
    pub fn to_bool(&self) -> bool {
        self.found
    }
}

impl VisitResult for FindAny {
    fn new() -> Self {
        FindAny { found: false }
    }

    fn return_early(&self) -> bool {
        self.found
    }
    fn combine(self, other: Self) -> Self {
        FindAny {
            found: self.found || other.found,
        }
    }
}

struct FindFreeVarsVisitor<'i, I: Interner> {
    interner: &'i I,
}

impl<'i, I: Interner> Visitor<'i, I> for FindFreeVarsVisitor<'i, I> {
    type Result = FindAny;

    fn as_dyn(&mut self) -> &mut dyn Visitor<'i, I, Result = Self::Result> {
        self
    }

    fn interner(&self) -> &'i I {
        self.interner
    }

    fn visit_free_var(
        &mut self,
        _bound_var: BoundVar,
        _outer_binder: DebruijnIndex,
    ) -> Self::Result {
        FindAny::FOUND
    }
}
