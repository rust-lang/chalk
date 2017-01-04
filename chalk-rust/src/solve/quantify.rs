use errors::*;
use fold::{Fold, Folder};
use ir;
use std::collections::HashMap;

pub struct Quantified<T> {
    pub value: T,
    pub binders: usize,
}

/// Given a value `value` with variables in it, returns a "Quantified"
/// version where the variables have been remapped to small integer
/// indices 0...N in order of appearance.
///
/// Example:
///
///    ?22: Foo<?23>
///
/// would be quantified to
///
///    Quantified { value: `?0: Foo<?1>`, binders: 2 }
pub fn quantify<T>(value: &T) -> Quantified<T::Result>
    where T: Fold
{
    let mut q = Quantifier { var_map: HashMap::new() };
    let r = value.fold_with(&mut q).unwrap();
    Quantified {
        value: r,
        binders: q.var_map.len(),
    }
}

struct Quantifier {
    var_map: HashMap<usize, ir::Ty>,
}

impl Folder for Quantifier {
    fn fold_var(&mut self, depth: usize) -> Result<ir::Ty> {
        let next_index = self.var_map.len();
        Ok(self.var_map
           .entry(depth)
           .or_insert(ir::Ty::Var(next_index))
           .clone())
    }
}
