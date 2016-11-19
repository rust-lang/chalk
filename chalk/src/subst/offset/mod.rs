use infer::InferenceVariable;
use formula::{BoundVariable, Leaf, Folder};
use std::fmt::Debug;

use super::Subst;

#[cfg(test)]
mod test;

/// When substituting across terms that may contain binders,
/// we often
#[derive(Debug)]
pub struct OffsetSubst<L: Debug> {
    offset: usize,
    subst: Subst<L>,
}

impl<L: Debug> OffsetSubst<L> {
    pub fn new(subst: &Subst<L>) -> Self {
        OffsetSubst {
            offset: 0,
            subst: subst.clone(),
        }
    }

    pub fn push_offset(&self) -> Self {
        OffsetSubst {
            offset: self.offset + 1,
            subst: self.subst.clone(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&L> {
        if index < self.offset {
            None
        } else {
            Some(&self.subst[index - self.offset])
        }
    }
}

impl<L: Debug> Clone for OffsetSubst<L> {
    fn clone(&self) -> Self {
        OffsetSubst {
            offset: self.offset,
            subst: self.subst.clone(),
        }
    }
}

impl Folder for OffsetSubst<Leaf> {
    fn in_binders<OP, R>(&mut self, num_binders: usize, op: OP) -> R
        where OP: FnOnce(&mut Self) -> R
    {
        let mut subst = (0..num_binders).fold(self.clone(), |s, _| s.push_offset());
        op(&mut subst)
    }

    fn replace_bound_variable(&mut self, from_leaf: &Leaf, v: BoundVariable) -> Leaf {
        match self.get(v.depth) {
            None => from_leaf.clone(),
            Some(l) => l.clone(),
        }
    }

    fn replace_inference_variable(&mut self, from_leaf: &Leaf, _: InferenceVariable) -> Leaf {
        from_leaf.clone()
    }
}
