use crate::debug::Angle;
use crate::tls;
use crate::LifetimeData;
use crate::ProjectionTy;
use crate::TyData;
use std::fmt::{self, Debug};
use std::hash::Hash;
use std::marker::PhantomData;

/// A "type family" encapsulates the concrete representation of
/// certain "core types" from chalk-ir. All the types in chalk-ir are
/// parameterized by a `TF: TypeFamily`, and so (e.g.) if they want to
/// store a type, they don't store a `Ty<TF>` instance directly, but
/// rather prefer a `Ty<TF>`. You can think of `TF::Type` as the
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
    type InternedType: Debug + Clone + Eq + Ord + Hash;

    /// "Interned" representation of lifetimes. You can use the
    /// `Lookup` trait to convert this to a `Lifetime<Self>`.
    type InternedLifetime: Debug + Clone + Eq + Ord + Hash;

    /// Prints the debug representation of a projection. To get good
    /// results, this requires inspecting TLS, and is difficult to
    /// code without reference to a specific type-family (and hence
    /// fully known types).
    fn debug_projection(
        projection: &ProjectionTy<Self>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> fmt::Result;

    /// Create an "interned" type from `ty`. You can also use the
    /// `TyData::intern` method, which is preferred.
    fn intern_ty(ty: TyData<Self>) -> Self::InternedType;

    /// Lookup the `TyData` from an interned type.
    fn ty_data(ty: &Self::InternedType) -> &TyData<Self>;

    /// Create an "interned" type from `lifetime`. You can also use
    /// the `Lifetime::intern` method, which is preferred.
    fn intern_lifetime(lifetime: LifetimeData<Self>) -> Self::InternedLifetime;

    fn lifetime_data(lifetime: &Self::InternedLifetime) -> &LifetimeData<Self>;
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

/// The default "type family" and the only type family used by chalk
/// itself. In this family, no interning actually occurs.
#[derive(Debug, Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct ChalkIr {}

impl TypeFamily for ChalkIr {
    type InternedType = TyData<ChalkIr>;
    type InternedLifetime = LifetimeData<ChalkIr>;

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

    fn intern_ty(ty: TyData<ChalkIr>) -> TyData<ChalkIr> {
        ty
    }

    fn ty_data(ty: &TyData<ChalkIr>) -> &TyData<Self> {
        ty
    }

    fn intern_lifetime(lifetime: LifetimeData<ChalkIr>) -> LifetimeData<ChalkIr> {
        lifetime
    }

    fn lifetime_data(lifetime: &LifetimeData<ChalkIr>) -> &LifetimeData<ChalkIr> {
        lifetime
    }
}

impl HasTypeFamily for ChalkIr {
    type TypeFamily = ChalkIr;
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

impl<TF: TypeFamily> HasTypeFamily for PhantomData<TF> {
    type TypeFamily = TF;
}

impl<A, B, TF> HasTypeFamily for (A, B)
where
    A: HasTypeFamily<TypeFamily = TF>,
    B: HasTypeFamily<TypeFamily = TF>,
    TF: TypeFamily,
{
    type TypeFamily = TF;
}
