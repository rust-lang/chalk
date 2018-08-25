use fold::shift::Shift;
use ir::*;

use super::*;

crate struct Subst<'s> {
    /// Values to substitute. A reference to a free variable with
    /// index `i` will be mapped to `parameters[i]` -- if `i >
    /// parameters.len()`, then we will leave the variable untouched.
    parameters: &'s [Parameter],
}

impl<'s> Subst<'s> {
    crate fn apply<T: Fold>(parameters: &[Parameter], value: &T) -> T::Result {
        value.fold_with(&mut Subst { parameters }, 0).unwrap()
    }
}

impl QuantifiedTy {
    crate fn substitute(&self, parameters: &[Parameter]) -> Ty {
        assert_eq!(self.num_binders, parameters.len());
        Subst::apply(parameters, &self.ty)
    }
}

impl<'b> DefaultTypeFolder for Subst<'b> {}

impl<'b> ExistentialFolder for Subst<'b> {
    fn fold_free_existential_ty(&mut self, depth: usize, binders: usize) -> Fallible<Ty> {
        if depth >= self.parameters.len() {
            Ok(Ty::Var(depth - self.parameters.len() + binders))
        } else {
            match self.parameters[depth] {
                ParameterKind::Ty(ref t) => Ok(t.shifted_in(binders)),
                _ => panic!("mismatched kinds in substitution"),
            }
        }
    }

    fn fold_free_existential_lifetime(
        &mut self,
        depth: usize,
        binders: usize,
    ) -> Fallible<Lifetime> {
        if depth >= self.parameters.len() {
            Ok(Lifetime::Var(depth - self.parameters.len() + binders))
        } else {
            match self.parameters[depth] {
                ParameterKind::Lifetime(ref l) => Ok(l.shifted_in(binders)),
                _ => panic!("mismatched kinds in substitution"),
            }
        }
    }
}

impl<'b> IdentityUniversalFolder for Subst<'b> {}
