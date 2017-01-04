use errors::*;
use ir;
use solve::environment;
use solve::infer;
use std::sync::Arc;

pub trait Folder {
    fn fold_var(&mut self, depth: usize) -> Result<ir::Ty>;
}

impl<'f, F: Folder> Folder for &'f mut F {
    fn fold_var(&mut self, depth: usize) -> Result<ir::Ty> {
        (**self).fold_var(depth)
    }
}

impl<F1: Folder, F2: Folder> Folder for (F1, F2) {
    fn fold_var(&mut self, depth: usize) -> Result<ir::Ty> {
        self.0.fold_var(depth)?.fold_with(&mut self.1)
    }
}

pub trait Fold {
    type Result;
    fn fold_with(&self, folder: &mut Folder) -> Result<Self::Result>;
}

macro_rules! struct_fold {
    ($m:ident :: $s:ident { $($name:ident),* }) => {
        impl Fold for $m::$s {
            type Result = Self;
            fn fold_with(&self, folder: &mut Folder) -> Result<Self::Result> {
                Ok($m::$s {
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

impl Fold for ir::Ty {
    type Result = Self;
    fn fold_with(&self, folder: &mut Folder) -> Result<Self::Result> {
        match *self {
            ir::Ty::Var(depth) => folder.fold_var(depth),
            ir::Ty::Apply(ref apply) => Ok(ir::Ty::Apply(apply.fold_with(folder)?)),
            ir::Ty::Projection(ref proj) => {
                Ok(ir::Ty::Projection(proj.fold_with(folder)?))
            }
        }
    }
}

impl Fold for ir::Identifier {
    type Result = Self;
    fn fold_with(&self, _folder: &mut Folder) -> Result<Self::Result> {
        Ok(*self)
    }
}

impl Fold for infer::UniverseIndex {
    type Result = Self;
    fn fold_with(&self, _folder: &mut Folder) -> Result<Self::Result> {
        Ok(*self)
    }
}

impl Fold for ir::ItemId {
    type Result = Self;
    fn fold_with(&self, _folder: &mut Folder) -> Result<Self::Result> {
        Ok(*self)
    }
}

impl Fold for ir::WhereClause {
    type Result = Self;
    fn fold_with(&self, folder: &mut Folder) -> Result<Self::Result> {
        match *self {
            ir::WhereClause::Implemented(ref trait_ref) => {
                Ok(ir::WhereClause::Implemented(trait_ref.fold_with(folder)?))
            }
            ir::WhereClause::NormalizeTo(ref pred) => {
                Ok(ir::WhereClause::NormalizeTo(pred.fold_with(folder)?))
            }
        }
    }
}

impl<F: Fold> Fold for environment::InEnvironment<F> {
    type Result = environment::InEnvironment<F::Result>;
    fn fold_with(&self, folder: &mut Folder) -> Result<Self::Result> {
        Ok(environment::InEnvironment {
            environment: self.environment.fold_with(folder)?,
            goal: self.goal.fold_with(folder)?,
        })
    }
}

struct_fold!(ir::ApplicationTy { id, args });
struct_fold!(ir::ProjectionTy { trait_ref, name });
struct_fold!(ir::TraitRef { trait_id, args });
struct_fold!(ir::NormalizeTo { projection, ty });
struct_fold!(ir::ImplData { parameters, trait_ref, assoc_ty_values });
struct_fold!(ir::AssocTyValue { name, value });
struct_fold!(environment::Environment { universe, clauses });

