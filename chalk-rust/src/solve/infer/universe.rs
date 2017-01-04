#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UniverseIndex {
    pub counter: usize,
}

impl UniverseIndex {
    pub fn root() -> UniverseIndex {
        UniverseIndex { counter: 0 }
    }
}


