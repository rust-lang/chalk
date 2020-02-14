use chalk_engine::fallible::*;
use chalk_ir::fold::shift::Shift;
use chalk_ir::fold::{Fold, Folder};
use chalk_ir::interner::Interner;
use chalk_ir::*;

use super::{EnaVariable, InferenceTable};

impl<I: Interner> InferenceTable<I> {
    /// Given a value `value` with variables in it, replaces those variables
    /// with their instantiated values (if any). Uninstantiated variables are
    /// left as-is.
    ///
    /// This is mainly intended for getting final values to dump to
    /// the user and its use should otherwise be avoided, particularly
    /// given the possibility of snapshots and rollbacks.
    ///
    /// See also `InferenceTable::canonicalize`, which -- during real
    /// processing -- is often used to capture the "current state" of
    /// variables.
    pub(crate) fn normalize_deep<T: Fold<I>>(&mut self, interner: &I, value: &T) -> T::Result {
        value
            .fold_with(
                &mut DeepNormalizer {
                    interner,
                    table: self,
                },
                0,
            )
            .unwrap()
    }
}

struct DeepNormalizer<'table, 'i, I: Interner> {
    table: &'table mut InferenceTable<I>,
    interner: &'i I,
}

impl<I: Interner> Folder<I> for DeepNormalizer<'_, '_, I> {
    fn as_dyn(&mut self) -> &mut dyn Folder<I> {
        self
    }

    fn fold_inference_ty(&mut self, var: InferenceVar, binders: usize) -> Fallible<Ty<I>> {
        let var = EnaVariable::from(var);
        match self.table.probe_ty_var(var) {
            Some(ty) => Ok(ty.fold_with(self, 0)?.shifted_in(self.interner(), binders)), // FIXME shift
            None => Ok(var.to_ty(self.interner())),
        }
    }

    fn fold_inference_lifetime(
        &mut self,
        var: InferenceVar,
        binders: usize,
    ) -> Fallible<Lifetime<I>> {
        let var = EnaVariable::from(var);
        match self.table.probe_lifetime_var(var) {
            Some(l) => Ok(l.fold_with(self, 0)?.shifted_in(self.interner(), binders)),
            None => Ok(var.to_lifetime()), // FIXME shift
        }
    }

    fn forbid_free_vars(&self) -> bool {
        true
    }

    fn interner(&self) -> &I {
        self.interner
    }

    fn target_interner(&self) -> &I {
        self.interner()
    }
}
