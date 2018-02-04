use fallible::*;
use fold::*;
use super::*;
use super::unify::UnificationResult;

impl InferenceTable {
    pub fn normalize<T>(&mut self, value: &T) -> T::Result
    where
        T: Fold,
    {
        value.fold_with(&mut Normalizer { table: self }, 0).unwrap()
    }
}

struct Normalizer<'a> {
    table: &'a mut InferenceTable,
}

impl<'q> DefaultTypeFolder for Normalizer<'q> {}

impl<'q> ExistentialFolder for Normalizer<'q> {
    fn fold_free_existential_ty(&mut self, depth: usize, binders: usize) -> Fallible<Ty> {
        assert_eq!(binders, 0);
        let var = InferenceVariable::from_depth(depth);
        match self.table.probe_ty_var(var) {
            Some(ty) => ty.fold_with(self, 0),
            None => Ok(var.to_ty()),
        }
    }

    fn fold_free_existential_lifetime(
        &mut self,
        depth: usize,
        binders: usize,
    ) -> Fallible<Lifetime> {
        assert_eq!(binders, 0);
        Ok(InferenceVariable::from_depth(depth).to_lifetime())
    }
}

impl<'q> IdentityUniversalFolder for Normalizer<'q> {}

#[test]
fn infer() {
    let mut table = InferenceTable::new();
    let environment0 = Environment::new();
    let a = table.new_variable(U0).to_ty();
    let b = table.new_variable(U0).to_ty();
    table
        .unify(&environment0, &a, &ty!(apply (item 0) (expr b)))
        .unwrap();
    assert_eq!(table.normalize(&a), ty!(apply (item 0) (expr b)));
    table
        .unify(&environment0, &b, &ty!(apply (item 1)))
        .unwrap();
    assert_eq!(table.normalize(&a), ty!(apply (item 0) (apply (item 1))));
}

#[test]
fn universe_error() {
    // exists(A -> forall(X -> A = X)) ---> error
    let mut table = InferenceTable::new();
    let environment0 = Environment::new();
    let a = table.new_variable(U0).to_ty();
    table
        .unify(&environment0, &a, &ty!(apply (skol 1)))
        .unwrap_err();
}

#[test]
fn cycle_error() {
    // exists(A -> A = foo A) ---> error
    let mut table = InferenceTable::new();
    let environment0 = Environment::new();
    let a = table.new_variable(U0).to_ty();
    table
        .unify(&environment0, &a, &ty!(apply (item 0) (expr a)))
        .unwrap_err();

    // exists(A -> A = for<'a> A)
    table
        .unify(&environment0, &a, &ty!(for_all 1 (var 1)))
        .unwrap_err();
}

#[test]
fn cycle_indirect() {
    // exists(A -> A = foo B, A = B) ---> error
    let mut table = InferenceTable::new();
    let environment0 = Environment::new();
    let a = table.new_variable(U0).to_ty();
    let b = table.new_variable(U0).to_ty();
    table
        .unify(&environment0, &a, &ty!(apply (item 0) (expr b)))
        .unwrap();
    table.unify(&environment0, &a, &b).unwrap_err();
}

#[test]
fn universe_error_indirect_1() {
    // exists(A -> forall(X -> exists(B -> B = X, A = B))) ---> error
    let mut table = InferenceTable::new();
    let environment0 = Environment::new();
    let a = table.new_variable(U0).to_ty();
    let b = table.new_variable(U1).to_ty();
    table
        .unify(&environment0, &b, &ty!(apply (skol 1)))
        .unwrap();
    table.unify(&environment0, &a, &b).unwrap_err();
}

#[test]
fn universe_error_indirect_2() {
    // exists(A -> forall(X -> exists(B -> B = A, B = X))) ---> error
    let mut table = InferenceTable::new();
    let environment0 = Environment::new();
    let a = table.new_variable(U0).to_ty();
    let b = table.new_variable(U1).to_ty();
    table.unify(&environment0, &a, &b).unwrap();
    table
        .unify(&environment0, &b, &ty!(apply (skol 1)))
        .unwrap_err();
}

#[test]
fn universe_promote() {
    // exists(A -> forall(X -> exists(B -> A = foo(B), A = foo(i32)))) ---> OK
    let mut table = InferenceTable::new();
    let environment0 = Environment::new();
    let a = table.new_variable(U0).to_ty();
    let b = table.new_variable(U1).to_ty();
    table
        .unify(&environment0, &a, &ty!(apply (item 0) (expr b)))
        .unwrap();
    table
        .unify(&environment0, &a, &ty!(apply (item 0) (apply (item 1))))
        .unwrap();
}

#[test]
fn universe_promote_bad() {
    // exists(A -> forall(X -> exists(B -> A = foo(B), B = X))) ---> error
    let mut table = InferenceTable::new();
    let environment0 = Environment::new();
    let a = table.new_variable(U0).to_ty();
    let b = table.new_variable(U1).to_ty();
    table
        .unify(&environment0, &a, &ty!(apply (item 0) (expr b)))
        .unwrap();
    table
        .unify(&environment0, &b, &ty!(apply (skol 1)))
        .unwrap_err();
}

