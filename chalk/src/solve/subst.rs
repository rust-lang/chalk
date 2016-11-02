pub struct Subst {
    parent: Option<Arc<SubstLink>>,
}

pub struct SubstLink {
    value: Leaf
}
