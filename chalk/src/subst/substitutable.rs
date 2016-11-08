use formula::*;
use super::{Subst, OffsetSubst};

pub trait Substitutable {
    fn subst(&self, subst: &Subst<Leaf>) -> Self;
}

impl<T: OffsetSubstitutable> Substitutable for T {
    fn subst(&self, subst: &Subst<Leaf>) -> Self {
        self.offset_subst(&OffsetSubst::new(subst))
    }
}

pub trait OffsetSubstitutable {
    fn offset_subst(&self, subst: &OffsetSubst<Leaf>) -> Self;
}

impl<T: OffsetSubstitutable> OffsetSubstitutable for Vec<T> {
    fn offset_subst(&self, subst: &OffsetSubst<Leaf>) -> Self {
        self.iter().map(|v| v.offset_subst(subst)).collect()
    }
}

impl OffsetSubstitutable for Leaf {
    fn offset_subst(&self, subst: &OffsetSubst<Leaf>) -> Self {
        match self.kind {
            LeafKind::BoundVariable(bv) => {
                match subst.get(bv.depth) {
                    None => self.clone(),
                    Some(leaf) => leaf.clone(),
                }
            }
            LeafKind::InferenceVariable(_) => self.clone(),
            LeafKind::Application(ref appl) => {
                Leaf::new(LeafData {
                    kind: LeafKind::Application(Application {
                        constant: appl.constant,
                        args: appl.args.offset_subst(subst),
                    })
                })
            }
        }
    }
}
