use ena::unify::{UnifyKey, UnifyValue};
use crate::*;
use std::cmp::min;
use std::fmt;
use std::u32;

/// An inference variable represents an unknown term -- either a type
/// or a lifetime. The variable itself is just an index into the unification
/// table; the unification table maps it to an `InferenceValue`.
///
/// Inference variables can be in one of two states (represents by the variants
/// of an `InferenceValue`):
///
/// - Unbound(`ui`). In this case, the value of the variable is not yet known. We carry
///   along a universe index `ui` that tracks the universe in which the variable was
///   created; this determines what names may appear in the variable's value.
///   - In this state, we do **not** track the kind of this variable
///     (i.e., whether it represents a type or a lifetime). There is
///     no need: if it represents a lifetime, for example, then there
///     should only ever be constraints that relate it to other
///     lifetimes, or use it in lifetime position.
/// - Bound. In this case, the value of the variable is known. We
///   carry along the value. We discard the universe index in which
///   the variable was created, since that was only needed to help us
///   reject illegal values. Once the value of a variable is known, it
///   can never change.
///   - The value we actually store for variables is a
///     `ir::Parameter`, and hence it does carry along the kind of the
///     variable via the enum variant. However, we should always know
///     the kind of the variable from context, and hence we typically
///     "downcast" the resulting variable using
///     e.g. `value.ty().unwrap()`.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
crate struct InferenceVariable {
    index: u32,
}

impl InferenceVariable {
    /// Create an inference variable from a debruijn depth. In terms,
    /// like the `Ty::Var` variant, we use debruijn indices to refer
    /// to either index variables or bound variables. Presuming that
    /// the depth D in the term is greater than the number of
    /// enclosing binders B, then it refers to an inference variable,
    /// and the inference variable can be created via
    /// `InferenceVariable::from_depth(D - B)`.
    pub fn from_depth(depth: usize) -> InferenceVariable {
        assert!(depth < u32::MAX as usize);
        InferenceVariable {
            index: depth as u32,
        }
    }

    /// Convert this inference variable into a type. When using this
    /// method, naturally you should know from context that the kind
    /// of this inference variable is a type (we can't check it).
    pub fn to_ty(self) -> Ty {
        Ty::Var(self.index as usize)
    }

    /// Convert this inference variable into a lifetime. When using this
    /// method, naturally you should know from context that the kind
    /// of this inference variable is a lifetime (we can't check it).
    pub fn to_lifetime(self) -> Lifetime {
        Lifetime::Var(self.index as usize)
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

/// The value of an inference variable. We start out as `Unbound` with a
/// universe index; when the inference variable is assigned a value, it becomes
/// bound and records that value. See `InferenceVariable` for more details.
#[derive(Clone, Debug, PartialEq, Eq)]
crate enum InferenceValue {
    Unbound(UniverseIndex),
    Bound(Parameter),
}

impl From<Ty> for InferenceValue {
    fn from(ty: Ty) -> Self {
        InferenceValue::Bound(ParameterKind::Ty(ty))
    }
}

impl From<Lifetime> for InferenceValue {
    fn from(lifetime: Lifetime) -> Self {
        InferenceValue::Bound(ParameterKind::Lifetime(lifetime))
    }
}

impl UnifyValue for InferenceValue {
    fn unify_values(
        a: &InferenceValue,
        b: &InferenceValue,
    ) -> Result<InferenceValue, (InferenceValue, InferenceValue)> {
        match (a, b) {
            (&InferenceValue::Unbound(ui_a), &InferenceValue::Unbound(ui_b)) => {
                Ok(InferenceValue::Unbound(min(ui_a, ui_b)))
            }
            (bound @ &InferenceValue::Bound(_), &InferenceValue::Unbound(_)) |
            (&InferenceValue::Unbound(_), bound @ &InferenceValue::Bound(_)) => Ok(bound.clone()),
            (&InferenceValue::Bound(_), &InferenceValue::Bound(_)) => {
                panic!("we should not be asked to unify two bound things")
            }
        }
    }
}

impl fmt::Debug for InferenceVariable {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "?{}", self.index)
    }
}
