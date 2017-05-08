use ena::unify::{UnifyKey, UnifyValue};
use ir::*;
use std::cmp::min;
use std::fmt::{self, Debug};
use std::u32;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct TyInferenceVariable {
    index: u32,
}

impl TyInferenceVariable {
    pub fn from_depth(depth: usize) -> TyInferenceVariable {
        assert!(depth < u32::MAX as usize);
        TyInferenceVariable { index: depth as u32 }
    }

    pub fn from_u32(depth: u32) -> TyInferenceVariable {
        TyInferenceVariable { index: depth }
    }

    pub fn to_ty(&self) -> Ty {
        Ty::Var(self.index as usize)
    }
}

impl UnifyKey for TyInferenceVariable {
    type Value = InferenceValue<Ty>;

    fn index(&self) -> u32 {
        self.index
    }

    fn from_index(u: u32) -> Self {
        TyInferenceVariable { index: u }
    }

    fn tag() -> &'static str {
        "TyInferenceVariable"
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct LifetimeInferenceVariable {
    index: u32,
}

impl LifetimeInferenceVariable {
    pub fn from_depth(depth: usize) -> LifetimeInferenceVariable {
        LifetimeInferenceVariable { index: depth as u32 }
    }

    pub fn to_usize(&self) -> usize {
        self.index as usize
    }

    pub fn to_lifetime(&self) -> Lifetime {
        Lifetime::Var(self.to_usize())
    }
}

impl fmt::Debug for LifetimeInferenceVariable {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "'?{}", self.index)
    }
}

impl UnifyKey for LifetimeInferenceVariable {
    type Value = InferenceValue<Lifetime>;

    fn index(&self) -> u32 {
        self.index
    }

    fn from_index(u: u32) -> Self {
        LifetimeInferenceVariable { index: u }
    }

    fn tag() -> &'static str {
        "LifetimeInferenceVariable"
    }
}

/// The value of an inference variable. We start out as `Unbound` with a
/// universe index; when the inference variable is assigned a value, it becomes
/// bound and records that value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InferenceValue<V: Clone + Debug> {
    Unbound(UniverseIndex),
    Bound(V),
}

impl<V: Clone + Debug> InferenceValue<V> {
    pub fn unbound(&self) -> Option<UniverseIndex> {
        match *self {
            InferenceValue::Unbound(ui) => Some(ui),
            _ => None,
        }
    }
}

impl<V: Clone + Debug> UnifyValue for InferenceValue<V> {
    fn unify_values(a: &InferenceValue<V>, b: &InferenceValue<V>)
                    -> Result<InferenceValue<V>, (InferenceValue<V>, InferenceValue<V>)> {
        match (a, b) {
            (&InferenceValue::Unbound(ui_a), &InferenceValue::Unbound(ui_b)) => {
                Ok(InferenceValue::Unbound(min(ui_a, ui_b)))
            }
            (bound @ &InferenceValue::Bound(_), &InferenceValue::Unbound(_)) |
            (&InferenceValue::Unbound(_), bound @ &InferenceValue::Bound(_)) => {
                Ok(bound.clone())
            }
            (&InferenceValue::Bound(_), &InferenceValue::Bound(_)) => {
                panic!("we should not be asked to unify two bound things")
            }
        }
    }
}

impl fmt::Debug for TyInferenceVariable {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "?{}", self.index)
    }
}
