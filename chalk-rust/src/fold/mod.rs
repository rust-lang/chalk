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

pub trait Fold: Sized {
    fn fold_with(&self, folder: &mut Folder) -> Result<Self>;
}

impl<T: Fold> Fold for Vec<T> {
    fn fold_with(&self, folder: &mut Folder) -> Result<Self> {
        self.iter().map(|e| e.fold_with(folder)).collect()
    }
}

impl<T: Fold> Fold for Option<T> {
    fn fold_with(&self, folder: &mut Folder) -> Result<Self> {
        match *self {
            None => Ok(None),
            Some(ref e) => Ok(Some(e.fold_with(folder)?)),
        }
    }
}

impl Fold for ir::Ty {
    fn fold_with(&self, folder: &mut Folder) -> Result<Self> {
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
    fn fold_with(&self, folder: &mut Folder) -> Result<Self> {
        Ok(ir::ApplicationTy {
            id: self.id.fold_with(folder)?,
            args: self.args.fold_with(folder)?,
        })
    }
}

impl Fold for ir::ProjectionTy {
    fn fold_with(&self, folder: &mut Folder) -> Result<Self> {
        Ok(ir::ProjectionTy {
            trait_ref: self.trait_ref.fold_with(folder)?,
            name: self.name.fold_with(folder)?,
        })
    }
}

impl Fold for ir::TraitRef {
    fn fold_with(&self, folder: &mut Folder) -> Result<Self> {
        Ok(ir::TraitRef {
            trait_id: self.trait_id.fold_with(folder)?,
            args: self.args.fold_with(folder)?,
        })
    }
}

impl Fold for ir::Identifier {
    fn fold_with(&self, _folder: &mut Folder) -> Result<Self> {
        Ok(*self)
    }
}

impl Fold for ir::ItemId {
    fn fold_with(&self, _folder: &mut Folder) -> Result<Self> {
        Ok(*self)
    }
}

impl Fold for ir::WhereClause {
    fn fold_with(&self, folder: &mut Folder) -> Result<Self> {
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
    fn fold_with(&self, folder: &mut Folder) -> Result<Self> {
        Ok(ir::NormalizeTo {
            projection: self.projection.fold_with(folder)?,
            ty: self.ty.fold_with(folder)?,
        })
    }
}
