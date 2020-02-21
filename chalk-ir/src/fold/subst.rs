use super::*;
use crate::fold::shift::Shift;

pub struct Subst<'s, 'i, I: Interner> {
    /// Values to substitute. A reference to a free variable with
    /// index `i` will be mapped to `parameters[i]` -- if `i >
    /// parameters.len()`, then we will leave the variable untouched.
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
                0,
            )
            .unwrap()
    }
}

impl<'i, I: Interner> Folder<'i, I> for Subst<'_, 'i, I> {
    fn as_dyn(&mut self) -> &mut dyn Folder<'i, I> {
        self
    }

    fn fold_free_var_ty(&mut self, bound_var: BoundVar, binders: usize) -> Fallible<Ty<I>> {
        let interner = self.interner();
        let index = bound_var.index();
        if index >= self.parameters.len() {
            let debruijn = DebruijnIndex::from(index - self.parameters.len()).shifted_in(binders);
            Ok(TyData::<I>::BoundVar(BoundVar::new(debruijn)).intern(interner))
        } else {
            match self.parameters[index].data(interner) {
                ParameterKind::Ty(t) => Ok(t.shifted_in(interner, binders)),
                _ => panic!("mismatched kinds in substitution"),
            }
        }
    }

    fn fold_free_var_lifetime(
        &mut self,
        bound_var: BoundVar,
        binders: usize,
    ) -> Fallible<Lifetime<I>> {
        let interner = self.interner();
        let index = bound_var.index();
        if index >= self.parameters.len() {
            let debruijn = DebruijnIndex::from(index - self.parameters.len()).shifted_in(binders);
            Ok(LifetimeData::<I>::BoundVar(BoundVar::new(debruijn)).intern(interner))
        } else {
            match self.parameters[index].data(interner) {
                ParameterKind::Lifetime(l) => Ok(l.shifted_in(interner, binders)),
                _ => panic!("mismatched kinds in substitution"),
            }
        }
    }

    fn interner(&self) -> &'i I {
        self.interner
    }

    fn target_interner(&self) -> &'i I {
        self.interner()
    }
}
