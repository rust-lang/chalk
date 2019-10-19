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

pub trait TypeFamily: Debug + Copy + Eq + Ord + Hash {
    type Type: Debug
        + Clone
        + Eq
        + Ord
        + Hash
        + ReflexiveFold<Self>
        + Zip<Self>
        + Lookup<Ty<Self>>
        + Cast<Parameter<Self>>;

    type Lifetime: Debug
        + Clone
        + Eq
        + Ord
        + Hash
        + ReflexiveFold<Self>
        + Zip<Self>
        + Lookup<Lifetime<Self>>
        + Cast<Parameter<Self>>;

    fn debug_projection(
        projection: &ProjectionTy<Self>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> fmt::Result;

    fn intern_ty(ty: Ty<Self>) -> Self::Type;

    fn intern_lifetime(lifetime: Lifetime<Self>) -> Self::Lifetime;
}

pub trait HasTypeFamily {
    type TypeFamily: TypeFamily;
}

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
