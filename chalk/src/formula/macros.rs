macro_rules! formula {
    ($f:tt => $g:tt) => {
        Formula::new(FormulaData {
            kind: FormulaKind::Implication()
        })
    }
}

macro_rules! leaf {
    (expr $expr:expr) => {
        $expr.clone()
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
        Constant::Program(intern($n))
    }
}

