use ir::*;

use super::*;

pub struct Subst<'s> {
    /// Values to substitute. A reference to a free variable with
    /// index `i` will be mapped to `parameters[i]` -- if `i >
    /// parameters.len()`, then we will leave the variable untouched.
    parameters: &'s [Parameter]
}

impl<'s> Subst<'s> {
    pub fn new(parameters: &'s [Parameter]) -> Subst<'s> {
        Subst { parameters }
    }

    pub fn apply<T: Fold>(parameters: &[Parameter], value: &T) -> T::Result {
        value.fold_with(&mut Subst { parameters }, 0).unwrap()
    }
}

impl QuantifiedTy {
    pub fn subst(&self, parameters: &[Parameter]) -> Ty {
        assert_eq!(self.num_binders, parameters.len());
        self.ty.subst(parameters)
    }
}

macro_rules! subst_method {
    ($t:ty) => {
        impl $t {
            pub fn subst(&self, parameters: &[Parameter]) -> Self {
                Subst::apply(parameters, self)
            }
        }
    }
}

subst_method!(Goal);
subst_method!(Ty);

impl<'b> FolderVar for Subst<'b> {
    fn fold_free_var(&mut self, depth: usize, binders: usize) -> Result<Ty> {
        if depth >= self.parameters.len() {
            Ok(Ty::Var(depth - self.parameters.len() + binders))
        } else {
            match self.parameters[depth] {
                ParameterKind::Ty(ref t) => Ok(t.up_shift(binders)),
                _ => panic!("mismatched kinds in substitution"),
            }
        }
    }

    fn fold_free_lifetime_var(&mut self, depth: usize, binders: usize) -> Result<Lifetime> {
        if depth >= self.parameters.len() {
            Ok(Lifetime::Var(depth - self.parameters.len() + binders))
        } else {
            match self.parameters[depth] {
                ParameterKind::Lifetime(ref l) => Ok(l.up_shift(binders)),
                _ => panic!("mismatched kinds in substitution"),
            }
        }
    }
}
