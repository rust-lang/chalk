use lalrpop_intern::intern;

use super::*;
use super::leaf::*;

macro_rules! leaf {
    (var $expr:expr) => {
        $expr.to_leaf()
    };
    (expr $expr:expr) => {
        $expr
    };
    (apply $name:tt/$universe:tt $($exprs:tt)*) => {
        InferenceLeaf::new(InferenceLeafData {
            kind: InferenceLeafKind::Application(InferenceApplication {
                constant: InferenceConstant {
                    operator: intern($name),
                    universe_index: UniverseIndex { counter: $universe },
                },
                args: vec![$(leaf!($exprs)),*],
            })
        })
    };
    (($($a:tt)*)) => {
        leaf!($($a)*)
    }
}

#[test]
fn infer() {
    let mut table = InferenceTable::new();
    let universe0 = UniverseIndex { counter: 0 };
    let a = table.new_variable(universe0);
    let b = table.new_variable(universe0);
    table.unify(&a.to_leaf(), &leaf!(apply "foo"/0 (var b))).unwrap();
    table.unify(&b.to_leaf(), &leaf!(apply "bar"/0)).unwrap();
    let c = table.normalize_deep(&a.to_leaf());
    assert_eq!(c, leaf!(apply "foo"/0 (apply "bar"/0)));
}

#[test]
fn universe_error() {
    // exists(A -> forall(X -> A = X)) ---> error
    let mut table = InferenceTable::new();
    let universe0 = UniverseIndex { counter: 0 };
    let a = table.new_variable(universe0);
    table.unify(&a.to_leaf(), &leaf!(apply "foo"/1)).unwrap_err();
}

#[test]
fn cycle_error() {
    // exists(A -> A = foo A) ---> error
    let mut table = InferenceTable::new();
    let universe0 = UniverseIndex { counter: 0 };
    let a = table.new_variable(universe0);
    table.unify(&a.to_leaf(), &leaf!(apply "foo"/1 (var a))).unwrap_err();
}

#[test]
fn universe_error_indirect_1() {
    // exists(A -> forall(X -> exists(B -> B = X, A = B))) ---> error
    let mut table = InferenceTable::new();
    let universe0 = UniverseIndex { counter: 0 };
    let universe1 = UniverseIndex { counter: 1 };
    let a = table.new_variable(universe0);
    let b = table.new_variable(universe1);
    table.unify(&b.to_leaf(), &leaf!(apply "foo"/1)).unwrap();
    table.unify(&a.to_leaf(), &b.to_leaf()).unwrap_err();
}

#[test]
fn universe_error_indirect_2() {
    // exists(A -> forall(X -> exists(B -> B = A, B = X))) ---> error
    let mut table = InferenceTable::new();
    let universe0 = UniverseIndex { counter: 0 };
    let universe1 = UniverseIndex { counter: 1 };
    let a = table.new_variable(universe0);
    let b = table.new_variable(universe1);
    table.unify(&a.to_leaf(), &b.to_leaf()).unwrap();
    table.unify(&b.to_leaf(), &leaf!(apply "foo"/1)).unwrap_err();
}

