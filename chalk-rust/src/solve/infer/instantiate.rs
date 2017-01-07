use errors::*;
use ir::*;
use fold::*;

use super::*;

impl InferenceTable {
    // Create a instance of `arg` where each variable is replaced with
    // a fresh inference variable.
    pub fn instantiate<U, T>(&mut self, universes: U, arg: &T) -> T::Result
        where T: Fold,
              U: IntoIterator<Item = UniverseIndex>
    {
        let vars: Vec<_> = universes.into_iter()
            .map(|u| self.new_variable(u))
            .collect();
        let mut instantiator = Instantiator { vars: vars };
        arg.fold_with(&mut instantiator).expect("")
    }
}

struct Instantiator {
    vars: Vec<InferenceVariable>,
}

impl Folder for Instantiator {
    fn fold_var(&mut self, depth: usize) -> Result<Ty> {
        Ok(self.vars[depth].to_ty())
    }
}
