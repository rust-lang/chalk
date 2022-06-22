//! Shifting of debruijn indices

use crate::*;

/// Methods for converting debruijn indices to move values into or out
/// of binders.
pub trait Shift<I: Interner>: TypeFoldable<I> {
    /// Shifts this term in one level of binders.
    fn shifted_in(self, interner: I) -> Self;

    /// Shifts a term valid at `outer_binder` so that it is
    /// valid at the innermost binder. See [`DebruijnIndex::shifted_in_from`]
    /// for a detailed explanation.
    fn shifted_in_from(self, interner: I, source_binder: DebruijnIndex) -> Self;

    /// Shifts this term out one level of binders.
    fn shifted_out(self, interner: I) -> Fallible<Self>;

    /// Shifts a term valid at the innermost binder so that it is
    /// valid at `outer_binder`. See [`DebruijnIndex::shifted_out_to`]
    /// for a detailed explanation.
    fn shifted_out_to(self, interner: I, target_binder: DebruijnIndex) -> Fallible<Self>;
}

impl<T: TypeFoldable<I>, I: Interner> Shift<I> for T {
    fn shifted_in(self, interner: I) -> Self {
        self.shifted_in_from(interner, DebruijnIndex::ONE)
    }

    fn shifted_in_from(self, interner: I, source_binder: DebruijnIndex) -> T {
        self.try_fold_with(
            &mut Shifter {
                source_binder,
                interner,
            },
            DebruijnIndex::INNERMOST,
        )
        .unwrap()
    }

    fn shifted_out_to(self, interner: I, target_binder: DebruijnIndex) -> Fallible<T> {
        self.try_fold_with(
            &mut DownShifter {
                target_binder,
                interner,
            },
            DebruijnIndex::INNERMOST,
        )
    }

    fn shifted_out(self, interner: I) -> Fallible<Self> {
        self.shifted_out_to(interner, DebruijnIndex::ONE)
    }
}

/// A folder that adjusts debruijn indices by a certain amount.
#[derive(FallibleTypeFolder)]
struct Shifter<I: Interner> {
    source_binder: DebruijnIndex,
    interner: I,
}

impl<I: Interner> Shifter<I> {
    /// Given a free variable at `depth`, shifts that depth to `depth
    /// + self.adjustment`, and then wraps *that* within the internal
    /// set `binders`.
    fn adjust(&self, bound_var: BoundVar, outer_binder: DebruijnIndex) -> BoundVar {
        bound_var
            .shifted_in_from(self.source_binder)
            .shifted_in_from(outer_binder)
    }
}

impl<I: Interner> TypeFolder<I> for Shifter<I> {
    fn as_dyn(&mut self) -> &mut dyn TypeFolder<I> {
        self
    }

    fn fold_free_var_ty(&mut self, bound_var: BoundVar, outer_binder: DebruijnIndex) -> Ty<I> {
        TyKind::<I>::BoundVar(self.adjust(bound_var, outer_binder))
            .intern(TypeFolder::interner(self))
    }

    fn fold_free_var_lifetime(
        &mut self,
        bound_var: BoundVar,
        outer_binder: DebruijnIndex,
    ) -> Lifetime<I> {
        LifetimeData::<I>::BoundVar(self.adjust(bound_var, outer_binder))
            .intern(TypeFolder::interner(self))
    }

    fn fold_free_var_const(
        &mut self,
        ty: Ty<I>,
        bound_var: BoundVar,
        outer_binder: DebruijnIndex,
    ) -> Const<I> {
        // const types don't have free variables, so we can skip folding `ty`
        self.adjust(bound_var, outer_binder)
            .to_const(TypeFolder::interner(self), ty)
    }

    fn interner(&self) -> I {
        self.interner
    }
}

//---------------------------------------------------------------------------

/// A shifter that reduces debruijn indices -- in other words, which lifts a value
/// *out* from binders. Consider this example:
///
struct DownShifter<I> {
    target_binder: DebruijnIndex,
    interner: I,
}

impl<I> DownShifter<I> {
    /// Given a reference to a free variable at depth `depth`
    /// (appearing within `binders` internal binders), attempts to
    /// lift that free variable out from `adjustment` levels of
    /// binders (i.e., convert it to depth `depth -
    /// self.adjustment`). If the free variable is bound by one of
    /// those internal binders (i.e., `depth < self.adjustment`) the
    /// this will fail with `Err`. Otherwise, returns the variable at
    /// this new depth (but adjusted to appear within `binders`).
    fn adjust(&self, bound_var: BoundVar, outer_binder: DebruijnIndex) -> Fallible<BoundVar> {
        match bound_var.shifted_out_to(self.target_binder) {
            Some(bound_var1) => Ok(bound_var1.shifted_in_from(outer_binder)),
            None => Err(NoSolution),
        }
    }
}

impl<I: Interner> FallibleTypeFolder<I> for DownShifter<I> {
    type Error = NoSolution;

    fn as_dyn(&mut self) -> &mut dyn FallibleTypeFolder<I, Error = Self::Error> {
        self
    }

    fn try_fold_free_var_ty(
        &mut self,
        bound_var: BoundVar,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Ty<I>> {
        Ok(TyKind::<I>::BoundVar(self.adjust(bound_var, outer_binder)?).intern(self.interner()))
    }

    fn try_fold_free_var_lifetime(
        &mut self,
        bound_var: BoundVar,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Lifetime<I>> {
        Ok(
            LifetimeData::<I>::BoundVar(self.adjust(bound_var, outer_binder)?)
                .intern(self.interner()),
        )
    }

    fn try_fold_free_var_const(
        &mut self,
        ty: Ty<I>,
        bound_var: BoundVar,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Const<I>> {
        // const types don't have free variables, so we can skip folding `ty`
        Ok(self
            .adjust(bound_var, outer_binder)?
            .to_const(self.interner(), ty))
    }

    fn interner(&self) -> I {
        self.interner
    }
}
