use fold::*;
use std::fmt::Debug;

use super::*;

impl InferenceTable {
    // Create a instance of `arg` where each variable is replaced with
    // a fresh inference variable.
    pub fn instantiate<U, T>(&mut self, universes: U, arg: &T) -> T::Result
        where T: Fold + Debug,
              U: IntoIterator<Item = ParameterKind<UniverseIndex>>
    {
        debug!("instantiate(arg={:?})", arg);
        let vars: Vec<_> = universes.into_iter()
            .map(|u| self.new_parameter_variable(u))
            .collect();
        debug!("instantiate: vars={:?}", vars);
        let mut instantiator = Instantiator { vars: vars };
        arg.fold_with(&mut instantiator, 0).expect("")
    }
}

struct Instantiator {
    vars: Vec<ParameterInferenceVariable>,
}

impl Folder for Instantiator {
    fn fold_free_var(&mut self, depth: usize, binders: usize) -> Result<Ty> {
        Ok(self.vars[depth].as_ref().ty().unwrap().to_ty().up_shift(binders))
    }

    fn fold_free_lifetime_var(&mut self, depth: usize, binders: usize) -> Result<Lifetime> {
        Ok(self.vars[depth].as_ref().lifetime().unwrap().to_lifetime().up_shift(binders))
    }
}
