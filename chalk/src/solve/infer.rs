use ena::unify::{UnifyKey, UnifyValue, InfallibleUnifyValue};
use solve::*;
use std::cmp::max;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct InferenceVariable {
    index: u32,
}

impl UnifyKey for InferenceVariable {
    type Value = UniverseIndex;

    fn index(&self) -> u32 {
        self.index
    }

    fn from_index(u: u32) -> Self {
        InferenceVariable { index: u }
    }

    fn tag() -> &'static str {
        "InferenceVariable"
    }
}

impl UnifyValue for UniverseIndex {
    fn unify_values(a: &UniverseIndex, b: &UniverseIndex)
                    -> Result<UniverseIndex, (UniverseIndex, UniverseIndex)> {
        Ok(*max(a, b))
    }
}

impl InfallibleUnifyValue for UniverseIndex {
}
