use super::*;
use crate::fold::shift::Shift;

pub struct Subst<'s, TF: TypeFamily> {
    /// Values to substitute. A reference to a free variable with
    /// index `i` will be mapped to `parameters[i]` -- if `i >
    /// parameters.len()`, then we will leave the variable untouched.
    parameters: &'s [Parameter<TF>],
}

impl<TF: TypeFamily> Subst<'_, TF> {
    pub fn apply<T: Fold<TF, TF>>(parameters: &[Parameter<TF>], value: &T) -> T::Result {
        value.fold_with(&mut Subst { parameters }, 0).unwrap()
    }
}

impl<TF: TypeFamily> Folder<TF> for Subst<'_, TF> {
    fn as_dyn(&mut self) -> &mut dyn Folder<TF> {
        self
    }

    fn fold_free_var_ty(&mut self, depth: usize, binders: usize) -> Fallible<Ty<TF>> {
        if depth >= self.parameters.len() {
            Ok(TyData::<TF>::BoundVar(depth - self.parameters.len() + binders).intern())
        } else {
            match self.parameters[depth].data() {
                ParameterKind::Ty(t) => Ok(t.shifted_in(binders)),
                _ => panic!("mismatched kinds in substitution"),
            }
        }
    }

    fn fold_free_var_lifetime(&mut self, depth: usize, binders: usize) -> Fallible<Lifetime<TF>> {
        if depth >= self.parameters.len() {
            Ok(LifetimeData::<TF>::BoundVar(depth - self.parameters.len() + binders).intern())
        } else {
            match self.parameters[depth].data() {
                ParameterKind::Lifetime(l) => Ok(l.shifted_in(binders)),
                _ => panic!("mismatched kinds in substitution"),
            }
        }
    }
}
