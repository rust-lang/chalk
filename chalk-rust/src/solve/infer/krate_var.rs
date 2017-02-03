use ena::unify::{UnifyKey, UnifyValue};
use ir::*;
use std::cmp::min;
use std::fmt;
use std::u32;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct KrateInferenceVariable {
    index: u32,
}

impl KrateInferenceVariable {
    pub fn from_depth(depth: usize) -> KrateInferenceVariable {
        assert!(depth < u32::MAX as usize);
        KrateInferenceVariable { index: depth as u32 }
    }

    pub fn from_u32(depth: u32) -> KrateInferenceVariable {
        KrateInferenceVariable { index: depth }
    }

    pub fn to_krate(&self) -> Krate {
        Krate::Var(self.index as usize)
    }
}

impl UnifyKey for KrateInferenceVariable {
    type Value = KrateInferenceValue;

    fn index(&self) -> u32 {
        self.index
    }

    fn from_index(u: u32) -> Self {
        KrateInferenceVariable { index: u }
    }

    fn tag() -> &'static str {
        "KrateInferenceVariable"
    }
}

/// The value of an inference variable. We start out as `Unbound` with
/// a universe index; when the inference variable is assigned a value,
/// it becomes bound and refers to an entry in the
/// `InferenceTable.value` vector.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KrateInferenceValue {
    Unbound(UniverseIndex),
    Bound(KrateId),
}

impl UnifyValue for KrateInferenceValue {
    fn unify_values(&a: &KrateInferenceValue, &b: &KrateInferenceValue)
                    -> Result<KrateInferenceValue, (KrateInferenceValue, KrateInferenceValue)> {
        match (a, b) {
            (KrateInferenceValue::Unbound(ui_a), KrateInferenceValue::Unbound(ui_b)) => {
                Ok(KrateInferenceValue::Unbound(min(ui_a, ui_b)))
            }
            (bound @ KrateInferenceValue::Bound(_), KrateInferenceValue::Unbound(_)) |
            (KrateInferenceValue::Unbound(_), bound @ KrateInferenceValue::Bound(_)) => {
                Ok(bound)
            }
            (KrateInferenceValue::Bound(c_a), KrateInferenceValue::Bound(c_b)) => {
                if c_a == c_b {
                    Ok(a)
                } else {
                    Err((a, b))
                }
            }
        }
    }
}

impl fmt::Debug for KrateInferenceVariable {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "?{}", self.index)
    }
}
