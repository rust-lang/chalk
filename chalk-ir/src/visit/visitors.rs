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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FindAny {
    pub found: bool,
}

impl FindAny {
    pub const FOUND: FindAny = FindAny { found: true };

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

    fn visit_free_var_ty(
        &mut self,
        _bound_var: BoundVar,
        _outer_binder: DebruijnIndex,
    ) -> Self::Result {
        FindAny::FOUND
    }

    fn visit_free_var_lifetime(
        &mut self,
        _bound_var: BoundVar,
        _outer_binder: DebruijnIndex,
    ) -> Self::Result {
        FindAny::FOUND
    }
}
