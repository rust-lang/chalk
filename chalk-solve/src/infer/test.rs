#![cfg(test)]

use super::unify::RelationResult;
use super::*;
use chalk_integration::interner::ChalkIr;
use chalk_integration::{arg, lifetime, ty};

// We just use a vec of 20 `Invariant`, since this is zipped and no substs are
// longer than this
#[derive(Debug)]
struct TestDatabase;
impl UnificationDatabase<ChalkIr> for TestDatabase {
    fn fn_def_variance(&self, _fn_def_id: FnDefId<ChalkIr>) -> Variances<ChalkIr> {
        Variances::from_iter(ChalkIr, [Variance::Invariant; 20].iter().copied())
    }

    fn adt_variance(&self, _adt_id: AdtId<ChalkIr>) -> Variances<ChalkIr> {
        Variances::from_iter(ChalkIr, [Variance::Invariant; 20].iter().copied())
    }
}

#[test]
fn universe_error() {
    // exists(A -> forall(X -> A = X)) ---> error
    let interner = ChalkIr;
    let mut table: InferenceTable<ChalkIr> = InferenceTable::new();
    let environment0 = Environment::new(interner);
    let a = table.new_variable(U0).to_ty(interner);
    table
        .relate(
            interner,
            &TestDatabase,
            &environment0,
            Variance::Invariant,
            &a,
            &ty!(placeholder 1),
        )
        .unwrap_err();
}

#[test]
fn cycle_error() {
    // exists(A -> A = foo A) ---> error
    let interner = ChalkIr;
    let mut table: InferenceTable<ChalkIr> = InferenceTable::new();
    let environment0 = Environment::new(interner);
    let a = table.new_variable(U0).to_ty(interner);
    table
        .relate(
            interner,
            &TestDatabase,
            &environment0,
            Variance::Invariant,
            &a,
            &ty!(apply (item 0) (expr a)),
        )
        .unwrap_err();

    // exists(A -> A = for<'a> A)
    table
        .relate(
            interner,
            &TestDatabase,
            &environment0,
            Variance::Invariant,
            &a,
            &ty!(function 1 (infer 0)),
        )
        .unwrap_err();
}

#[test]
fn cycle_indirect() {
    // exists(A -> A = foo B, A = B) ---> error
    let interner = ChalkIr;
    let mut table: InferenceTable<ChalkIr> = InferenceTable::new();
    let environment0 = Environment::new(interner);
    let a = table.new_variable(U0).to_ty(interner);
    let b = table.new_variable(U0).to_ty(interner);
    table
        .relate(
            interner,
            &TestDatabase,
            &environment0,
            Variance::Invariant,
            &a,
            &ty!(apply (item 0) (expr b)),
        )
        .unwrap();
    table
        .relate(
            interner,
            &TestDatabase,
            &environment0,
            Variance::Invariant,
            &a,
            &b,
        )
        .unwrap_err();
}

#[test]
fn universe_error_indirect_1() {
    // exists(A -> forall(X -> exists(B -> B = X, A = B))) ---> error
    let interner = ChalkIr;
    let mut table: InferenceTable<ChalkIr> = InferenceTable::new();
    let environment0 = Environment::new(interner);
    let a = table.new_variable(U0).to_ty(interner);
    let b = table.new_variable(U1).to_ty(interner);
    table
        .relate(
            interner,
            &TestDatabase,
            &environment0,
            Variance::Invariant,
            &b,
            &ty!(placeholder 1),
        )
        .unwrap();
    table
        .relate(
            interner,
            &TestDatabase,
            &environment0,
            Variance::Invariant,
            &a,
            &b,
        )
        .unwrap_err();
}

#[test]
fn universe_error_indirect_2() {
    // exists(A -> forall(X -> exists(B -> B = A, B = X))) ---> error
    let interner = ChalkIr;
    let mut table: InferenceTable<ChalkIr> = InferenceTable::new();
    let environment0 = Environment::new(interner);
    let a = table.new_variable(U0).to_ty(interner);
    let b = table.new_variable(U1).to_ty(interner);
    table
        .relate(
            interner,
            &TestDatabase,
            &environment0,
            Variance::Invariant,
            &a,
            &b,
        )
        .unwrap();
    table
        .relate(
            interner,
            &TestDatabase,
            &environment0,
            Variance::Invariant,
            &b,
            &ty!(placeholder 1),
        )
        .unwrap_err();
}

