use ir;
use std::fmt;

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

impl fmt::Debug for LifetimeInferenceVariable {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "'?{}", self.index)
    }
}
