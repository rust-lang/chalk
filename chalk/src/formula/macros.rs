macro_rules! formula {
    ($f:tt => $g:tt) => {
        Formula::new(FormulaData {
            kind: FormulaKind::Implication()
        })
    }
}

macro_rules! clause {
    (forall ($binders:expr) $($c:tt)*) => {
        Clause::new(Quantification {
                num_binders: $binders,
                formula: clause_formula!($($c)*)
        })
    };
    (($($a:tt)*)) => {
        clause!($($a)*)
    };
    ($($a:tt)*) => {
        clause!(forall(0) $($a)*)
    };
}

macro_rules! clause_formula {
    (expr $leaf:expr) => {
        ClauseImplication {
            condition: None,
            consequence: $leaf
        }
    };
    (leaf $leaf:tt) => {
        ClauseImplication {
            condition: None,
            consequence: leaf!($leaf)
        }
    };
    (implies $g:tt => $c:tt) => {
        ClauseImplication {
            condition: Some(goal!($g)),
            consequence: leaf!($c)
        }
    };
    (($($a:tt)*)) => {
        clause_formula!($($a)*)
    };
}

macro_rules! goal {
    (expr $expr:expr) => {
        $expr.clone()
    };
    (leaf $leaf:tt) => {
        Goal::new(GoalData {
            kind: GoalKind::Leaf(leaf!($leaf))
        })
    };
    (true) => {
        Goal::new(GoalData {
            kind: GoalKind::True
        })
    };
    (and $a:tt $b:tt) => {
        Goal::new(GoalData {
            kind: GoalKind::And(goal!($a), goal!($b))
        })
    };
    (or $a:tt $b:tt) => {
        Goal::new(GoalData {
            kind: GoalKind::Or(goal!($a), goal!($b))
        })
    };
    (implies $($g:tt),* => $c:tt) => {
        Goal::new(GoalData {
            kind: GoalKind::Implication(vec![$(clause!($g)),*], goal!($c))
        })
    };
    (forall ($binders:expr) $c:tt) => {
        Goal::new(GoalData {
            kind: GoalKind::ForAll(Quantification {
                num_binders: $binders,
                formula: goal!($c)
            })
        })
    };
    (exists ($binders:expr) $c:tt) => {
        Goal::new(GoalData {
            kind: GoalKind::Exists(Quantification {
                num_binders: $binders,
                formula: goal!($c)
            })
        })
    };
    (($($a:tt)*)) => {
        goal!($($a)*)
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

