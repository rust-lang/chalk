use super::*;

#[test]
fn index() {
    let subst = Subst::root(22).push(44).push(66);
    assert_eq!(subst[0], 66);
    assert_eq!(subst[1], 44);
    assert_eq!(subst[2], 22);
}

#[test]
#[should_panic]
fn oob() {
    let subst = Subst::root(22).push(44).push(66);
    subst[3];
}

