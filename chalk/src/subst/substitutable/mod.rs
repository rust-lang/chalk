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
                    }),
                })
            }
        }
    }
}

macro_rules! fold {
    ($this:expr, $subst:expr, $Type:ident, $TypeData:ident, $TypeKind:ident {
        nullary { $($NullaryVariantName:ident),* },
        $($VariantName:ident($($arg_name:ident),*)),*
    }) => {
        match $this.kind {
            $(
                $TypeKind::$NullaryVariantName => $this.clone(),
            )*
            $(
                $TypeKind::$VariantName(
                    $(ref $arg_name),*
                ) => {
                    $Type::new($TypeData {
                        kind: $TypeKind::$VariantName(
                            $($arg_name.offset_subst($subst)),*
                        )
                    })
                }
            )*
        }
    }
}

impl<L: OffsetSubstitutable> OffsetSubstitutable for Clause<L> {
    fn offset_subst(&self, subst: &OffsetSubst<Leaf>) -> Self {
        fold!(self, subst, Clause, ClauseData, ClauseKind {
            nullary { },
            Leaf(l),
            And(l, r),
            Implication(g, c),
            ForAll(q)
        })
    }
}

impl<L: OffsetSubstitutable> OffsetSubstitutable for Goal<L> {
    fn offset_subst(&self, subst: &OffsetSubst<Leaf>) -> Self {
        fold!(self, subst, Goal, GoalData, GoalKind {
            nullary { True },
            Leaf(l),
            And(l, r),
            Or(l, r),
            Exists(q),
            Implication(c, g),
            ForAll(q)
        })
    }
}

impl<F: OffsetSubstitutable> OffsetSubstitutable for Quantification<F> {
    fn offset_subst(&self, subst: &OffsetSubst<Leaf>) -> Self {
        // push N offsets
        let subst = (0..self.num_binders).fold(subst.clone(), |s, _| s.push_offset());
        Quantification {
            num_binders: self.num_binders,
            formula: self.formula.offset_subst(&subst),
        }
    }
}
