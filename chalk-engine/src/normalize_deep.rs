use chalk_ir::fold::shift::Shift;
use chalk_ir::fold::{Fold, Folder};
use chalk_ir::interner::Interner;
use chalk_ir::*;
use chalk_solve::infer::InferenceTable;

pub(crate) struct DeepNormalizer<'table, 'i, I: Interner> {
    table: &'table mut InferenceTable<I>,
    interner: &'i I,
}

impl<I: Interner> DeepNormalizer<'_, '_, I> {
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
    pub fn normalize_deep<T: Fold<I>>(
        table: &mut InferenceTable<I>,
        interner: &I,
        value: &T,
    ) -> T::Result {
        value
            .fold_with(
                &mut DeepNormalizer { interner, table },
                DebruijnIndex::INNERMOST,
            )
            .unwrap()
    }
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
        kind: TyKind,
        _outer_binder: DebruijnIndex,
    ) -> Fallible<Ty<I>> {
        let interner = self.interner;
        match self.table.probe_var(var) {
            Some(ty) => Ok(ty
                .assert_ty_ref(interner)
                .fold_with(self, DebruijnIndex::INNERMOST)?
                .shifted_in(interner)), // FIXME shift
            None => Ok(var.to_ty(interner, kind)),
        }
    }

    fn fold_inference_lifetime(
        &mut self,
        var: InferenceVar,
        _outer_binder: DebruijnIndex,
    ) -> Fallible<Lifetime<I>> {
        let interner = self.interner;
        match self.table.probe_var(var) {
            Some(l) => Ok(l
                .assert_lifetime_ref(interner)
                .fold_with(self, DebruijnIndex::INNERMOST)?
                .shifted_in(interner)),
            None => Ok(var.to_lifetime(interner)), // FIXME shift
        }
    }

    fn fold_inference_const(
        &mut self,
        ty: &Ty<I>,
        var: InferenceVar,
        _outer_binder: DebruijnIndex,
    ) -> Fallible<Const<I>> {
        let interner = self.interner;
        match self.table.probe_var(var) {
            Some(c) => Ok(c
                .assert_const_ref(interner)
                .fold_with(self, DebruijnIndex::INNERMOST)?
                .shifted_in(interner)),
            None => Ok(var.to_const(interner, ty.clone())), // FIXME shift
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

#[cfg(test)]
mod test {
    use super::*;
    use chalk_integration::interner::ChalkIr;
    use chalk_integration::{arg, ty, ty_name};

    const U0: UniverseIndex = UniverseIndex { counter: 0 };

    #[test]
    fn infer() {
        let interner = &ChalkIr;
        let mut table: InferenceTable<ChalkIr> = InferenceTable::new();
        let environment0 = Environment::new(interner);
        let a = table.new_variable(U0).to_ty(interner);
        let b = table.new_variable(U0).to_ty(interner);
        table
            .unify(interner, &environment0, &a, &ty!(apply (item 0) (expr b)))
            .unwrap();
        assert_eq!(
            DeepNormalizer::normalize_deep(&mut table, interner, &a),
            ty!(apply (item 0) (expr b))
        );
        table
            .unify(interner, &environment0, &b, &ty!(apply (item 1)))
            .unwrap();
        assert_eq!(
            DeepNormalizer::normalize_deep(&mut table, interner, &a),
            ty!(apply (item 0) (apply (item 1)))
        );
    }
}
