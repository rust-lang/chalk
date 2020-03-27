use super::Fold;
use crate::*;

/// Methods for converting debruijn indices to move values into or out
/// of binders.
pub trait Shift<I: Interner>: Fold<I, I> {
    /// Shifts debruijn indices in `self` **up**, which is used when a
    /// value is being placed under additional levels of binders.
    ///
    /// For example, if we had some goal
    /// like:
    ///
    /// ```notrust
    /// T: Trait<?X>
    /// ```
    ///
    /// where `?X` refers to some inference variable (and hence has depth 3),
    /// we might use `up_shift` when constructing a goal like:
    ///
    /// ```notrust
    /// exists<U> { T = U, T: Trait<?X> }
    /// ```
    ///
    /// This is because, internally, the inference variable `?X` (as
    /// well as the new quantified variable `U`) are going to be
    /// represented by debruijn indices. So if the index of `X` is
    /// zero, then while originally we might have had `T: Trait<?0>`,
    /// inside the `exists` we want to represent `X` with `?1`, to
    /// account for the binder:
    ///
    /// ```notrust
    ///     exists { T = ?0, T: Trait<?1> }
    ///                  ^^           ^^ refers to `?X`
    ///                  refers to `U`
    /// ```
    fn shifted_in(&self, interner: &I, adjustment: usize) -> Self::Result;

    /// Shifts debruijn indices in `self` **down**, hence **removing**
    /// a value from binders. This will fail with `Err(NoSolution)` in
    /// the case that the value refers to something from one of those
    /// binders.
    ///
    /// Consider the final example from `up_shift`:
    ///
    /// ```notrust
    ///     exists { T = ?0, T: Trait<?1> }
    ///                  ^^           ^^ refers to `?X`
    ///                  refers to `U`
    /// ```
    ///
    /// If we `down_shift` the `T: Trait<?1>` goal by 1,
    /// we will get `T: Trait<?0>`, which is what we started with.
    /// In other words, we will have extracted it from the `exists`
    /// binder.
    ///
    /// But if we try to `down_shift` the `T = ?0` goal by 1, we will
    /// get `Err`, because it refers to the type bound by the
    /// `exists`.
    fn shifted_out(&self, interner: &I, adjustment: usize) -> Fallible<Self::Result>;
}

impl<T: Fold<I, I> + Eq, I: Interner> Shift<I> for T {
    fn shifted_in(&self, interner: &I, adjustment: usize) -> T::Result {
        self.fold_with(
            &mut Shifter {
                adjustment,
                interner,
            },
            0,
        )
        .unwrap()
    }

    fn shifted_out(&self, interner: &I, adjustment: usize) -> Fallible<T::Result> {
        self.fold_with(
            &mut DownShifter {
                adjustment,
                interner,
            },
            0,
        )
    }
}

/// A folder that adjusts debruijn indices by a certain amount.
struct Shifter<'i, I> {
    adjustment: usize,
    interner: &'i I,
}

impl<I> Shifter<'_, I> {
    /// Given a free variable at `depth`, shifts that depth to `depth
    /// + self.adjustment`, and then wraps *that* within the internal
    /// set `binders`.
    fn adjust(&self, bound_var: BoundVar, binders: usize) -> BoundVar {
        bound_var.shifted_in_by(self.adjustment + binders)
    }
}

impl<'i, I: Interner> Folder<'i, I> for Shifter<'i, I> {
    fn as_dyn(&mut self) -> &mut dyn Folder<'i, I> {
        self
    }

    fn fold_free_var_ty(&mut self, bound_var: BoundVar, binders: usize) -> Fallible<Ty<I>> {
        Ok(TyData::<I>::BoundVar(self.adjust(bound_var, binders)).intern(self.interner()))
    }

    fn fold_free_var_lifetime(
        &mut self,
        bound_var: BoundVar,
        binders: usize,
    ) -> Fallible<Lifetime<I>> {
        Ok(LifetimeData::<I>::BoundVar(self.adjust(bound_var, binders)).intern(self.interner()))
    }

    fn interner(&self) -> &'i I {
        self.interner
    }

    fn target_interner(&self) -> &'i I {
        self.interner()
    }
}

//---------------------------------------------------------------------------

/// A shifter that reduces debruijn indices -- in other words, which lifts a value
/// *out* from binders. Consider this example:
///
struct DownShifter<'i, I> {
    adjustment: usize,
    interner: &'i I,
}

impl<I> DownShifter<'_, I> {
    /// Given a reference to a free variable at depth `depth`
    /// (appearing within `binders` internal binders), attempts to
    /// lift that free variable out from `adjustment` levels of
    /// binders (i.e., convert it to depth `depth -
    /// self.adjustment`). If the free variable is bound by one of
    /// those internal binders (i.e., `depth < self.adjustment`) the
    /// this will fail with `Err`. Otherwise, returns the variable at
    /// this new depth (but adjusted to appear within `binders`).
    fn adjust(&self, bound_var: BoundVar, binders: usize) -> Fallible<BoundVar> {
        match bound_var.shifted_out_by(self.adjustment) {
            Some(bound_var1) => Ok(bound_var1.shifted_in_by(binders)),
            None => Err(NoSolution),
        }
    }
}

impl<'i, I: Interner> Folder<'i, I> for DownShifter<'i, I> {
    fn as_dyn(&mut self) -> &mut dyn Folder<'i, I> {
        self
    }

    fn fold_free_var_ty(&mut self, bound_var: BoundVar, binders: usize) -> Fallible<Ty<I>> {
        Ok(TyData::<I>::BoundVar(self.adjust(bound_var, binders)?).intern(self.interner()))
    }

    fn fold_free_var_lifetime(
        &mut self,
        bound_var: BoundVar,
        binders: usize,
    ) -> Fallible<Lifetime<I>> {
        Ok(LifetimeData::<I>::BoundVar(self.adjust(bound_var, binders)?).intern(self.interner()))
    }

    fn interner(&self) -> &'i I {
        self.interner
    }

    fn target_interner(&self) -> &'i I {
        self.interner()
    }
}
