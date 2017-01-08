use ir;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct LifetimeInferenceVariable {
    index: usize,
}

impl LifetimeInferenceVariable {
    pub fn from_depth(depth: usize) -> LifetimeInferenceVariable {
        LifetimeInferenceVariable { index: depth }
    }

    pub fn to_usize(&self) -> usize {
        self.index
    }

    pub fn to_lifetime(&self) -> ir::Lifetime {
        ir::Lifetime::Var(self.index)
    }
}
