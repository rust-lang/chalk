use super::*;

pub trait Zip: Sized {
    fn zip_with<Z: Zipper>(&self, other: &Self, zipper: &mut Z) -> Result<Self, ZipError>;
}

pub enum ZipError {
    UnequalConstantArity { left: (Constant, usize), right: (Constant, usize) },
}

pub trait Zipper {
    fn zip_leaves(&mut self, leaf1: &Leaf, leaf2: &Leaf) -> Result<Leaf, ZipError>;
}

impl<T: Zip> Zip for Vec<T> {
    fn zip_with<Z: Zipper>(&self, other: &Self, zipper: &mut Z) -> Result<Self, ZipError> {
        assert_eq!(self.len(), other.len());
        self.iter()
            .zip(other)
            .map(|(a, b)| a.zip_with(b, zipper))
            .collect()
    }
}

impl<T: Zip> Zip for Option<T> {
    fn zip_with<Z: Zipper>(&self, other: &Self, zipper: &mut Z) -> Result<Self, ZipError> {
        assert_eq!(self.is_some(), other.is_some());
        match self.as_ref() {
            Some(a) => {
                let c = a.zip_with(other.as_ref().unwrap(), zipper)?;
                Ok(Some(c))
            }
            None => Ok(None),
        }
    }
}

impl Zip for Application {
    fn zip_with<Z: Zipper>(&self, other: &Self, zipper: &mut Z) -> Result<Self, ZipError> {
        if self.constant_and_arity() != other.constant_and_arity() {
            return Err(ZipError::UnequalConstantArity {
                left: self.constant_and_arity(),
                right: other.constant_and_arity()
            });
        }

        Ok(Application {
            constant: self.constant,
            args: self.args.zip_with(&other.args, zipper)?
        })
    }
}

impl Zip for Leaf {
    fn zip_with<Z: Zipper>(&self, other: &Self, zipper: &mut Z) -> Result<Self, ZipError> {
        zipper.zip_leaves(self, other)
    }
}
