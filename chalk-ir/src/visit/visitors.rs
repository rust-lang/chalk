use crate::{BoundVar, DebruijnIndex, Interner, Visit, VisitResult, Visitor};

pub trait VisitExt<I: Interner>: Visit<I> {
    fn has_free_vars(&self, interner: &I) -> bool {
        self.visit_with(
            &mut FindFreeVarsVisitor { interner },
            DebruijnIndex::INNERMOST,
        )
        .to_bool()
    }
}

impl<T, I: Interner> VisitExt<I> for T where T: Visit<I> {}

#[derive(Clone, Copy, Debug)]
pub struct FindAny {
    pub found: bool,
}

impl FindAny {
    pub fn to_bool(&self) -> bool {
        self.found
    }
}

impl VisitResult for FindAny {
    fn new() -> Self {
        FindAny { found: false }
    }
    fn and_then(self, op: impl FnOnce() -> Self) -> Self {
        if self.found {
            self
        } else {
            op()
        }
    }
}

pub struct FindFreeVarsVisitor<'i, I: Interner> {
    pub interner: &'i I,
}

impl<'i, I: Interner> Visitor<'i, I> for FindFreeVarsVisitor<'i, I> {
    type Result = FindAny;

    fn as_dyn(&mut self) -> &mut dyn Visitor<'i, I, Result = Self::Result> {
        self
    }

    fn interner(&self) -> &'i I {
        self.interner
    }

    fn visit_free_var_ty(
        &mut self,
        _bound_var: BoundVar,
        _outer_binder: DebruijnIndex,
    ) -> Self::Result {
        FindAny { found: true }
    }

    fn visit_free_var_lifetime(
        &mut self,
        _bound_var: BoundVar,
        _outer_binder: DebruijnIndex,
    ) -> Self::Result {
        FindAny { found: true }
    }
}
