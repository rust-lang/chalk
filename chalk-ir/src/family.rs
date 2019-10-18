use crate::debug::Angle;
use crate::tls;
use crate::ProjectionTy;
use std::fmt::{self, Debug};
use std::hash::Hash;

pub trait TypeFamily: Debug + Copy + Eq + Ord + Hash {
    fn debug_projection(
        projection: &ProjectionTy<Self>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> fmt::Result;
}

pub trait HasTypeFamily {
    type TypeFamily: TypeFamily;
}

#[derive(Debug, Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct ChalkIr {}

impl TypeFamily for ChalkIr {
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
}

impl<T: HasTypeFamily> HasTypeFamily for [T] {
    type TypeFamily = T::TypeFamily;
}

impl<T: HasTypeFamily> HasTypeFamily for Vec<T> {
    type TypeFamily = T::TypeFamily;
}
