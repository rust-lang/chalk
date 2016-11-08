macro_rules! formula {
    ($f:tt => $g:tt) => {
        Formula::new(FormulaData {
            kind: FormulaKind::Implication()
        })
    }
}

macro_rules! clause {
    (leaf $leaf:tt) => {
        Clause::new(ClauseData {
            kind: ClauseKind::Leaf(leaf!($leaf))
        })
    };
    (and $a:tt $b:tt) => {
        Clause::new(ClauseData {
            kind: ClauseKind::And(clause!($a), clause!($b))
        })
    };
    (implies $g:tt => $c:tt) => {
        Clause::new(ClauseData {
            kind: ClauseKind::Implication(goal!($g), clause!($c))
        })
    };
    (forall ($binders:expr) $c:tt) => {
        Clause::new(ClauseData {
            kind: ClauseKind::ForAll(Quantification {
                num_binders: $binders,
                formula: clause!($c)
            })
        })
    };
    (($($a:tt)*)) => {
        clause!($($a)*)
    }
}

macro_rules! leaf {
    (expr $expr:expr) => {
        $expr.clone()
    };
    (var $n:expr) => {
        Leaf::new(LeafData {
            kind: LeafKind::InferenceVariable($n)
        })
    };
    (bound $depth:expr) => {
        Leaf::new(LeafData {
            kind: LeafKind::BoundVariable(BoundVariable { depth: $depth })
        })
    };
    (apply $name:tt $($exprs:tt)*) => {
        Leaf::new(LeafData {
            kind: LeafKind::Application(Application {
                constant: constant!($name),
                args: vec![$(leaf!($exprs)),*],
            })
        })
    };
    (($($a:tt)*)) => {
        leaf!($($a)*)
    }
}

macro_rules! constant {
    (skol $n:tt) => {
        Constant::Skolemized(UniverseIndex { counter: $n })
    };
    (($($a:tt)*)) => {
        constant!($($a)*)
    };
    ($n:expr) => {
        Constant::Program(::lalrpop_intern::intern($n))
    }
}

