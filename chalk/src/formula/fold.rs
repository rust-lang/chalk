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

impl Fold for Leaf {
    fn fold_with<F: Folder>(&self, folder: &mut F) -> Self {
        match self.kind {
            LeafKind::BoundVariable(v) => folder.replace_bound_variable(self, v),
            LeafKind::InferenceVariable(v) => folder.replace_inference_variable(self, v),
            LeafKind::Application(ref appl) => {
                Leaf::new(LeafData {
                    kind: LeafKind::Application(Application {
                        constant: appl.constant,
                        args: appl.args.fold_with(folder),
                    }),
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
        fold!(self, folder, Clause, ClauseData, ClauseKind {
            nullary { },
            Leaf(l),
            And(l, r),
            Implication(g, c),
            ForAll(q)
        })
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