#[test]
fn projection_eq() {
    // exists(A -> A = Item0<<A as Item1>::foo>)
    //                       ^^^^^^^^^^^^ Can A repeat here? For now,
    //                       we say no, but it's an interesting question.
    let mut table = InferenceTable::new();
    let environment0 = Environment::new();
    let a = table.new_variable(U0).to_ty();

    // expect an error ("cycle during unification")
    table
        .unify(
            &environment0,
            &a,
            &ty!(apply (item 0) (projection (item 1) (expr a))),
        )
        .unwrap_err();
}

const U0: UniverseIndex = UniverseIndex { counter: 0 };
const U1: UniverseIndex = UniverseIndex { counter: 1 };
const U2: UniverseIndex = UniverseIndex { counter: 2 };

fn make_table() -> InferenceTable {
    let mut table = InferenceTable::new();
    let _ = table.new_universe(); // U1
    let _ = table.new_universe(); // U2
    table
}

#[test]
fn quantify_simple() {
    let mut table = make_table();
    let _ = table.new_variable(U0);
    let _ = table.new_variable(U1);
    let _ = table.new_variable(U2);

    assert_eq!(
        table
            .canonicalize(&ty!(apply (item 0) (var 2) (var 1) (var 0)))
            .quantified,
        Canonical {
            value: ty!(apply (item 0) (var 0) (var 1) (var 2)),
            binders: vec![
                ParameterKind::Ty(U2),
                ParameterKind::Ty(U1),
                ParameterKind::Ty(U0),
            ],
        }
    );
}

#[test]
fn quantify_bound() {
    let mut table = make_table();
    let environment0 = Environment::new();

    let v0 = table.new_variable(U0).to_ty();
    let v1 = table.new_variable(U1).to_ty();
    let v2a = table.new_variable(U2).to_ty();
    let v2b = table.new_variable(U2).to_ty();

    table
        .unify(
            &environment0,
            &v2b,
            &ty!(apply (item 1) (expr v1) (expr v0)),
        )
        .unwrap();

    assert_eq!(
        table
            .canonicalize(&ty!(apply (item 0) (expr v2b) (expr v2a) (expr v1) (expr v0)))
            .quantified,
        Canonical {
            value: ty!(apply (item 0) (apply (item 1) (var 0) (var 1)) (var 2) (var 0) (var 1)),
            binders: vec![
                ParameterKind::Ty(U1),
                ParameterKind::Ty(U0),
                ParameterKind::Ty(U2),
            ],
        }
    );
}

#[test]
fn quantify_ty_under_binder() {
    let mut table = make_table();
    let v0 = table.new_variable(U0);
    let v1 = table.new_variable(U0);
    let _r0 = table.new_variable(U0);

    // Unify v0 and v1.
    let environment0 = Environment::new();
    table
        .unify(&environment0, &v0.to_ty(), &v1.to_ty())
        .unwrap();

    // Here: the `for_all` introduces 3 binders, so `(var 3)`
    // references `v0` and `(var v4)` references `v1` above.
    assert_eq!(
        table
            .canonicalize(
                &ty!(for_all 3 (apply (item 0) (var 1) (var 3) (var 4) (lifetime (var 3))))
            )
            .quantified,
        Canonical {
            value: ty!(for_all 3 (apply (item 0) (var 1) (var 3) (var 3) (lifetime (var 4)))),
            binders: vec![ParameterKind::Ty(U0), ParameterKind::Lifetime(U0)],
        }
    );
}

#[test]
fn lifetime_constraint_indirect() {
    let mut table = InferenceTable::new();
    let _ = table.new_universe(); // U1

    let _t_0 = table.new_variable(U0);
    let _l_1 = table.new_variable(U1);

    let environment0 = Environment::new();

    // Here, we unify '?1 (the lifetime variable in universe 1) with
    // '!1.
    let t_a = ty!(apply (item 0) (lifetime (skol 1)));
    let t_b = ty!(apply (item 0) (lifetime (var 1)));
    let UnificationResult { goals, constraints } = table.unify(&environment0, &t_a, &t_b).unwrap();
    assert!(goals.is_empty());
    assert!(constraints.is_empty());

    // Here, we try to unify `?0` (the type variable in universe 0)
    // with something that involves `'?1`. Since `'?1` has been
    // unified with `'!1`, and `'!1` is not visible from universe 0,
    // we will replace `'!1` with a new variable `'?2` and introduce a
    // (likely unsatisfiable) constraint relating them.
    let t_c = ty!(var 0);
    let UnificationResult { goals, constraints } = table.unify(&environment0, &t_c, &t_b).unwrap();
    assert!(goals.is_empty());
    assert_eq!(constraints.len(), 1);
    assert_eq!(
        format!("{:?}", constraints[0]),
        "InEnvironment { environment: Env([]), goal: \'?2 == \'!1 }",
    );
}
