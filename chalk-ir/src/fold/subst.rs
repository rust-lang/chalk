use super::*;
use crate::fold::shift::Shift;

/// Substitution used during folding
#[derive(FallibleTypeFolder)]
pub struct Subst<'s, I: Interner> {
    /// Values to substitute. A reference to a free variable with
    /// index `i` will be mapped to `parameters[i]` -- if `i >
    /// parameters.len()`, then we will leave the variable untouched.
    parameters: &'s [GenericArg<I>],
    interner: I,
}

impl<I: Interner> Subst<'_, I> {
    /// Applies the substitution by folding
    pub fn apply<T: TypeFoldable<I>>(interner: I, parameters: &[GenericArg<I>], value: T) -> T {
        value
            .try_fold_with(
                &mut Subst {
                    parameters,
                    interner,
                },
                DebruijnIndex::INNERMOST,
            )
            .unwrap()
    }
}

impl<I: Interner> TypeFolder<I> for Subst<'_, I> {
    fn as_dyn(&mut self) -> &mut dyn TypeFolder<I> {
        self
    }

    /// We are eliminating one binder, but binders outside of that get preserved.
    ///
    /// So e.g. consider this:
    ///
    /// ```notrust
    /// for<A, B> { for<C> { [A, C] } }
    /// //          ^ the binder we are substituing with `[u32]`
    /// ```
    ///
    /// Here, `A` would be `^1.0` and `C` would be `^0.0`. We will replace `^0.0` with the
    /// 0th index from the list (`u32`). We will convert `^1.0` (A) to `^0.0` -- i.e., shift
    /// it **out** of one level of binder (the `for<C>` binder we are eliminating).
    ///
    /// This gives us as a result:
    ///
    /// ```notrust
    /// for<A, B> { [A, u32] }
    ///              ^ represented as `^0.0`
    /// ```
    fn fold_free_var_ty(&mut self, bound_var: BoundVar, outer_binder: DebruijnIndex) -> Ty<I> {
        if let Some(index) = bound_var.index_if_innermost() {
            match self.parameters[index].data(TypeFolder::interner(self)) {
                GenericArgData::Ty(t) => t
                    .clone()
                    .shifted_in_from(TypeFolder::interner(self), outer_binder),
                _ => panic!("mismatched kinds in substitution"),
            }
        } else {
            bound_var
                .shifted_out()
                .expect("cannot fail because this is not the innermost")
                .shifted_in_from(outer_binder)
                .to_ty(TypeFolder::interner(self))
        }
    }

    /// see `fold_free_var_ty`
    fn fold_free_var_lifetime(
        &mut self,
        bound_var: BoundVar,
        outer_binder: DebruijnIndex,
    ) -> Lifetime<I> {
        if let Some(index) = bound_var.index_if_innermost() {
            match self.parameters[index].data(TypeFolder::interner(self)) {
                GenericArgData::Lifetime(l) => l
                    .clone()
                    .shifted_in_from(TypeFolder::interner(self), outer_binder),
                _ => panic!("mismatched kinds in substitution"),
            }
        } else {
            bound_var
                .shifted_out()
                .unwrap()
                .shifted_in_from(outer_binder)
                .to_lifetime(TypeFolder::interner(self))
        }
    }

    /// see `fold_free_var_ty`
    fn fold_free_var_const(
        &mut self,
        ty: Ty<I>,
        bound_var: BoundVar,
        outer_binder: DebruijnIndex,
    ) -> Const<I> {
        if let Some(index) = bound_var.index_if_innermost() {
            match self.parameters[index].data(TypeFolder::interner(self)) {
                GenericArgData::Const(c) => c
                    .clone()
                    .shifted_in_from(TypeFolder::interner(self), outer_binder),
                _ => panic!("mismatched kinds in substitution"),
            }
        } else {
            bound_var
                .shifted_out()
                .unwrap()
                .shifted_in_from(outer_binder)
                .to_const(TypeFolder::interner(self), ty)
        }
    }

    fn interner(&self) -> I {
        self.interner
    }
}
