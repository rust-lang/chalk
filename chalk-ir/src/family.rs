use crate::cast::Cast;
use crate::debug::Angle;
use crate::fold::{Fold, Folder, ReflexiveFold};
use crate::tls;
use crate::zip::Zip;
use crate::Lifetime;
use crate::Parameter;
use crate::ParameterKind;
use crate::ProjectionTy;
use crate::Ty;
use chalk_engine::fallible::Fallible;
use std::fmt::{self, Debug};
use std::hash::Hash;

/// A "type family" encapsulates the concrete representation of
/// certain "core types" from chalk-ir. All the types in chalk-ir are
/// parameterized by a `TF: TypeFamily`, and so (e.g.) if they want to
/// store a type, they don't store a `Ty<TF>` instance directly, but
/// rather prefer a `TF::Type`. You can think of `TF::Type` as the
/// interned representation (and, indeed, it may well be an interned
/// pointer, e.g. in rustc).
///
/// Type families allow chalk to be embedded in different contexts
/// where the concrete representation of core types varies. They also
/// allow us to write generic code that reasons about multiple
/// distinct sets of types by using distinct generic type parameters
/// (e.g., `SourceTF` and `TargetTF`) -- even if those type parameters
/// wind up being mapped to the same underlying type families in the
/// end.
pub trait TypeFamily: Debug + Copy + Eq + Ord + Hash {
    /// "Interned" representation of types. You can use the `Lookup`
    /// trait to convert this to a `Ty<Self>`.
    type Type: Debug
        + Clone
        + Eq
        + Ord
        + Hash
        + ReflexiveFold<Self>
        + Zip<Self>
        + Lookup<Ty<Self>>
        + Cast<Parameter<Self>>;

    /// "Interned" representation of lifetimes. You can use the
    /// `Lookup` trait to convert this to a `Lifetime<Self>`.
    type Lifetime: Debug
        + Clone
        + Eq
        + Ord
        + Hash
        + ReflexiveFold<Self>
        + Zip<Self>
        + Lookup<Lifetime<Self>>
        + Cast<Parameter<Self>>;

    /// Prints the debug representation of a projection. To get good
    /// results, this requires inspecting TLS, and is difficult to
    /// code without reference to a specific type-family (and hence
    /// fully known types).
    fn debug_projection(
        projection: &ProjectionTy<Self>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> fmt::Result;

    /// Create an "interned" type from `ty`. You can also use the
    /// `Ty::intern` method, which is preferred.
    fn intern_ty(ty: Ty<Self>) -> Self::Type;

    /// Create an "interned" type from `lifetime`. You can also use
    /// the `Lifetime::intern` method, which is preferred.
    fn intern_lifetime(lifetime: Lifetime<Self>) -> Self::Lifetime;
}

/// Implemented by types that have an associated type family (which
/// are virtually all of the types in chalk-ir, for example).
/// This lets us map from a type like `Ty<TF>` to the parameter `TF`.
///
/// It's particularly useful for writing `Fold` impls for generic
/// types like `Binder<T>`, since it allows us to figure out the type
/// family of `T`.
pub trait HasTypeFamily {
    type TypeFamily: TypeFamily;
}

/// Given an interned representation, convert to the uninterned enum
/// `DataType`. Used to (e.g.) convert from `TF::Type` to `Ty<TF>`.
pub trait Lookup<DataType> {
    fn lookup_ref(&self) -> &DataType;

    fn lookup(self) -> DataType;
}

impl Lookup<Ty<ChalkIr>> for Ty<ChalkIr> {
    fn lookup_ref(&self) -> &Ty<ChalkIr> {
        self
    }

    fn lookup(self) -> Ty<ChalkIr> {
        self
    }
}

impl Lookup<Lifetime<ChalkIr>> for Lifetime<ChalkIr> {
    fn lookup_ref(&self) -> &Lifetime<ChalkIr> {
        self
    }

    fn lookup(self) -> Lifetime<ChalkIr> {
        self
    }
}

/// The default "type family" and the only type family used by chalk
/// itself. In this family, no interning actually occurs.
#[derive(Debug, Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct ChalkIr {}

impl TypeFamily for ChalkIr {
    type Type = Ty<ChalkIr>;
    type Lifetime = Lifetime<ChalkIr>;

    fn debug_projection(
        projection: &ProjectionTy<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        tls::with_current_program(|p| match p {
            Some(program) => program.debug_projection(projection, fmt),
            None => write!(
                fmt,
                "({:?}){:?}",
                projection.associated_ty_id,
                Angle(&projection.parameters)
            ),
        })
    }

    fn intern_ty(ty: Ty<ChalkIr>) -> Ty<ChalkIr> {
        ty
    }

    fn intern_lifetime(lifetime: Lifetime<ChalkIr>) -> Lifetime<ChalkIr> {
        lifetime
    }
}

impl<T: HasTypeFamily> HasTypeFamily for [T] {
    type TypeFamily = T::TypeFamily;
}

impl<T: HasTypeFamily> HasTypeFamily for Vec<T> {
    type TypeFamily = T::TypeFamily;
}

impl<T: HasTypeFamily + ?Sized> HasTypeFamily for &T {
    type TypeFamily = T::TypeFamily;
}

impl<A, B, TF> HasTypeFamily for (A, B)
where
    A: HasTypeFamily<TypeFamily = TF>,
    B: HasTypeFamily<TypeFamily = TF>,
    TF: TypeFamily,
{
    type TypeFamily = TF;
}

impl Fold<ChalkIr> for Ty<ChalkIr> {
    type Result = Self;
    fn fold_with(
        &self,
        folder: &mut dyn Folder<ChalkIr>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        folder.fold_ty(self, binders)
    }
}

impl Cast<Parameter<ChalkIr>> for Ty<ChalkIr> {
    fn cast(self) -> Parameter<ChalkIr> {
        Parameter(ParameterKind::Ty(self))
    }
}

impl Fold<ChalkIr> for Lifetime<ChalkIr> {
    type Result = Self;
    fn fold_with(
        &self,
        folder: &mut dyn Folder<ChalkIr>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        folder.fold_lifetime(self, binders)
    }
}

impl Cast<Parameter<ChalkIr>> for Lifetime<ChalkIr> {
    fn cast(self) -> Parameter<ChalkIr> {
        Parameter(ParameterKind::Lifetime(self))
    }
}
