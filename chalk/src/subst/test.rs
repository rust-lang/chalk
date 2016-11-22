use ena::unify::UnifyKey;
use infer::*;
use formula::*;
use subst::*;

fn var(n: u32) -> InferenceVariable {
    InferenceVariable::from_index(n)
}

#[test]
fn subst_clause_bound() {
    let subst: Subst<Leaf> = Subst::root(var(0).to_leaf());
    let leaf0 = clause! {
        (forall(1) (leaf (apply "foo" (bound 0) (bound 1))))
        //                            ^^^^^^^^^ should not be substituted
        //                                      ^^^^^^^^^ should be substituted
    };
    let leaf1 = subst.apply(&leaf0);
    let leaf_expected = clause! {
        (forall(1) (leaf (apply "foo" (bound 0) (expr var(0).to_leaf()))))
        //                                      ^^^^^^^^^^^^^^^^^^^^^^^ was substituted
    };
    debug!("leaf0={:?}", leaf0);
    debug!("leaf1={:?}", leaf1);
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
    let leaf1 = subst.apply(&leaf0);
    let leaf_expected = goal! {
        (and
         (leaf (expr var(0).to_leaf())) // <-- was substituted
         (forall(1) (leaf (apply "foo" (bound 0) (expr var(0).to_leaf())))))
        //                                       ^^^^^^^^^^^^^^^^^^^^^^^ was substituted
    };
    debug!("leaf0={:?}", leaf0);
    debug!("leaf1={:?}", leaf1);
    assert_eq!(leaf1, leaf_expected);
}
