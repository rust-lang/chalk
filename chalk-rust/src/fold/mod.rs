use errors::*;
use ir::*;
use solve::environment::{Environment, InEnvironment};
use std::sync::Arc;

mod instantiate;
mod shifter;

pub use self::shifter::Shifter;
pub use self::instantiate::Subst;

pub trait Folder {
    fn fold_free_var(&mut self, depth: usize, binders: usize) -> Result<Ty>;
    fn fold_free_lifetime_var(&mut self, depth: usize, binders: usize) -> Result<Lifetime>;
}

impl<'f, F: Folder> Folder for &'f mut F {
    fn fold_free_var(&mut self, depth: usize, binders: usize) -> Result<Ty> {
        (**self).fold_free_var(depth, binders)
    }

    fn fold_free_lifetime_var(&mut self, depth: usize, binders: usize) -> Result<Lifetime> {
        (**self).fold_free_lifetime_var(depth, binders)
    }
}

impl<F1: Folder, F2: Folder> Folder for (F1, F2) {
    fn fold_free_var(&mut self, depth: usize, binders: usize) -> Result<Ty> {
        self.0.fold_free_var(depth, binders)?.fold_with(&mut self.1, binders)
    }

    fn fold_free_lifetime_var(&mut self, depth: usize, binders: usize) -> Result<Lifetime> {
        self.0.fold_free_lifetime_var(depth, binders)?.fold_with(&mut self.1, binders)
    }
}

pub trait Fold {
    type Result;
    fn fold_with(&self, folder: &mut Folder, binders: usize) -> Result<Self::Result>;
}

impl<'a, T: Fold> Fold for &'a T {
    type Result = T::Result;
    fn fold_with(&self, folder: &mut Folder, binders: usize) -> Result<Self::Result> {
        (**self).fold_with(folder, binders)
    }
}

impl<T: Fold> Fold for Vec<T> {
    type Result = Vec<T::Result>;
    fn fold_with(&self, folder: &mut Folder, binders: usize) -> Result<Self::Result> {
        self.iter().map(|e| e.fold_with(folder, binders)).collect()
    }
}

impl<T: Fold> Fold for Box<T> {
    type Result = Box<T::Result>;
    fn fold_with(&self, folder: &mut Folder, binders: usize) -> Result<Self::Result> {
        Ok(Box::new((**self).fold_with(folder, binders)?))
    }
}

impl<T: Fold> Fold for Arc<T> {
    type Result = Arc<T::Result>;
    fn fold_with(&self, folder: &mut Folder, binders: usize) -> Result<Self::Result> {
        Ok(Arc::new((**self).fold_with(folder, binders)?))
    }
}

impl<T: Fold, U: Fold> Fold for (T, U) {
    type Result = (T::Result, U::Result);
    fn fold_with(&self, folder: &mut Folder, binders: usize) -> Result<Self::Result> {
        Ok((self.0.fold_with(folder, binders)?, self.1.fold_with(folder, binders)?))
    }
}

impl<T: Fold> Fold for Option<T> {
    type Result = Option<T::Result>;
    fn fold_with(&self, folder: &mut Folder, binders: usize) -> Result<Self::Result> {
        match *self {
            None => Ok(None),
            Some(ref e) => Ok(Some(e.fold_with(folder, binders)?)),
        }
    }
}

impl Fold for Ty {
    type Result = Self;
    fn fold_with(&self, folder: &mut Folder, binders: usize) -> Result<Self::Result> {
        match *self {
            Ty::Var(depth) => if depth >= binders {
                folder.fold_free_var(depth - binders, binders)
            } else {
                Ok(Ty::Var(depth))
            },
            Ty::Apply(ref apply) => Ok(Ty::Apply(apply.fold_with(folder, binders)?)),
            Ty::Projection(ref proj) => {
                Ok(Ty::Projection(proj.fold_with(folder, binders)?))
            }
            Ty::ForAll(ref quantified_ty) => {
                Ok(Ty::ForAll(quantified_ty.fold_with(folder, binders)?))
            }
        }
    }
}

impl Fold for QuantifiedTy {
    type Result = Self;
    fn fold_with(&self, folder: &mut Folder, binders: usize) -> Result<Self::Result> {
        let QuantifiedTy { num_binders, ref ty } = *self;
        Ok(QuantifiedTy { num_binders, ty: ty.fold_with(folder, binders + num_binders)? })
    }
}

impl Fold for Lifetime {
    type Result = Self;
    fn fold_with(&self, folder: &mut Folder, binders: usize) -> Result<Self::Result> {
        match *self {
            Lifetime::Var(depth) => if depth >= binders {
                folder.fold_free_lifetime_var(depth - binders, binders)
            } else {
                Ok(Lifetime::Var(depth))
            },
            Lifetime::ForAll(universe) => Ok(Lifetime::ForAll(universe)),
        }
    }
}

macro_rules! copy_fold {
    ($t:ty) => {
        impl Fold for $t {
            type Result = Self;
            fn fold_with(&self,
                         _folder: &mut Folder,
                         _binders: usize)
                         -> Result<Self::Result> {
                Ok(*self)
            }
        }
    }
}

copy_fold!(Identifier);
copy_fold!(UniverseIndex);
copy_fold!(ItemId);
copy_fold!(TypeName);
copy_fold!(usize);

macro_rules! enum_fold {
    ($s:ident [$($n:ident),*] { $($variant:ident($($name:ident),*)),* } $($w:tt)*) => {
        impl<$($n),*> Fold for $s<$($n),*> $($w)* {
            type Result = $s<$($n :: Result),*>;
            fn fold_with(&self, folder: &mut Folder, binders: usize) -> Result<Self::Result> {
                match *self {
                    $(
                        $s::$variant( $(ref $name),* ) => {
                            Ok($s::$variant( $($name.fold_with(folder, binders)?),* ))
                        }
                    )*
                }
            }
        }
    }
}

enum_fold!(ParameterKind[T,L] { Ty(a), Lifetime(a) } where T: Fold, L: Fold);
enum_fold!(WhereClause[] { Implemented(a), Normalize(a) });
enum_fold!(WhereClauseGoal[] { Implemented(a), Normalize(a), UnifyTys(a), WellFormed(a) });
enum_fold!(Constraint[] { LifetimeEq(a, b) });

macro_rules! struct_fold {
    ($s:ident [$($n:ident),*] { $($name:ident),* } $($w:tt)*) => {
        impl<$($n),*> Fold for $s<$($n),*> $($w)* {
            type Result = $s<$($n :: Result),*>;
            fn fold_with(&self, folder: &mut Folder, binders: usize) -> Result<Self::Result> {
                Ok($s {
                    $($name: self.$name.fold_with(folder, binders)?),*
                })
            }
        }
    }
}

struct_fold!(ApplicationTy[] { name, parameters });
struct_fold!(ProjectionTy[] { associated_ty_id, parameters });
struct_fold!(TraitRef[] { trait_id, parameters });
struct_fold!(Normalize[] { projection, ty });
struct_fold!(AssocTyValue[] { name, value });
struct_fold!(Environment[] { universe, clauses });
struct_fold!(InEnvironment[F] { environment, goal } where F: Fold);
struct_fold!(Unify[T] { a, b } where T: Fold);
struct_fold!(Constrained[F] { value, constraints } where F: Fold);
