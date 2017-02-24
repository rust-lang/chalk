use ena::unify::{UnifyKey, UnifyValue};
use ir;
use std::cmp::min;
use std::fmt;
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

    pub fn to_ty(&self) -> ir::Ty {
        ir::Ty::Var(self.index as usize)
    }
}

impl UnifyKey for TyInferenceVariable {
    type Value = TyInferenceValue;

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

/// The value of an inference variable. We start out as `Unbound` with
/// a universe index; when the inference variable is assigned a value,
/// it becomes bound and refers to an entry in the
/// `InferenceTable.value` vector.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TyInferenceValue {
    Unbound(ir::UniverseIndex),
    Bound(ValueIndex),
}

impl UnifyValue for TyInferenceValue {
    fn unify_values(a: &TyInferenceValue, b: &TyInferenceValue)
                    -> Result<TyInferenceValue, (TyInferenceValue, TyInferenceValue)> {
        match (*a, *b) {
            (TyInferenceValue::Unbound(ui_a), TyInferenceValue::Unbound(ui_b)) => {
                Ok(TyInferenceValue::Unbound(min(ui_a, ui_b)))
            }
            (bound @ TyInferenceValue::Bound(_), TyInferenceValue::Unbound(_)) |
            (TyInferenceValue::Unbound(_), bound @ TyInferenceValue::Bound(_)) => {
                Ok(bound)
            }
            (TyInferenceValue::Bound(_), TyInferenceValue::Bound(_)) => {
                // we don't even try to allow unifying things that are
                // already bound; that is handled at a higher-level by
                // the `InferenceTable`; this could probably just be a
                // `panic!` actually
                Err((*a, *b))
            }
        }
    }
}

impl fmt::Debug for TyInferenceVariable {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "?{}", self.index)
    }
}
