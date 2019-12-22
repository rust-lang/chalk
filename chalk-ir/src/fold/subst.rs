use super::*;
use crate::fold::shift::Shift;

pub struct Subst<'s, TF: TypeFamily> {
    /// Values to substitute. A reference to a free variable with
    /// index `i` will be mapped to `parameters[i]` -- if `i >
    /// parameters.len()`, then we will leave the variable untouched.
    parameters: &'s [Parameter<TF>],
}

impl<'s, TF: TypeFamily> Subst<'s, TF> {
    pub fn apply<T: Fold<TF, TF>>(parameters: &[Parameter<TF>], value: &T) -> T::Result {
        value.fold_with(&mut Subst { parameters }, 0).unwrap()
    }
}

impl<TF: TypeFamily> QuantifiedTy<TF> {
    pub fn substitute(&self, parameters: &[Parameter<TF>]) -> Ty<TF> {
        assert_eq!(self.num_binders, parameters.len());
        Subst::apply(parameters, &self.ty)
    }
}

impl<'b, TF: TypeFamily> DefaultTypeFolder for Subst<'b, TF> {}

impl<'b, TF: TypeFamily> FreeVarFolder<TF> for Subst<'b, TF> {
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

impl<'b, TF: TypeFamily> DefaultPlaceholderFolder for Subst<'b, TF> {}

impl<'b, TF: TypeFamily> DefaultInferenceFolder for Subst<'b, TF> {}
