use chalk_ir::cast::Cast;
use chalk_ir::interner::Interner;
use chalk_ir::*;
use ena::unify::{UnifyKey, UnifyValue};
use std::cmp::min;
use std::fmt;
use std::marker::PhantomData;
use std::u32;

/// Wrapper around `chalk_ir::InferenceVar` for coherence purposes.
/// An inference variable represents an unknown term -- either a type
/// or a lifetime. The variable itself is just an index into the
/// unification table; the unification table maps it to an
/// `InferenceValue`.
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
///     `ir::GenericArg`, and hence it does carry along the kind of the
///     variable via the enum variant. However, we should always know
///     the kind of the variable from context, and hence we typically
///     "downcast" the resulting variable using
///     e.g. `value.ty().unwrap()`.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EnaVariable<I: Interner> {
    var: InferenceVar,
    phantom: PhantomData<I>,
}

impl<I: Interner> From<InferenceVar> for EnaVariable<I> {
    fn from(var: InferenceVar) -> EnaVariable<I> {
        EnaVariable {
            var,
            phantom: PhantomData,
        }
    }
}

impl<I: Interner> From<EnaVariable<I>> for InferenceVar {
    fn from(ena_var: EnaVariable<I>) -> InferenceVar {
        ena_var.var
    }
}

impl<I: Interner> EnaVariable<I> {
    /// Convert this inference variable into a type. When using this
    /// method, naturally you should know from context that the kind
    /// of this inference variable is a type (we can't check it).
    pub fn to_ty_with_kind(self, interner: I, kind: TyVariableKind) -> Ty<I> {
        self.var.to_ty(interner, kind)
    }

    /// Same as `to_ty_with_kind`, but the kind is set to `TyVariableKind::General`.
    /// This should be used instead of `to_ty_with_kind` when creating a new
    /// inference variable (when the kind is not known).
    pub fn to_ty(self, interner: I) -> Ty<I> {
        self.var.to_ty(interner, TyVariableKind::General)
    }

    /// Convert this inference variable into a lifetime. When using this
    /// method, naturally you should know from context that the kind
    /// of this inference variable is a lifetime (we can't check it).
    pub fn to_lifetime(self, interner: I) -> Lifetime<I> {
        self.var.to_lifetime(interner)
    }

    /// Convert this inference variable into a const. When using this
    /// method, naturally you should know from context that the kind
    /// of this inference variable is a const (we can't check it).
    pub fn to_const(self, interner: I, ty: Ty<I>) -> Const<I> {
        self.var.to_const(interner, ty)
    }
}

impl<I: Interner> UnifyKey for EnaVariable<I> {
    type Value = InferenceValue<I>;

    fn index(&self) -> u32 {
        self.var.index()
    }

    fn from_index(u: u32) -> Self {
        EnaVariable::from(InferenceVar::from(u))
    }

    fn tag() -> &'static str {
        "EnaVariable"
    }
}

/// The value of an inference variable. We start out as `Unbound` with a
/// universe index; when the inference variable is assigned a value, it becomes
/// bound and records that value. See `EnaVariable` for more details.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InferenceValue<I: Interner> {
    Unbound(UniverseIndex),
    Bound(GenericArg<I>),
}

impl<I: Interner> InferenceValue<I> {
    pub fn from_ty(interner: I, ty: Ty<I>) -> Self {
        InferenceValue::Bound(ty.cast(interner))
    }

    pub fn from_lifetime(interner: I, lifetime: Lifetime<I>) -> Self {
        InferenceValue::Bound(lifetime.cast(interner))
    }

    pub fn from_const(interner: I, constant: Const<I>) -> Self {
        InferenceValue::Bound(constant.cast(interner))
    }
}

impl<I: Interner> UnifyValue for InferenceValue<I> {
    type Error = (InferenceValue<I>, InferenceValue<I>);

    fn unify_values(
        a: &InferenceValue<I>,
        b: &InferenceValue<I>,
    ) -> Result<InferenceValue<I>, (InferenceValue<I>, InferenceValue<I>)> {
        match (a, b) {
            (&InferenceValue::Unbound(ui_a), &InferenceValue::Unbound(ui_b)) => {
                Ok(InferenceValue::Unbound(min(ui_a, ui_b)))
            }
            (bound @ &InferenceValue::Bound(_), &InferenceValue::Unbound(_))
            | (&InferenceValue::Unbound(_), bound @ &InferenceValue::Bound(_)) => Ok(bound.clone()),
            (&InferenceValue::Bound(_), &InferenceValue::Bound(_)) => {
                panic!("we should not be asked to unify two bound things")
            }
        }
    }
}

impl<I: Interner> fmt::Debug for EnaVariable<I> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}", self.var)
    }
}
