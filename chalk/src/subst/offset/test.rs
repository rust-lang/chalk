use super::*;
use super::super::Subst;

#[test]
fn index() {
    let subst = Subst::root(22).push(44).push(66);
    let offset_subst = OffsetSubst::new(&subst);
    let offset_subst = offset_subst.push_offset();
    let offset_subst = offset_subst.push_offset();
    assert_eq!(offset_subst.get(0), None);
    assert_eq!(offset_subst.get(1), None);
    assert_eq!(offset_subst.get(2), Some(&66));
    assert_eq!(offset_subst.get(3), Some(&44));
    assert_eq!(offset_subst.get(4), Some(&22));
}

#[test]
#[should_panic]
fn oob() {
    let subst = Subst::root(22).push(44).push(66);
    let offset_subst = OffsetSubst::new(&subst);
    let offset_subst = offset_subst.push_offset();
    let offset_subst = offset_subst.push_offset();
    offset_subst.get(5);
}

