//! TypeVisitor helpers

use crate::{BoundVar, ControlFlow, DebruijnIndex, Interner, TypeVisitable, TypeVisitor};

/// TypeVisitor extensions.
pub trait VisitExt<I: Interner>: TypeVisitable<I> {
    /// Check whether there are free (non-bound) variables.
    fn has_free_vars(&self, interner: I) -> bool {
        let flow = self.visit_with(
            &mut FindFreeVarsVisitor { interner },
            DebruijnIndex::INNERMOST,
        );
        matches!(flow, ControlFlow::Break(_))
    }
}

impl<T, I: Interner> VisitExt<I> for T where T: TypeVisitable<I> {}

struct FindFreeVarsVisitor<I: Interner> {
    interner: I,
}

impl<I: Interner> TypeVisitor<I> for FindFreeVarsVisitor<I> {
    type BreakTy = ();

    fn as_dyn(&mut self) -> &mut dyn TypeVisitor<I, BreakTy = Self::BreakTy> {
        self
    }

    fn interner(&self) -> I {
        self.interner
    }

    fn visit_free_var(
        &mut self,
        _bound_var: BoundVar,
        _outer_binder: DebruijnIndex,
    ) -> ControlFlow<()> {
        ControlFlow::Break(())
    }
}
