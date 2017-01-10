use lalrpop_intern::intern;
use errors::*;
use ir::*;
use fold::*;
use super::*;

macro_rules! ty {
    (apply $n:tt $($arg:tt)*) => {
        Ty::Apply(ApplicationTy {
            name: ty_name!($n),
            parameters: vec![$(arg!($arg)),*],
        })
    };

    (projection $t:tt $n:tt) => {
        Ty::Projection(ProjectionTy {
            trait_ref: trait_ref!($t),
            name: intern($n),
        })
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
        value.fold_with(&mut Normalizer { table: self }).unwrap()
    }
}

struct Normalizer<'a> {
    table: &'a mut InferenceTable
}

impl<'q> Folder for Normalizer<'q> {
    fn fold_var(&mut self, depth: usize) -> Result<Ty> {
        let var = InferenceVariable::from_depth(depth);
        match self.table.probe_var(var) {
            Some(ty) => (*ty).fold_with(self),
            None => Ok(var.to_ty()),
        }
    }

    fn fold_lifetime_var(&mut self, depth: usize) -> Result<Lifetime> {
        Ok(LifetimeInferenceVariable::from_depth(depth).to_lifetime())
    }
}

#[test]
fn infer() {
    let mut table = InferenceTable::new();
    let universe0 = UniverseIndex { counter: 0 };
    let a = table.new_variable(universe0).to_ty();
    let b = table.new_variable(universe0).to_ty();
    table.unify(&a, &ty!(apply (item 0) (expr b))).unwrap();
    assert_eq!(table.normalize(&a), ty!(apply (item 0) (expr b)));
    table.unify(&b, &ty!(apply (item 1))).unwrap();
    assert_eq!(table.normalize(&a), ty!(apply (item 0) (apply (item 1))));
}

#[test]
fn universe_error() {
    // exists(A -> forall(X -> A = X)) ---> error
    let mut table = InferenceTable::new();
    let universe0 = UniverseIndex { counter: 0 };
    let a = table.new_variable(universe0).to_ty();
    table.unify(&a, &ty!(apply (skol 1))).unwrap_err();
}

#[test]
fn cycle_error() {
    // exists(A -> A = foo A) ---> error
    let mut table = InferenceTable::new();
    let universe0 = UniverseIndex { counter: 0 };
    let a = table.new_variable(universe0).to_ty();
    table.unify(&a, &ty!(apply (skol 1) (expr a))).unwrap_err();
}

#[test]
fn cycle_indirect() {
    // exists(A -> A = foo B, A = B) ---> error
    let mut table = InferenceTable::new();
    let universe0 = UniverseIndex { counter: 0 };
    let a = table.new_variable(universe0).to_ty();
    let b = table.new_variable(universe0).to_ty();
    table.unify(&a, &ty!(apply (item 0) (expr b))).unwrap();
    table.unify(&a, &b).unwrap_err();
}

#[test]
fn universe_error_indirect_1() {
    // exists(A -> forall(X -> exists(B -> B = X, A = B))) ---> error
    let mut table = InferenceTable::new();
    let universe0 = UniverseIndex { counter: 0 };
    let universe1 = UniverseIndex { counter: 1 };
    let a = table.new_variable(universe0).to_ty();
    let b = table.new_variable(universe1).to_ty();
    table.unify(&b, &ty!(apply (skol 1))).unwrap();
    table.unify(&a, &b).unwrap_err();
}

#[test]
fn universe_error_indirect_2() {
    // exists(A -> forall(X -> exists(B -> B = A, B = X))) ---> error
    let mut table = InferenceTable::new();
    let universe0 = UniverseIndex { counter: 0 };
    let universe1 = UniverseIndex { counter: 1 };
    let a = table.new_variable(universe0).to_ty();
    let b = table.new_variable(universe1).to_ty();
    table.unify(&a, &b).unwrap();
    table.unify(&b, &ty!(apply (skol 1))).unwrap_err();
}

#[test]
fn universe_promote() {
    // exists(A -> forall(X -> exists(B -> A = foo(B), A = foo(i32)))) ---> OK
    let mut table = InferenceTable::new();
    let universe0 = UniverseIndex { counter: 0 };
    let universe1 = UniverseIndex { counter: 1 };
    let a = table.new_variable(universe0).to_ty();
    let b = table.new_variable(universe1).to_ty();
    table.unify(&a, &ty!(apply (item 0) (expr b))).unwrap();
    table.unify(&a, &ty!(apply (item 0) (apply (item 1)))).unwrap();
}

#[test]
fn universe_promote_bad() {
    // exists(A -> forall(X -> exists(B -> A = foo(B), B = X))) ---> error
    let mut table = InferenceTable::new();
    let universe0 = UniverseIndex { counter: 0 };
    let universe1 = UniverseIndex { counter: 1 };
    let a = table.new_variable(universe0).to_ty();
    let b = table.new_variable(universe1).to_ty();
    table.unify(&a, &ty!(apply (item 0) (expr b))).unwrap();
    table.unify(&b, &ty!(apply (skol 1))).unwrap_err();
}

#[test]
fn projection_eq() {
    // exists(A -> A = Item0<<A as Item1>::foo>)
    //                       ^^^^^^^^^^^^ Can A repeat here? For now,
    //                       we say no, but it's an interesting question.
    let mut table = InferenceTable::new();
    let universe0 = UniverseIndex { counter: 0 };
    let a = table.new_variable(universe0).to_ty();

    // expect an error ("cycle during unification")
    table.unify(&a, &ty!(apply (item 0) (projection ((item 1) (expr a)) "foo"))).unwrap_err();
}

