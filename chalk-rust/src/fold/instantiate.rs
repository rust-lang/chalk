use ir::*;

use super::*;

pub struct Subst<'s> {
    /// A stack of values, where the top of the stack is the most
    /// recently pushed. Therefore, deBruijn depth 0 maps to the last
    /// item in the slice.
    parameters: &'s [Parameter]
}

impl<'s> Subst<'s> {
    fn apply<T: Fold>(parameters: &[Parameter], value: &T) -> T::Result {
        value.fold_with(&mut Subst { parameters }, 0).unwrap()
    }
}

impl QuantifiedTy {
    pub fn instantiate(&self, parameters: &[Parameter]) -> Ty {
        assert_eq!(self.num_binders, parameters.len());
        self.ty.fold_with(&mut Subst { parameters }, 0).unwrap()
    }
}

impl<'b> Folder for Subst<'b> {
    fn fold_free_var(&mut self, depth: usize, binders: usize) -> Result<Ty> {
        if depth > self.parameters.len() {
            Ok(Ty::Var(depth + binders))
        } else {
            match self.parameters[self.parameters.len() - 1 - depth] {
                ParameterKind::Ty(ref t) => Ok(t.up_shift(binders)),
                ParameterKind::Lifetime(_) => panic!("mismatched kinds in substitution"),
            }
        }
    }

    fn fold_free_lifetime_var(&mut self, depth: usize, binders: usize) -> Result<Lifetime> {
        if depth > self.parameters.len() {
            Ok(Lifetime::Var(depth + binders))
        } else {
            match self.parameters[self.parameters.len() - 1 - depth] {
                ParameterKind::Lifetime(ref l) => Ok(l.up_shift(binders)),
                ParameterKind::Ty(_) => panic!("mismatched kinds in substitution"),
            }
        }
    }
}
