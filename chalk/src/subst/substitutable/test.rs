use ena::unify::UnifyKey;
use infer::*;
use formula::*;
use subst::*;

fn var(n: u32) -> InferenceVariable {
    InferenceVariable::from_index(n)
}

#[test]
fn subst_clause_bound_and() {
    let subst: Subst<Leaf> = Subst::root(var(0).to_leaf());
    let leaf0 = clause! {
        (and
         (leaf (bound 0))
         (forall(1) (leaf (apply "foo" (bound 0) (bound 1)))))
        //                             ^^^^^^^^^ should not be substituted
        //                                       ^^^^^^^^^ should be substituted
    };
    let leaf1 = leaf0.subst(&subst);
    let leaf_expected = clause! {
        (and
         (leaf (expr var(0).to_leaf())) // <-- was substituted
         (forall(1) (leaf (apply "foo" (bound 0) (expr var(0).to_leaf())))))
        //                                       ^^^^^^^^^^^^^^^^^^^^^^^ was substituted
    };
    println!("leaf0={:?}", leaf0);
    println!("leaf1={:?}", leaf1);
    assert_eq!(leaf1, leaf_expected);
}

#[test]
fn subst_goal_bound_and() {
    let subst: Subst<Leaf> = Subst::root(var(0).to_leaf());
    let leaf0 = goal! {
        (and
         (leaf (bound 0))
         (forall(1) (leaf (apply "foo" (bound 0) (bound 1)))))
        //                             ^^^^^^^^^ should not be substituted
        //                                       ^^^^^^^^^ should be substituted
    };
    let leaf1 = leaf0.subst(&subst);
    let leaf_expected = goal! {
        (and
         (leaf (expr var(0).to_leaf())) // <-- was substituted
         (forall(1) (leaf (apply "foo" (bound 0) (expr var(0).to_leaf())))))
        //                                       ^^^^^^^^^^^^^^^^^^^^^^^ was substituted
    };
    println!("leaf0={:?}", leaf0);
    println!("leaf1={:?}", leaf1);
    assert_eq!(leaf1, leaf_expected);
}
