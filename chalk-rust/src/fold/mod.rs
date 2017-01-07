use errors::*;
use ir::*;
use solve::environment::{Environment, InEnvironment};
use std::sync::Arc;

pub trait Folder {
    fn fold_var(&mut self, depth: usize) -> Result<Ty>;
}

impl<'f, F: Folder> Folder for &'f mut F {
    fn fold_var(&mut self, depth: usize) -> Result<Ty> {
        (**self).fold_var(depth)
    }
}

impl<F1: Folder, F2: Folder> Folder for (F1, F2) {
    fn fold_var(&mut self, depth: usize) -> Result<Ty> {
        self.0.fold_var(depth)?.fold_with(&mut self.1)
    }
}

pub trait Fold {
    type Result;
    fn fold_with(&self, folder: &mut Folder) -> Result<Self::Result>;
}

macro_rules! struct_fold {
    ($s:ident { $($name:ident),* }) => {
        impl Fold for $s {
            type Result = Self;
            fn fold_with(&self, folder: &mut Folder) -> Result<Self::Result> {
                Ok($s {
                    $($name: self.$name.fold_with(folder)?),*
                })
            }
        }
    }
}

impl<'a, T: Fold> Fold for &'a T {
    type Result = T::Result;
    fn fold_with(&self, folder: &mut Folder) -> Result<Self::Result> {
        (**self).fold_with(folder)
    }
}

impl<T: Fold> Fold for Vec<T> {
    type Result = Vec<T::Result>;
    fn fold_with(&self, folder: &mut Folder) -> Result<Self::Result> {
        self.iter().map(|e| e.fold_with(folder)).collect()
    }
}

impl<T: Fold> Fold for Arc<T> {
    type Result = Arc<T::Result>;
    fn fold_with(&self, folder: &mut Folder) -> Result<Self::Result> {
        Ok(Arc::new((**self).fold_with(folder)?))
    }
}

impl<T: Fold, U: Fold> Fold for (T, U) {
    type Result = (T::Result, U::Result);
    fn fold_with(&self, folder: &mut Folder) -> Result<Self::Result> {
        Ok((self.0.fold_with(folder)?, self.1.fold_with(folder)?))
    }
}

impl<T: Fold> Fold for Option<T> {
    type Result = Option<T::Result>;
    fn fold_with(&self, folder: &mut Folder) -> Result<Self::Result> {
        match *self {
            None => Ok(None),
            Some(ref e) => Ok(Some(e.fold_with(folder)?)),
        }
    }
}

impl Fold for Parameter {
    type Result = Self;
    fn fold_with(&self, folder: &mut Folder) -> Result<Self::Result> {
        match *self {
            Parameter::Ty(ref t) => Ok(Parameter::Ty(t.fold_with(folder)?)),
        }
    }
}

impl Fold for Ty {
    type Result = Self;
    fn fold_with(&self, folder: &mut Folder) -> Result<Self::Result> {
        match *self {
            Ty::Var(depth) => folder.fold_var(depth),
            Ty::Apply(ref apply) => Ok(Ty::Apply(apply.fold_with(folder)?)),
            Ty::Projection(ref proj) => {
                Ok(Ty::Projection(proj.fold_with(folder)?))
            }
        }
    }
}

impl Fold for Identifier {
    type Result = Self;
    fn fold_with(&self, _folder: &mut Folder) -> Result<Self::Result> {
        Ok(*self)
    }
}

impl Fold for UniverseIndex {
    type Result = Self;
    fn fold_with(&self, _folder: &mut Folder) -> Result<Self::Result> {
        Ok(*self)
    }
}

impl Fold for ItemId {
    type Result = Self;
    fn fold_with(&self, _folder: &mut Folder) -> Result<Self::Result> {
        Ok(*self)
    }
}

impl Fold for TypeName {
    type Result = Self;
    fn fold_with(&self, _folder: &mut Folder) -> Result<Self::Result> {
        Ok(*self)
    }
}

impl Fold for usize {
    type Result = Self;
    fn fold_with(&self, _folder: &mut Folder) -> Result<Self::Result> {
        Ok(*self)
    }
}

impl Fold for ParameterKind {
    type Result = Self;
    fn fold_with(&self, _folder: &mut Folder) -> Result<Self::Result> {
        Ok(self.clone())
    }
}

impl Fold for WhereClause {
    type Result = Self;
    fn fold_with(&self, folder: &mut Folder) -> Result<Self::Result> {
        match *self {
            WhereClause::Implemented(ref trait_ref) => {
                Ok(WhereClause::Implemented(trait_ref.fold_with(folder)?))
            }
            WhereClause::Normalize(ref pred) => {
                Ok(WhereClause::Normalize(pred.fold_with(folder)?))
            }
        }
    }
}

impl<F: Fold> Fold for InEnvironment<F> {
    type Result = InEnvironment<F::Result>;
    fn fold_with(&self, folder: &mut Folder) -> Result<Self::Result> {
        Ok(InEnvironment {
            environment: self.environment.fold_with(folder)?,
            goal: self.goal.fold_with(folder)?,
        })
    }
}

struct_fold!(ApplicationTy { name, parameters });
struct_fold!(ProjectionTy { trait_ref, name });
struct_fold!(TraitRef { trait_id, parameters });
struct_fold!(Normalize { projection, ty });
struct_fold!(ImplData { parameter_kinds, trait_ref, assoc_ty_values, where_clauses });
struct_fold!(AssocTyValue { name, value });
struct_fold!(Environment { universe, clauses });

