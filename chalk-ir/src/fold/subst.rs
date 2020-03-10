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

    fn fold_free_var_ty(&mut self, depth: usize, binders: usize) -> Fallible<Ty<I>> {
        if depth >= self.parameters.len() {
            Ok(
                TyData::<I>::BoundVar(depth - self.parameters.len() + binders)
                    .intern(self.interner()),
            )
        } else {
            match self.parameters[depth].data() {
                ParameterKind::Ty(t) => Ok(t.shifted_in(self.interner(), binders)),
                _ => panic!("mismatched kinds in substitution"),
            }
        }
    }

    fn fold_free_var_lifetime(&mut self, depth: usize, binders: usize) -> Fallible<Lifetime<I>> {
        if depth >= self.parameters.len() {
            Ok(
                LifetimeData::<I>::BoundVar(depth - self.parameters.len() + binders)
                    .intern(self.interner()),
            )
        } else {
            match self.parameters[depth].data() {
                ParameterKind::Lifetime(l) => Ok(l.shifted_in(self.interner(), binders)),
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
