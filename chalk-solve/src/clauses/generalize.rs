//! This gets rid of free variables in a type by replacing them by fresh bound
//! ones. We use this when building clauses that contain types passed to
//! `program_clauses`; these may contain variables, and just copying those
//! variables verbatim leads to problems. Instead, we return a slightly more
//! general program clause, with new variables in those places. This can only
//! happen with `dyn Trait` currently; that's the only case where we use the
//! types passed to `program_clauses` in the clauses we generate.

use chalk_ir::{
    fold::{Fold, Folder},
    interner::{HasInterner, Interner},
    Binders, BoundVar, Const, ConstData, ConstValue, DebruijnIndex, Fallible, Lifetime,
    LifetimeData, NoSolution, Ty, TyKind, TyVariableKind, VariableKind, VariableKinds,
};
use rustc_hash::FxHashMap;

pub struct Generalize<I: Interner> {
    binders: Vec<VariableKind<I>>,
    mapping: FxHashMap<BoundVar, usize>,
    interner: I,
}

impl<I: Interner> Generalize<I> {
    pub fn apply<T>(interner: I, value: T) -> Binders<T::Result>
    where
        T: HasInterner<Interner = I> + Fold<I>,
        T::Result: HasInterner<Interner = I>,
    {
        let mut generalize = Generalize {
            binders: Vec::new(),
            mapping: FxHashMap::default(),
            interner,
        };
        let value = value
            .fold_with(&mut generalize, DebruijnIndex::INNERMOST)
            .unwrap();
        Binders::new(
            VariableKinds::from_iter(interner, generalize.binders),
            value,
        )
    }
}

impl<I: Interner> Folder<I> for Generalize<I> {
    type Error = NoSolution;

    fn as_dyn(&mut self) -> &mut dyn Folder<I, Error = Self::Error> {
        self
    }

    fn fold_free_var_ty(
        &mut self,
        bound_var: BoundVar,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Ty<I>> {
        let binder_vec = &mut self.binders;
        let new_index = self.mapping.entry(bound_var).or_insert_with(|| {
            let i = binder_vec.len();
            binder_vec.push(VariableKind::Ty(TyVariableKind::General));
            i
        });
        let new_var = BoundVar::new(outer_binder, *new_index);
        Ok(TyKind::BoundVar(new_var).intern(self.interner()))
    }

    fn fold_free_var_const(
        &mut self,
        ty: Ty<I>,
        bound_var: BoundVar,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Const<I>> {
        let binder_vec = &mut self.binders;
        let new_index = self.mapping.entry(bound_var).or_insert_with(|| {
            let i = binder_vec.len();
            binder_vec.push(VariableKind::Const(ty.clone()));
            i
        });
        let new_var = BoundVar::new(outer_binder, *new_index);
        Ok(ConstData {
            ty,
            value: ConstValue::BoundVar(new_var),
        }
        .intern(self.interner()))
    }

    fn fold_free_var_lifetime(
        &mut self,
        bound_var: BoundVar,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Lifetime<I>> {
        let binder_vec = &mut self.binders;
        let new_index = self.mapping.entry(bound_var).or_insert_with(|| {
            let i = binder_vec.len();
            binder_vec.push(VariableKind::Lifetime);
            i
        });
        let new_var = BoundVar::new(outer_binder, *new_index);
        Ok(LifetimeData::BoundVar(new_var).intern(self.interner()))
    }

    fn interner(&self) -> I {
        self.interner
    }
}
