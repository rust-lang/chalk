use fold::*;
use std::fmt::Debug;

use super::*;

impl InferenceTable {
    /// Create a instance of `arg` where each variable is replaced with
    /// a fresh inference variable of suitable kind.
    pub fn instantiate<U, T>(&mut self, universes: U, arg: &T) -> T::Result
        where T: Fold + Debug,
              U: IntoIterator<Item = ParameterKind<UniverseIndex>>
    {
        debug!("instantiate(arg={:?})", arg);
        let vars: Vec<_> = universes.into_iter()
            .map(|u| self.new_parameter_variable(u))
            .collect();
        debug!("instantiate: vars={:?}", vars);
        let mut instantiator = Instantiator { vars };
        arg.fold_with(&mut instantiator, 0).expect("")
    }
}

struct Instantiator {
    vars: Vec<ParameterInferenceVariable>,
}

/// Folder: when we encounter a free variable (of any kind) with index
/// `i`, we want to map anything in the first N binders to
/// `self.vars[i]`. Everything else stays intact, but we have to
/// subtract `self.vars.len()` to account for the binders we are
/// instantiating.
impl Folder for Instantiator {
    fn fold_free_var(&mut self, depth: usize, binders: usize) -> Result<Ty> {
        if depth < self.vars.len() {
            Ok(self.vars[depth].as_ref().ty().unwrap().to_ty().up_shift(binders))
        } else {
            Ok(Ty::Var(depth + binders - self.vars.len())) // see comment above
        }
    }

    fn fold_free_lifetime_var(&mut self, depth: usize, binders: usize) -> Result<Lifetime> {
        if depth < self.vars.len() {
            Ok(self.vars[depth].as_ref().lifetime().unwrap().to_lifetime().up_shift(binders))
        } else {
            Ok(Lifetime::Var(depth + binders - self.vars.len())) // see comment above
        }
    }
}