#[test]
fn universe_promote() {
    // exists(A -> forall(X -> exists(B -> A = foo(B), A = foo(i32)))) ---> OK
    let interner = ChalkIr;
    let mut table: InferenceTable<ChalkIr> = InferenceTable::new();
    let environment0 = Environment::new(interner);
    let a = table.new_variable(U0).to_ty(interner);
    let b = table.new_variable(U1).to_ty(interner);
    table
        .relate(
            interner,
            &TestDatabase,
            &environment0,
            Variance::Invariant,
            &a,
            &ty!(apply (item 0) (expr b)),
        )
        .unwrap();
    table
        .relate(
            interner,
            &TestDatabase,
            &environment0,
            Variance::Invariant,
            &a,
            &ty!(apply (item 0) (apply (item 1))),
        )
        .unwrap();
}

#[test]
fn universe_promote_bad() {
    // exists(A -> forall(X -> exists(B -> A = foo(B), B = X))) ---> error
    let interner = ChalkIr;
    let mut table: InferenceTable<ChalkIr> = InferenceTable::new();
    let environment0 = Environment::new(interner);
    let a = table.new_variable(U0).to_ty(interner);
    let b = table.new_variable(U1).to_ty(interner);
    table
        .relate(
            interner,
            &TestDatabase,
            &environment0,
            Variance::Invariant,
            &a,
            &ty!(apply (item 0) (expr b)),
        )
        .unwrap();
    table
        .relate(
            interner,
            &TestDatabase,
            &environment0,
            Variance::Invariant,
            &b,
            &ty!(placeholder 1),
        )
        .unwrap_err();
}

#[test]
fn projection_eq() {
    // exists(A -> A = Item0<<A as Item1>::foo>)
    //                       ^^^^^^^^^^^^ Can A repeat here? For now,
    //                       we say no, but it's an interesting question.
    let interner = ChalkIr;
    let mut table: InferenceTable<ChalkIr> = InferenceTable::new();
    let environment0 = Environment::new(interner);
    let a = table.new_variable(U0).to_ty(interner);

    // expect an error ("cycle during unification")
    table
        .relate(
            interner,
            &TestDatabase,
            &environment0,
            Variance::Invariant,
            &a,
            &ty!(apply (item 0) (projection (item 1) (expr a))),
        )
        .unwrap_err();
}

const U0: UniverseIndex = UniverseIndex { counter: 0 };
const U1: UniverseIndex = UniverseIndex { counter: 1 };
const U2: UniverseIndex = UniverseIndex { counter: 2 };

fn make_table() -> InferenceTable<ChalkIr> {
    let mut table: InferenceTable<ChalkIr> = InferenceTable::new();
    let _ = table.new_universe(); // U1
    let _ = table.new_universe(); // U2
    table
}

#[test]
fn quantify_simple() {
    let interner = ChalkIr;
    let mut table = make_table();
    let _ = table.new_variable(U0);
    let _ = table.new_variable(U1);
    let _ = table.new_variable(U2);

    assert_eq!(
        table
            .canonicalize(interner, ty!(apply (item 0) (infer 2) (infer 1) (infer 0)))
            .quantified,
        Canonical {
            value: ty!(apply (item 0) (bound 0) (bound 1) (bound 2)),
            binders: CanonicalVarKinds::from_iter(
                interner,
                vec![
                    CanonicalVarKind::new(VariableKind::Ty(TyVariableKind::General), U2),
                    CanonicalVarKind::new(VariableKind::Ty(TyVariableKind::General), U1),
                    CanonicalVarKind::new(VariableKind::Ty(TyVariableKind::General), U0),
                ]
            ),
        }
    );
}

