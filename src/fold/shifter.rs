use errors::*;
use ir::*;
use super::{Fold, Folder};

pub struct Shifter {
    adjustment: usize
}

impl Shifter {
    pub fn new(adjustment: usize) -> Shifter {
        Shifter { adjustment }
    }

    pub fn up_shift<T: Fold>(adjustment: usize, value: &T) -> T::Result {
        value.fold_with(&mut Shifter::new(adjustment), 0).unwrap()
    }
}

macro_rules! up_shift_method {
    ($t:ty) => {
        impl $t {
            pub fn up_shift(&self, adjustment: usize) -> Self {
                if adjustment == 0 {
                    self.clone()
                } else {
                    Shifter::up_shift(adjustment, self)
                }
            }
        }
    }
}

up_shift_method!(Ty);
up_shift_method!(Parameter);
up_shift_method!(Lifetime);
up_shift_method!(TraitRef);

impl Folder for Shifter {
    fn fold_free_var(&mut self, depth: usize, binders: usize) -> Result<Ty> {
        Ok(Ty::Var(depth + self.adjustment + binders))
    }

    fn fold_free_lifetime_var(&mut self, depth: usize, binders: usize) -> Result<Lifetime> {
        Ok(Lifetime::Var(depth + self.adjustment + binders))
    }
}
