use errors::*;
use ir;
use solve::environment;
use solve::infer;
use std::sync::Arc;

pub trait Zipper {
    fn zip_tys(&mut self, a: &ir::Ty, b: &ir::Ty) -> Result<()>;
    fn zip_item_ids(&mut self, a: ir::ItemId, b: ir::ItemId) -> Result<()>;
}

impl<'f, Z: Zipper> Zipper for &'f mut Z {
    fn zip_tys(&mut self, a: &ir::Ty, b: &ir::Ty) -> Result<()> {
        (**self).zip_tys(a, b)
    }

    fn zip_item_ids(&mut self, a: ir::ItemId, b: ir::ItemId) -> Result<()> {
        (**self).zip_item_ids(a, b)
    }
}

pub trait Zip {
    fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()>;
}

macro_rules! struct_zip {
    ($m:ident :: $s:ident { $($name:ident),* }) => {
        impl Zip for $m::$s {
            fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
                $(
                    Zip::zip_with(zipper, &a.$name, &b.$name)?;
                )*
                Ok(())
            }
        }
    }
}

impl<'a, T: Zip> Zip for &'a T {
    fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
        Zip::zip_with(zipper, &**a, &**b)
    }
}

impl<T: Zip> Zip for Vec<T> {
    fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
        if a.len() != b.len() {
            bail!("cannot zip arrays of different lengths: {} vs {}",
                  a.len(), b.len());
        }

        for (a_elem, b_elem) in a.iter().zip(b) {
            Zip::zip_with(zipper, a_elem, b_elem)?;
        }

        Ok(())
    }
}

impl<T: Zip> Zip for Arc<T> {
    fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
        Zip::zip_with(zipper, &**a, &**b)
    }
}

impl<T: Zip, U: Zip> Zip for (T, U) {
    fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
        Zip::zip_with(zipper, &a.0, &b.0)?;
        Zip::zip_with(zipper, &a.1, &b.1)?;
        Ok(())
    }
}

impl Zip for ir::Ty {
    fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
        zipper.zip_tys(a, b)
    }
}

impl Zip for ir::ItemId {
    fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
        zipper.zip_item_ids(*a, *b)
    }
}

impl Zip for ir::TraitRef {
    fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
        Zip::zip_with(zipper, &a.trait_id, &b.trait_id)?;
        Zip::zip_with(zipper, &a.args, &b.args)?;
        Ok(())
    }
}

impl Zip for ir::ApplicationTy {
    fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
        Zip::zip_with(zipper, &a.id, &b.id)?;
        Zip::zip_with(zipper, &a.args, &b.args)?;
        Ok(())
    }
}