#[test]
fn quantify_bound() {
    let interner = ChalkIr;
    let mut table = make_table();
    let environment0 = Environment::new(interner);

    let v0 = table.new_variable(U0).to_ty(interner);
    let v1 = table.new_variable(U1).to_ty(interner);
    let v2a = table.new_variable(U2).to_ty(interner);
    let v2b = table.new_variable(U2).to_ty(interner);

    table
        .relate(
            interner,
            &TestDatabase,
            &environment0,
            Variance::Invariant,
            &v2b,
            &ty!(apply (item 1) (expr v1) (expr v0)),
        )
        .unwrap();

    assert_eq!(
        table
            .canonicalize(
                interner,
                ty!(apply (item 0) (expr v2b) (expr v2a) (expr v1) (expr v0))
            )
            .quantified,
        Canonical {
            value: ty!(apply (item 0) (apply (item 1) (bound 0) (bound 1)) (bound 2) (bound 0) (bound 1)),
            binders: CanonicalVarKinds::from_iter(
                interner,
                vec![
                    CanonicalVarKind::new(VariableKind::Ty(TyVariableKind::General), U1),
                    CanonicalVarKind::new(VariableKind::Ty(TyVariableKind::General), U0),
                    CanonicalVarKind::new(VariableKind::Ty(TyVariableKind::General), U2),
                ]
            ),
        }
    );
}

#[test]
fn quantify_ty_under_binder() {
    let interner = ChalkIr;
    let mut table = make_table();
    let v0 = table.new_variable(U0);
    let v1 = table.new_variable(U0);
    let _r2 = table.new_variable(U0);

    // Unify v0 and v1.
    let environment0 = Environment::new(interner);
    table
        .relate(
            interner,
            &TestDatabase,
            &environment0,
            Variance::Invariant,
            &v0.to_ty(interner),
            &v1.to_ty(interner),
        )
        .unwrap();

    // Here: the `function` introduces 3 binders, so in the result,
    // `(bound 3)` references the first canonicalized inference
    // variable. -- note that `infer 0` and `infer 1` have been
    // unified above, as well.
    assert_eq!(
        table
            .canonicalize(
                interner,
                ty!(function 3 (apply (item 0) (bound 1) (infer 0) (infer 1) (lifetime (infer 2))))
            )
            .quantified,
        Canonical {
            value: ty!(function 3 (apply (item 0) (bound 1) (bound 1 0) (bound 1 0) (lifetime (bound 1 1)))),
            binders: CanonicalVarKinds::from_iter(
                interner,
                vec![
                    CanonicalVarKind::new(VariableKind::Ty(TyVariableKind::General), U0),
                    CanonicalVarKind::new(VariableKind::Lifetime, U0)
                ]
            ),
        }
    );
}

#[test]
fn lifetime_constraint_indirect() {
    let interner = ChalkIr;
    let mut table: InferenceTable<ChalkIr> = InferenceTable::new();
    let _ = table.new_universe(); // U1

    let _t_0 = table.new_variable(U0);
    let _l_1 = table.new_variable(U1);

    let environment0 = Environment::new(interner);

    // Here, we unify '?1 (the lifetime variable in universe 1) with
    // '!1.
    let t_a = ty!(apply (item 0) (lifetime (placeholder 1)));
    let t_b = ty!(apply (item 0) (lifetime (infer 1)));
    let RelationResult { goals } = table
        .relate(
            interner,
            &TestDatabase,
            &environment0,
            Variance::Invariant,
            &t_a,
            &t_b,
        )
        .unwrap();
    assert!(goals.is_empty());

    // Here, we try to unify `?0` (the type variable in universe 0)
    // with something that involves `'?1`. Since `'?1` has been
    // unified with `'!1`, and `'!1` is not visible from universe 0,
    // we will replace `'!1` with a new variable `'?2` and introduce a
    // (likely unsatisfiable) constraint relating them.
    let t_c = ty!(infer 0);
    let RelationResult { goals } = table
        .relate(
            interner,
            &TestDatabase,
            &environment0,
            Variance::Invariant,
            &t_c,
            &t_b,
        )
        .unwrap();
    assert_eq!(goals.len(), 2);
    assert_eq!(
        format!("{:?}", goals[0]),
        "InEnvironment { environment: Env([]), goal: \'?2: \'!1_0 }",
    );
    assert_eq!(
        format!("{:?}", goals[1]),
        "InEnvironment { environment: Env([]), goal: \'!1_0: \'?2 }",
    );
}
