use errors::*;
use fold::{DefaultTypeFolder, ExistentialFolder, Fold, IdentityUniversalFolder};
use ir::*;

use super::{InferenceTable, TyInferenceVariable, LifetimeInferenceVariable};

impl InferenceTable {
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
    pub fn normalize_deep<T: Fold>(&mut self, value: &T) -> T::Result {
        value.fold_with(&mut DeepNormalizer { table: self }, 0).unwrap()
    }
}

struct DeepNormalizer<'table> {
    table: &'table mut InferenceTable,
}

impl<'table> DefaultTypeFolder for DeepNormalizer<'table> { }

impl<'table> IdentityUniversalFolder for DeepNormalizer<'table> {
}

impl<'table> ExistentialFolder for DeepNormalizer<'table> {
    fn fold_free_existential_ty(&mut self, depth: usize, binders: usize) -> Result<Ty> {
        let var = TyInferenceVariable::from_depth(depth);
        match self.table.probe_var(var) {
            Some(ty) => Ok(ty.fold_with(self, 0)?.up_shift(binders)),
            None => Ok(TyInferenceVariable::from_depth(depth + binders).to_ty())
        }
    }

    fn fold_free_existential_lifetime(&mut self, depth: usize, binders: usize) -> Result<Lifetime> {
        let var = LifetimeInferenceVariable::from_depth(depth);
        match self.table.probe_lifetime_var(var) {
            Some(l) => Ok(l.fold_with(self, 0)?.up_shift(binders)),
            None => Ok(LifetimeInferenceVariable::from_depth(depth + binders).to_lifetime()),
        }
    }
}
