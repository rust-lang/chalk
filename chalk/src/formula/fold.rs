use super::*;
use infer::InferenceVariable;

pub trait Fold {
    fn fold_with<F: Folder>(&self, folder: &mut F) -> Self;
}

pub trait Folder {
    fn in_binders<OP, R>(&mut self, num_binders: usize, op: OP) -> R
        where OP: FnOnce(&mut Self) -> R;
    fn replace_bound_variable(&mut self, from_leaf: &Leaf, v: BoundVariable) -> Leaf;
    fn replace_inference_variable(&mut self, from_leaf: &Leaf, v: InferenceVariable) -> Leaf;
}

impl<T: Fold> Fold for Vec<T> {
    fn fold_with<F: Folder>(&self, folder: &mut F) -> Self {
        self.iter().map(|e| e.fold_with(folder)).collect()
    }
}

impl<T: Fold> Fold for Option<T> {
    fn fold_with<F: Folder>(&self, folder: &mut F) -> Self {
        self.as_ref().map(|e| e.fold_with(folder))
    }
}

impl Fold for Application {
    fn fold_with<F: Folder>(&self, folder: &mut F) -> Self {
        Application {
            constant: self.constant,
            args: self.args.fold_with(folder),
        }
    }
}

impl Fold for Leaf {
    fn fold_with<F: Folder>(&self, folder: &mut F) -> Self {
        match self.kind {
            LeafKind::BoundVariable(v) => folder.replace_bound_variable(self, v),
            LeafKind::InferenceVariable(v) => folder.replace_inference_variable(self, v),
            LeafKind::Application(ref appl) => {
                Leaf::new(LeafData {
                    kind: LeafKind::Application(appl.fold_with(folder))
                })
            }
        }
    }
}

/// Macro to generate boiler-plate for pushing substitutions through
/// clause/goal-kinds.
macro_rules! fold {
    ($this:expr, $folder:expr, $Type:ident, $TypeData:ident, $TypeKind:ident {
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
                            $($arg_name.fold_with($folder)),*
                        )
                    })
                }
            )*
        }
    }
}

impl<L: Fold> Fold for Clause<L> {
    fn fold_with<F: Folder>(&self, folder: &mut F) -> Self {
        Clause::new((**self).fold_with(folder))
    }
}

impl<L: Fold> Fold for ClauseImplication<L> {
    fn fold_with<F: Folder>(&self, folder: &mut F) -> Self {
        ClauseImplication {
            condition: self.condition.fold_with(folder),
            consequence: self.consequence.fold_with(folder),
        }
    }
}

impl<L: Fold> Fold for Goal<L> {
    fn fold_with<F: Folder>(&self, folder: &mut F) -> Self {
        fold!(self, folder, Goal, GoalData, GoalKind {
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

impl<Q: Fold> Fold for Quantification<Q> {
    fn fold_with<F: Folder>(&self, folder: &mut F) -> Self {
        folder.in_binders(self.num_binders, |folder| {
            Quantification {
                num_binders: self.num_binders,
                formula: self.formula.fold_with(folder),
            }
        })
    }
}

///////////////////////////////////////////////////////////////////////////

/// Folder to "open up" a gap in the bound variable indices.  This is
/// useful when you are inserting a term underneath a binder and wish
/// to avoid accidental capture. For example, if I have some (not
/// necessarily closed) term X and I wish to transform it to
/// `forall(1) X` while avoiding capture, I would fold X with
/// `OpenUp::new(1)`.
pub struct OpenUp {
    gap: usize,
    skip: usize,
}

impl OpenUp {
    pub fn new(gap: usize) -> OpenUp {
        OpenUp {
            gap: gap,
            skip: 0,
        }
    }
}

impl Folder for OpenUp {
    fn in_binders<OP, R>(&mut self, num_binders: usize, op: OP) -> R
        where OP: FnOnce(&mut Self) -> R
    {
        op(&mut OpenUp {
            gap: self.gap,
            skip: self.skip + num_binders,
        })
    }

    fn replace_bound_variable(&mut self, from_leaf: &Leaf, v: BoundVariable) -> Leaf {
        if v.depth < self.skip {
            from_leaf.clone()
        } else {
            leaf!((bound self.gap + v.depth))
        }
    }

    fn replace_inference_variable(&mut self, from_leaf: &Leaf, _v: InferenceVariable) -> Leaf {
        from_leaf.clone()
    }
}
