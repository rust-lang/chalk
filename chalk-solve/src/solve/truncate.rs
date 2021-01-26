//!

use crate::infer::InferenceTable;
use chalk_ir::interner::Interner;
use chalk_ir::visit::{ControlFlow, SuperVisit, Visit, Visitor};
use chalk_ir::*;
use std::cmp::max;

/// "Truncation" (called "abstraction" in the papers referenced below)
/// refers to the act of modifying a goal or answer that has become
/// too large in order to guarantee termination.
///
/// Currently we don't perform truncation (but it might me readded later).
///
/// Citations:
///
/// - Terminating Evaluation of Logic Programs with Finite Three-Valued Models
///   - Riguzzi and Swift; ACM Transactions on Computational Logic 2013
/// - Radial Restraint
///   - Grosof and Swift; 2013
pub fn needs_truncation<I: Interner>(
    interner: &I,
    infer: &mut InferenceTable<I>,
    max_size: usize,
    value: impl Visit<I>,
) -> bool {
    let mut visitor = TySizeVisitor::new(interner, infer);
    value.visit_with(&mut visitor, DebruijnIndex::INNERMOST);

    visitor.max_size > max_size
}

struct TySizeVisitor<'infer, 'i, I: Interner> {
    interner: &'i I,
    infer: &'infer mut InferenceTable<I>,
    size: usize,
    depth: usize,
    max_size: usize,
}

impl<'infer, 'i, I: Interner> TySizeVisitor<'infer, 'i, I> {
    fn new(interner: &'i I, infer: &'infer mut InferenceTable<I>) -> Self {
        Self {
            interner,
            infer,
            size: 0,
            depth: 0,
            max_size: 0,
        }
    }
}

impl<'infer, 'i, I: Interner> Visitor<'i, I> for TySizeVisitor<'infer, 'i, I> {
    type BreakTy = ();

    fn as_dyn(&mut self) -> &mut dyn Visitor<'i, I, BreakTy = Self::BreakTy> {
        self
    }

    fn visit_ty(&mut self, ty: &Ty<I>, outer_binder: DebruijnIndex) -> ControlFlow<()> {
        if let Some(normalized_ty) = self.infer.normalize_ty_shallow(self.interner, ty) {
            normalized_ty.visit_with(self, outer_binder);
            return ControlFlow::CONTINUE;
        }

        self.size += 1;
        self.max_size = max(self.size, self.max_size);

        self.depth += 1;
        ty.super_visit_with(self, outer_binder);
        self.depth -= 1;

        // When we get back to the first invocation, clear the counters.
        // We process each outermost type independently.
        if self.depth == 0 {
            self.size = 0;
        }
        ControlFlow::CONTINUE
    }

    fn interner(&self) -> &'i I {
        self.interner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chalk_integration::{arg, ty};

    #[test]
    fn one_type() {
        use chalk_integration::interner::ChalkIr;
        let interner = &ChalkIr;
        let mut table = InferenceTable::<chalk_integration::interner::ChalkIr>::new();
        let _u1 = table.new_universe();

        // Vec<Vec<Vec<Vec<T>>>>
        let ty0 = ty!(apply (item 0)
                      (apply (item 0)
                       (apply (item 0)
                        (apply (item 0)
                         (placeholder 1)))));

        let mut visitor = TySizeVisitor::new(interner, &mut table);
        ty0.visit_with(&mut visitor, DebruijnIndex::INNERMOST);
        assert!(visitor.max_size == 5);
    }

    #[test]
    fn multiple_types() {
        use chalk_integration::interner::ChalkIr;
        let interner = &ChalkIr;
        let mut table = InferenceTable::<chalk_integration::interner::ChalkIr>::new();
        let _u1 = table.new_universe();

        // Vec<Vec<Vec<Vec<T>>>>
        let ty0 = ty!(apply (item 0)
                      (apply (item 0)
                       (apply (item 0)
                        (apply (item 0)
                         (placeholder 1)))));

        let ty1 = ty!(apply (item 0)
                      (apply (item 0)
                       (apply (item 0)
                        (placeholder 1))));

        let mut visitor = TySizeVisitor::new(interner, &mut table);
        vec![&ty0, &ty1].visit_with(&mut visitor, DebruijnIndex::INNERMOST);
        assert!(visitor.max_size == 5);
    }
}
