use errors::*;
use fold::{Fold, Folder};
use ir;
use std::collections::HashMap;

use super::{InferenceTable, InferenceVariable};

impl InferenceTable {
    /// Given a value `value` with variables in it, replaces those
    /// variables with their instantiated values; any variables not
    /// yet instantiated are replaces with a small integer index 0..N
    /// in order of appearance. The result is a canonicalized
    /// representation of `value`.
    ///
    /// Example:
    ///
    ///    ?22: Foo<?23>
    ///
    /// would be quantified to
    ///
    ///    Quantified { value: `?0: Foo<?1>`, binders: 2 }
    pub fn quantify<T>(&mut self, value: &T) -> ir::Quantified<T::Result>
        where T: Fold
    {
        let mut q = Quantifier { table: self, var_map: HashMap::new() };
        let r = value.fold_with(&mut q).unwrap();
        ir::Quantified {
            value: r,
            binders: q.var_map.len(),
        }
    }
}

struct Quantifier<'q> {
    table: &'q mut InferenceTable,
    var_map: HashMap<InferenceVariable, InferenceVariable>,
}

impl<'q> Folder for Quantifier<'q> {
    fn fold_var(&mut self, depth: usize) -> Result<ir::Ty> {
        let var = InferenceVariable::from_depth(depth);
        match self.table.probe_var(var) {
            Some(ty) => {
                // If this variable is bound, canonicalize it to its
                // bound value.
                (*ty).fold_with(self)
            }
            None => {
                // If this variable is not yet bound, find its
                // canonical index `root_var` in the union-find table,
                // and then map `root_var` to a fresh index that is
                // unique to this quantification.
                let root_var = self.table.unify.find(var);
                let next_index = self.var_map.len();
                Ok(self.var_map
                   .entry(root_var)
                   .or_insert(InferenceVariable::from_depth(next_index))
                   .to_ty())
            }
        }
    }
}
