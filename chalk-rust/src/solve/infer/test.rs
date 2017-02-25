use fold::*;
use solve::environment::Environment;
use super::*;

macro_rules! ty {
    (apply $n:tt $($arg:tt)*) => {
        Ty::Apply(ApplicationTy {
            name: ty_name!($n),
            parameters: vec![$(arg!($arg)),*],
        })
    };

    (projection (item $n:tt) $($arg:tt)*) => {
        Ty::Projection(ProjectionTy {
            associated_ty_id: ItemId { index: $n },
            parameters: vec![$(arg!($arg)),*],
        })
    };

    (var $b:expr) => {
        Ty::Var($b)
    };

    (expr $b:expr) => {
        $b.clone()
    };

    (($($b:tt)*)) => {
        ty!($($b)*)
    };
}

macro_rules! trait_ref {
    ((item $n:tt) $($arg:tt)*) => {
        TraitRef {
            trait_id: ItemId { index: $n },
            parameters: vec![$(arg!($arg)),*],
        }
    };

    (($($b:tt)*)) => {
        trait_ref!($($b)*)
    };
}

macro_rules! arg {
    ($arg:tt) => {
        ParameterKind::Ty(ty!($arg))
    }
}

macro_rules! ty_name {
    ((item $n:expr)) => { TypeName::ItemId(ItemId { index: $n }) };
    ((skol $n:expr)) => { TypeName::ForAll(UniverseIndex { counter: $n }) }
}

impl InferenceTable {
    pub fn normalize<T>(&mut self, value: &T) -> T::Result
        where T: Fold
    {
        value.fold_with(&mut Normalizer { table: self }, 0).unwrap()
    }
}

struct Normalizer<'a> {
    table: &'a mut InferenceTable,
}

impl<'q> Folder for Normalizer<'q> {
    fn fold_free_var(&mut self, depth: usize, binders: usize) -> Result<Ty> {
        assert_eq!(binders, 0);
        let var = TyInferenceVariable::from_depth(depth);
        match self.table.probe_var(var) {
            Some(ty) => ty.fold_with(self, 0),
            None => Ok(var.to_ty()),
        }
    }

    fn fold_free_lifetime_var(&mut self, depth: usize, binders: usize) -> Result<Lifetime> {
        assert_eq!(binders, 0);
        Ok(LifetimeInferenceVariable::from_depth(depth).to_lifetime())
    }

    fn fold_free_krate_var(&mut self, depth: usize, binders: usize) -> Result<Krate> {
        assert_eq!(binders, 0);
        Ok(KrateInferenceVariable::from_depth(depth).to_krate())
    }
}

#[test]
fn infer() {
    let mut table = InferenceTable::new();
    let environment0 = Environment::new();
    let a = table.new_variable(environment0.universe).to_ty();
    let b = table.new_variable(environment0.universe).to_ty();
    table.unify(&environment0, &a, &ty!(apply (item 0) (expr b))).unwrap();
    assert_eq!(table.normalize(&a), ty!(apply (item 0) (expr b)));
    table.unify(&environment0, &b, &ty!(apply (item 1))).unwrap();
    assert_eq!(table.normalize(&a), ty!(apply (item 0) (apply (item 1))));
}

#[test]
fn universe_error() {
    // exists(A -> forall(X -> A = X)) ---> error
    let mut table = InferenceTable::new();
    let environment0 = Environment::new();
    let a = table.new_variable(environment0.universe).to_ty();
    table.unify(&environment0, &a, &ty!(apply (skol 1))).unwrap_err();
}

#[test]
fn cycle_error() {
    // exists(A -> A = foo A) ---> error
    let mut table = InferenceTable::new();
    let environment0 = Environment::new();
    let a = table.new_variable(environment0.universe).to_ty();
    table.unify(&environment0, &a, &ty!(apply (skol 1) (expr a))).unwrap_err();
}

#[test]
fn cycle_indirect() {
    // exists(A -> A = foo B, A = B) ---> error
    let mut table = InferenceTable::new();
    let environment0 = Environment::new();
    let a = table.new_variable(environment0.universe).to_ty();
    let b = table.new_variable(environment0.universe).to_ty();
    table.unify(&environment0, &a, &ty!(apply (item 0) (expr b))).unwrap();
    table.unify(&environment0, &a, &b).unwrap_err();
}

