use errors::*;
use fold::{Fold, Folder};
use ir::*;

use super::{InferenceTable, InferenceVariable};
use super::var::InferenceValue;

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
    ///    Quantified { value: `?0: Foo<?1>`, binders: [ui(?22), ui(?23)] }
    ///
    /// where `ui(?22)` and `ui(?23)` are the universe indices of
    /// `?22` and `?23` respectively.
    pub fn quantify<T>(&mut self, value: &T) -> Quantified<T::Result>
        where T: Fold
    {
        let mut q = Quantifier { table: self, free_vars: Vec::new() };
        let r = value.fold_with(&mut q).unwrap();
        Quantified {
            value: r,
            binders: q.into_binders(),
        }
    }
}

struct Quantifier<'q> {
    table: &'q mut InferenceTable,
    free_vars: Vec<InferenceVariable>,
}

impl<'q> Quantifier<'q> {
    fn into_binders(self) -> Vec<UniverseIndex> {
        let Quantifier { table, free_vars } = self;
        free_vars.iter()
                 .map(|&v| {
                     debug_assert!(table.unify.find(v) == v);
                     match table.unify.probe_value(v) {
                         InferenceValue::Unbound(ui) => ui,
                         InferenceValue::Bound(_) => panic!("free var now bound"),
                     }
                 })
                 .collect()
    }
}

impl<'q> Folder for Quantifier<'q> {
    fn fold_var(&mut self, depth: usize) -> Result<Ty> {
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
                let position = match self.free_vars.iter().position(|&v| v == var) {
                    Some(i) => i,
                    None => {
                        let next_index = self.free_vars.len();
                        self.free_vars.push(root_var);
                        next_index
                    }
                };
                Ok(InferenceVariable::from_depth(position).to_ty())
            }
        }
    }
}
