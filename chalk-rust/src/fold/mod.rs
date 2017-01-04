use errors::*;
use ir;

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

impl Fold for ir::ApplicationTy {
    type Result = Self;
    fn fold_with(&self, folder: &mut Folder) -> Result<Self::Result> {
        Ok(ir::ApplicationTy {
            id: self.id.fold_with(folder)?,
            args: self.args.fold_with(folder)?,
        })
    }
}

impl Fold for ir::ProjectionTy {
    type Result = Self;
    fn fold_with(&self, folder: &mut Folder) -> Result<Self::Result> {
        Ok(ir::ProjectionTy {
            trait_ref: self.trait_ref.fold_with(folder)?,
            name: self.name.fold_with(folder)?,
        })
    }
}

impl Fold for ir::TraitRef {
    type Result = Self;
    fn fold_with(&self, folder: &mut Folder) -> Result<Self::Result> {
        Ok(ir::TraitRef {
            trait_id: self.trait_id.fold_with(folder)?,
            args: self.args.fold_with(folder)?,
        })
    }
}

impl Fold for ir::Identifier {
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

impl Fold for ir::NormalizeTo {
    type Result = Self;
    fn fold_with(&self, folder: &mut Folder) -> Result<Self::Result> {
        Ok(ir::NormalizeTo {
            projection: self.projection.fold_with(folder)?,
            ty: self.ty.fold_with(folder)?,
        })
    }
}
