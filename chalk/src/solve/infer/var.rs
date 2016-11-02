use ena::unify::{UnifyKey, UnifyValue};
use std::cmp::max;

use super::leaf::*;
use super::universe::UniverseIndex;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct InferenceVariable {
    index: u32,
}

impl InferenceVariable {
    pub fn to_leaf(&self) -> InferenceLeaf {
        InferenceLeaf::new(InferenceLeafData {
            kind: InferenceLeafKind::Variable(*self)
        })
    }
}

impl UnifyKey for InferenceVariable {
    type Value = InferenceValue;

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

/// The value of an inference variable. We start out as `Unbound` with
/// a universe index; when the inference variable is assigned a value,
/// it becomes bound and refers to an entry in the
/// `InferenceTable.value` vector.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InferenceValue {
    Unbound(UniverseIndex),
    Bound(ValueIndex),
}

impl UnifyValue for InferenceValue {
    fn unify_values(a: &InferenceValue, b: &InferenceValue)
                    -> Result<InferenceValue, (InferenceValue, InferenceValue)> {
        match (*a, *b) {
            (InferenceValue::Unbound(ui_a), InferenceValue::Unbound(ui_b)) => {
                Ok(InferenceValue::Unbound(max(ui_a, ui_b)))
            }
            (InferenceValue::Bound(_), _) | (_, InferenceValue::Bound(_)) => {
                // we don't even try to allow unifying things that are
                // already bound; that is handled at a higher-level by
                // the `InferenceTable`; this could probably just be a
                // `panic!` actually
                Err((*a, *b))
            }
        }
    }
}

/// An index into the `InferenceTable.values` vector.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ValueIndex {
    index: u32
}

impl ValueIndex {
    pub fn new(value: usize) -> ValueIndex {
        ValueIndex { index: value as u32 }
    }

    pub fn as_usize(&self) -> usize {
        self.index as usize
    }
}
