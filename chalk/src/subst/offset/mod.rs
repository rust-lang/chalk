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
