use chalk_engine::fallible::*;
use chalk_ir::family::TypeFamily;
use chalk_ir::fold::shift::Shift;
use chalk_ir::fold::{Fold, Folder};
use chalk_ir::*;

use super::{EnaVariable, InferenceTable};

impl<TF: TypeFamily> InferenceTable<TF> {
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
    pub(crate) fn normalize_deep<T: Fold<TF>>(&mut self, value: &T) -> T::Result {
        value
            .fold_with(&mut DeepNormalizer { table: self }, 0)
            .unwrap()
    }
}

struct DeepNormalizer<'table, TF: TypeFamily> {
    table: &'table mut InferenceTable<TF>,
}

impl<TF: TypeFamily> Folder<TF> for DeepNormalizer<'_, TF> {
    fn as_dyn(&mut self) -> &mut dyn Folder<TF> {
        self
    }

    fn fold_inference_ty(&mut self, var: InferenceVar, binders: usize) -> Fallible<Ty<TF>> {
        let var = EnaVariable::from(var);
        match self.table.probe_ty_var(var) {
            Some(ty) => Ok(ty.fold_with(self, 0)?.shifted_in(binders)), // FIXME shift
            None => Ok(var.to_ty()),
        }
    }

    fn fold_inference_lifetime(
        &mut self,
        var: InferenceVar,
        binders: usize,
    ) -> Fallible<Lifetime<TF>> {
        let var = EnaVariable::from(var);
        match self.table.probe_lifetime_var(var) {
            Some(l) => Ok(l.fold_with(self, 0)?.shifted_in(binders)),
            None => Ok(var.to_lifetime()), // FIXME shift
        }
    }

    fn forbid_free_vars(&self) -> bool {
        true
    }
}
