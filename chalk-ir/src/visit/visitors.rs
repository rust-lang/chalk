//! Visitor helpers

use crate::{BoundVar, ControlFlow, DebruijnIndex, Interner, Visit, Visitor};

/// Visitor extensions.
pub trait VisitExt<I: Interner>: Visit<I> {
    /// Check whether there are free (non-bound) variables.
    fn has_free_vars(&self, interner: &I) -> bool {
        self.visit_with(
            &mut FindFreeVarsVisitor { interner },
            DebruijnIndex::INNERMOST,
        )
        .is_break()
    }
}

impl<T, I: Interner> VisitExt<I> for T where T: Visit<I> {}

struct FindFreeVarsVisitor<'i, I: Interner> {
    interner: &'i I,
}

impl<'i, I: Interner> Visitor<'i, I> for FindFreeVarsVisitor<'i, I> {
    type BreakTy = ();

    fn as_dyn(&mut self) -> &mut dyn Visitor<'i, I, BreakTy = Self::BreakTy> {
        self
    }

    fn interner(&self) -> &'i I {
        self.interner
    }

    fn visit_free_var(
        &mut self,
        _bound_var: BoundVar,
        _outer_binder: DebruijnIndex,
    ) -> ControlFlow<()> {
        ControlFlow::BREAK
    }
}
