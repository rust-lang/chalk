use errors::*;
use ir;

pub trait Fold: Sized {
    fn fold(&self, folder: &mut Folder) -> Result<Self>;
}

pub trait Folder {
    fn fold_var(&mut self, depth: usize) -> Result<ir::Ty>;
}

impl Fold for ir::Ty {
    fn fold(&self, folder: &mut Folder) -> Result<Self> {
        match *self {
            ir::Ty::Var { depth } => folder.fold_var(depth),
            ir::Ty::Apply { id, ref args } => {
                Ok(ir::Ty::Apply {
                    id: id.fold(folder)?,
                    args: try!(args.iter().map(|a| a.fold(folder)).collect()),
                })
            }
            ir::Ty::Projection { ref proj } => Ok(ir::Ty::Projection { proj: proj.fold(folder)? }),
        }
    }
}

impl Fold for ir::ProjectionTy {
    fn fold(&self, folder: &mut Folder) -> Result<Self> {
        Ok(ir::ProjectionTy {
            trait_ref: self.trait_ref.fold(folder)?,
            name: self.name.fold(folder)?,
        })
    }
}

impl Fold for ir::TraitRef {
    fn fold(&self, folder: &mut Folder) -> Result<Self> {
        Ok(ir::TraitRef {
            trait_id: self.trait_id.fold(folder)?,
            args: try!(self.args.iter().map(|a| a.fold(folder)).collect()),
        })
    }
}

impl Fold for ir::Identifier {
    fn fold(&self, _folder: &mut Folder) -> Result<Self> {
        Ok(*self)
    }
}

impl Fold for ir::ItemId {
    fn fold(&self, _folder: &mut Folder) -> Result<Self> {
        Ok(*self)
    }
}

impl Fold for ir::WhereClause {
    fn fold(&self, folder: &mut Folder) -> Result<Self> {
        match *self {
            ir::WhereClause::Implemented { ref trait_ref } => {
                Ok(ir::WhereClause::Implemented { trait_ref: trait_ref.fold(folder)? })
            }
            ir::WhereClause::ProjectionEq { ref projection, ref ty } => {
                Ok(ir::WhereClause::ProjectionEq {
                    projection: projection.fold(folder)?,
                    ty: ty.fold(folder)?,
                })
            }
        }
    }
}
