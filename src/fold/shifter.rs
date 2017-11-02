use fallible::*;
use ir::*;
use super::{Fold, DefaultTypeFolder, ExistentialFolder, IdentityUniversalFolder};

/// A folder that adjusts debruijn indices by a certain amount.
///
/// Typically, `up_shift` is used when inserting some term T under a
/// binder (or multiple binders). So for example if we had some goal
/// like `T: Trait<?X>`, where `?X` refers to an inference variable,
/// but we wanted to construct a goal like `exists<U> { T = U, T:
/// Trait<?X> }`, we might use `up_shift`.  This is because,
/// internally, the inference variable `?X` (as well as the new
/// quantified variable `U`) are going to be represented by debruijn
/// indices. So if the index of `X` is zero, then while originally we
/// might have had `T: Trait<?0>`, inside the `exists` we want to
/// represent `X` with `?1`, to account for the binder:
///
/// ```notrust
///     exists { T = ?0, T: Trait<?1> }
///                  ^^           ^^ refers to `?X`
///                  refers to `U`
/// ```
pub struct Shifter {
    adjustment: usize,
}

impl Shifter {
    pub fn new(adjustment: usize) -> Shifter {
        Shifter { adjustment }
    }

    pub fn up_shift<T: Fold>(adjustment: usize, value: &T) -> T::Result {
        value.fold_with(&mut Shifter::new(adjustment), 0).unwrap()
    }

    fn adjust(&self, depth: usize, binders: usize) -> usize {
        depth + binders + self.adjustment
    }
}

macro_rules! shift_method {
    ($t:ty) => {
        impl $t {
            /// See `Shifter`.
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

shift_method!(Ty);
shift_method!(Parameter);
shift_method!(Lifetime);
shift_method!(TraitRef);
shift_method!(ProjectionTy);
shift_method!(DomainGoal);

impl DefaultTypeFolder for Shifter { }

impl ExistentialFolder for Shifter {
    fn fold_free_existential_ty(&mut self, depth: usize, binders: usize) -> Fallible<Ty> {
        Ok(Ty::Var(self.adjust(depth, binders)))
    }

    fn fold_free_existential_lifetime(&mut self, depth: usize, binders: usize) -> Fallible<Lifetime> {
        Ok(Lifetime::Var(self.adjust(depth, binders)))
    }
}

impl IdentityUniversalFolder for Shifter { }