#[test]
fn universe_error_indirect_1() {
    // exists(A -> forall(X -> exists(B -> B = X, A = B))) ---> error
    let mut table = InferenceTable::new();
    let environment0 = Environment::new();
    let environment1 = environment0.new_universe();
    let a = table.new_variable(environment0.universe).to_ty();
    let b = table.new_variable(environment1.universe).to_ty();
    table.unify(&environment1, &b, &ty!(apply (skol 1))).unwrap();
    table.unify(&environment1, &a, &b).unwrap_err();
}

#[test]
fn universe_error_indirect_2() {
    // exists(A -> forall(X -> exists(B -> B = A, B = X))) ---> error
    let mut table = InferenceTable::new();
    let environment0 = Environment::new();
    let environment1 = environment0.new_universe();
    let a = table.new_variable(environment0.universe).to_ty();
    let b = table.new_variable(environment1.universe).to_ty();
    table.unify(&environment1, &a, &b).unwrap();
    table.unify(&environment1, &b, &ty!(apply (skol 1))).unwrap_err();
}

#[test]
fn universe_promote() {
    // exists(A -> forall(X -> exists(B -> A = foo(B), A = foo(i32)))) ---> OK
    let mut table = InferenceTable::new();
    let environment0 = Environment::new();
    let environment1 = environment0.new_universe();
    let a = table.new_variable(environment0.universe).to_ty();
    let b = table.new_variable(environment1.universe).to_ty();
    table.unify(&environment1, &a, &ty!(apply (item 0) (expr b))).unwrap();
    table.unify(&environment1, &a, &ty!(apply (item 0) (apply (item 1)))).unwrap();
}

#[test]
fn universe_promote_bad() {
    // exists(A -> forall(X -> exists(B -> A = foo(B), B = X))) ---> error
    let mut table = InferenceTable::new();
    let environment0 = Environment::new();
    let environment1 = environment0.new_universe();
    let a = table.new_variable(environment0.universe).to_ty();
    let b = table.new_variable(environment1.universe).to_ty();
    table.unify(&environment1, &a, &ty!(apply (item 0) (expr b))).unwrap();
    table.unify(&environment1, &b, &ty!(apply (skol 1))).unwrap_err();
}

#[test]
fn projection_eq() {
    // exists(A -> A = Item0<<A as Item1>::foo>)
    //                       ^^^^^^^^^^^^ Can A repeat here? For now,
    //                       we say no, but it's an interesting question.
    let mut table = InferenceTable::new();
    let environment0 = Environment::new();
    let a = table.new_variable(environment0.universe).to_ty();

    // expect an error ("cycle during unification")
    table.unify(&environment0,
               &a,
               &ty!(apply (item 0) (projection (item 1) (expr a))))
        .unwrap_err();
}

const U0: UniverseIndex = UniverseIndex { counter: 0 };
const U1: UniverseIndex = UniverseIndex { counter: 1 };
const U2: UniverseIndex = UniverseIndex { counter: 2 };

#[test]
fn quantify_simple() {
    let mut table = InferenceTable::new_with_vars(&[
        ParameterKind::Ty(U0),
        ParameterKind::Ty(U1),
        ParameterKind::Ty(U2),
    ]);

    assert_eq!(
        table.make_query(&ty!(apply (item 0) (var 2) (var 1) (var 0))),
        Query {
            value: ty!(apply (item 0) (var 0) (var 1) (var 2)),
            binders: vec![ParameterKind::Ty(U2), ParameterKind::Ty(U1), ParameterKind::Ty(U0)],
        });
}

#[test]
fn quantify_bound() {
    let mut table = InferenceTable::new();

    let environment0 = Environment::new();
    let environment1 = environment0.new_universe();
    let environment2 = environment1.new_universe();

    let v0 = table.new_variable(environment0.universe).to_ty();
    let v1 = table.new_variable(environment1.universe).to_ty();
    let v2a = table.new_variable(environment2.universe).to_ty();
    let v2b = table.new_variable(environment2.universe).to_ty();

    table.unify(&environment0,
                &v2b,
                &ty!(apply (item 1) (expr v1) (expr v0)))
        .unwrap();

    assert_eq!(
        table.make_query(&ty!(apply (item 0) (expr v2b) (expr v2a) (expr v1) (expr v0))),
        Query {
            value: ty!(apply (item 0) (apply (item 1) (var 0) (var 1)) (var 2) (var 0) (var 1)),
            binders: vec![ParameterKind::Ty(U1), ParameterKind::Ty(U0), ParameterKind::Ty(U2)],
        });
}
