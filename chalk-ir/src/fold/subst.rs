use super::*;
use crate::fold::shift::Shift;

pub struct Subst<'s, 'i, I: Interner> {
    /// Values to substitute. A reference to a free variable with
    /// index `i` will be mapped to `parameter_lists[i]` -- if `i >
    /// parameter_lists.len()`, then we will leave the variable untouched.
    parameters: &'s [Parameter<I>],
    interner: &'i I,
}

impl<I: Interner> Subst<'_, '_, I> {
    pub fn apply<T: Fold<I, I>>(interner: &I, parameters: &[Parameter<I>], value: &T) -> T::Result {
        value
            .fold_with(
                &mut Subst {
                    parameters,
                    interner,
                },
                DebruijnIndex::INNERMOST,
            )
            .unwrap()
    }
}

impl<'i, I: Interner> Folder<'i, I> for Subst<'_, 'i, I> {
    fn as_dyn(&mut self) -> &mut dyn Folder<'i, I> {
        self
    }

    fn fold_free_var_ty(
        &mut self,
        bound_var: BoundVar,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Ty<I>> {
        // We are eliminating one binder, but binders outside of that get preserved.
        //
        // So e.g. consider this:
        //
        // ```
        // for<A, B> { for<C> { [A, C] } }
        // //          ^ the binder we are substituing with `[u32]`
        // ```
        //
        // Here, `A` would be `^1.0` and `C` would be `^0.0`. We will replace `^0.0` with the
        // 0th index from the list (`u32`). We will convert `^1.0` (A) to `^0.0` -- i.e., shift
        // it **out** of one level of binder (the `for<C>` binder we are eliminating).
        //
        // This gives us as a result:
        //
        // ```
        // for<A, B> { [A, u32] }
        //              ^ represented as `^0.0`
        // ```
        if let Some(index) = bound_var.index_if_innermost() {
            match self.parameters[index].data(self.interner()) {
                ParameterKind::Ty(t) => Ok(t.shifted_in_from(self.interner(), outer_binder)),
                _ => panic!("mismatched kinds in substitution"),
            }
        } else {
            Ok(bound_var
                .shifted_out()
                .expect("cannot fail because this is not the innermost")
                .shifted_in_from(outer_binder)
                .to_ty(self.interner()))
        }
    }

    fn fold_free_var_lifetime(
        &mut self,
        bound_var: BoundVar,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Lifetime<I>> {
        // see comment in `fold_free_var_ty`

        if let Some(index) = bound_var.index_if_innermost() {
            match self.parameters[index].data(self.interner()) {
                ParameterKind::Lifetime(l) => Ok(l.shifted_in_from(self.interner(), outer_binder)),
                _ => panic!("mismatched kinds in substitution"),
            }
        } else {
            Ok(bound_var
                .shifted_out()
                .unwrap()
                .shifted_in_from(outer_binder)
                .to_lifetime(self.interner()))
        }
    }

    fn interner(&self) -> &'i I {
        self.interner
    }

    fn target_interner(&self) -> &'i I {
        self.interner()
    }
}
