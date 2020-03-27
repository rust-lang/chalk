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
                DebruijnIndex::INNERMOST,
            )
            .unwrap()
    }
}

struct DeepNormalizer<'table, 'i, I: Interner> {
    table: &'table mut InferenceTable<I>,
    interner: &'i I,
}

impl<'i, I: Interner> Folder<'i, I> for DeepNormalizer<'_, 'i, I>
where
    I: 'i,
{
    fn as_dyn(&mut self) -> &mut dyn Folder<'i, I> {
        self
    }

    fn fold_inference_ty(
        &mut self,
        var: InferenceVar,
        _outer_binder: DebruijnIndex,
    ) -> Fallible<Ty<I>> {
        let interner = self.interner;
        let var = EnaVariable::from(var);
        match self.table.probe_ty_var(interner, var) {
            Some(ty) => Ok(ty
                .fold_with(self, DebruijnIndex::INNERMOST)?
                .shifted_in(interner)), // FIXME shift
            None => Ok(var.to_ty(interner)),
        }
    }

    fn fold_inference_lifetime(
        &mut self,
        var: InferenceVar,
        _outer_binder: DebruijnIndex,
    ) -> Fallible<Lifetime<I>> {
        let interner = self.interner;
        let var = EnaVariable::from(var);
        match self.table.probe_lifetime_var(interner, var) {
            Some(l) => Ok(l
                .fold_with(self, DebruijnIndex::INNERMOST)?
                .shifted_in(interner)),
            None => Ok(var.to_lifetime(interner)), // FIXME shift
        }
    }

    fn forbid_free_vars(&self) -> bool {
        true
    }

    fn interner(&self) -> &'i I {
        self.interner
    }

    fn target_interner(&self) -> &'i I {
        self.interner()
    }
}
