use errors::*;
use ir::*;
use fold::*;
use std::collections::HashMap;

use super::*;

impl InferenceTable {
    // Create a instance of `arg` where each variable is replaced with
    // a fresh inference variable.
    pub fn instantiate<T: Fold>(&mut self, universe: UniverseIndex, arg: &T) -> T::Result {
        let mut instantiator = Instantiator {
            table: self,
            universe: universe,
            vars: HashMap::new(),
        };
        arg.fold_with(&mut instantiator).expect("")
    }
}

struct Instantiator<'t> {
    table: &'t mut InferenceTable,
    universe: UniverseIndex,
    vars: HashMap<usize, InferenceVariable>,
}

impl<'t> Folder for Instantiator<'t> {
    fn fold_var(&mut self, depth: usize) -> Result<Ty> {
        let table = &mut self.table;
        let universe = self.universe;
        Ok(self.vars
            .entry(depth)
            .or_insert_with(|| table.new_variable(universe))
            .to_ty())
    }
}
