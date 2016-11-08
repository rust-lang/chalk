use ena::unify::UnifyKey;
use infer::*;
use formula::*;
use subst::*;

fn var(n: u32) -> InferenceVariable {
    InferenceVariable::from_index(n)
}

fn subst() {
    let subst: Subst<Leaf> = Subst::root(var(0).to_leaf());
    let leaf0 = clause! {
        (and (leaf (bound 0))
             (forall(1) (leaf (apply "foo" (bound 0)))))
    };
    let leaf1 = leaf0.subst(&subst);
    let leaf_expected = clause! {
        (and (leaf (expr var(0).to_leaf()))
             (forall(1) (leaf (apply "foo" (bound 0)))))
    };
    println!("leaf0={:?}", leaf0);
    println!("leaf1={:?}", leaf1);
    assert_eq!(leaf1, leaf_expected);
}
