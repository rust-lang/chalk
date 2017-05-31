use errors::*;
use ir::*;
use std::sync::Arc;

pub trait Zipper {
    fn zip_tys(&mut self, a: &Ty, b: &Ty) -> Result<()>;
    fn zip_lifetimes(&mut self, a: &Lifetime, b: &Lifetime) -> Result<()>;
}

impl<'f, Z: Zipper> Zipper for &'f mut Z {
    fn zip_tys(&mut self, a: &Ty, b: &Ty) -> Result<()> {
        (**self).zip_tys(a, b)
    }

    fn zip_lifetimes(&mut self, a: &Lifetime, b: &Lifetime) -> Result<()> {
        (**self).zip_lifetimes(a, b)
    }
}

pub trait Zip {
    fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()>;
}

impl<'a, T: ?Sized + Zip> Zip for &'a T {
    fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
        <T as Zip>::zip_with(zipper, a, b)
    }
}

impl<T: Zip> Zip for Vec<T> {
    fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
        <[T] as Zip>::zip_with(zipper, a, b)
    }
}

impl<T: Zip> Zip for [T] {
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
        <T as Zip>::zip_with(zipper, a, b)
    }
}

impl<T: Zip, U: Zip> Zip for (T, U) {
    fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
        Zip::zip_with(zipper, &a.0, &b.0)?;
        Zip::zip_with(zipper, &a.1, &b.1)?;
        Ok(())
    }
}

impl<T: Zip, L: Zip> Zip for ParameterKind<T, L> {
    fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
        match (a, b) {
            (&ParameterKind::Ty(ref a), &ParameterKind::Ty(ref b)) => Zip::zip_with(zipper, a, b),
            (&ParameterKind::Lifetime(ref a), &ParameterKind::Lifetime(ref b)) => Zip::zip_with(zipper, a, b),
            (&ParameterKind::Ty(_), _) |
            (&ParameterKind::Lifetime(_), _) => {
                panic!("zipping things of mixed kind")
            }
        }
    }
}

impl Zip for Ty {
    fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
        zipper.zip_tys(a, b)
    }
}

impl Zip for Lifetime {
    fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
        zipper.zip_lifetimes(a, b)
    }
}

macro_rules! eq_zip {
    ($t:ty) => {
        impl Zip for $t {
            fn zip_with<Z: Zipper>(_zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
                if a != b {
                    bail!("cannot zip `{:?}` and `{:?}`", a, b)
                }
                Ok(())
            }
        }
    }
}

eq_zip!(ItemId);
eq_zip!(TypeName);
eq_zip!(Identifier);

macro_rules! struct_zip {
    ($t:ident$([$($param:tt)*])* { $($field:ident),* } $($w:tt)*) => {
        impl$(<$($param)*>)* Zip for $t $(<$($param)*>)* $($w)* {
            fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
                // Validate that we have indeed listed all fields
                let $t { $($field: _),* } = *a;
                $(
                    Zip::zip_with(zipper, &a.$field, &b.$field)?;
                )*
                Ok(())
            }
        }
    }
}

struct_zip!(TraitRef { trait_id, parameters });
struct_zip!(InEnvironment[T] { environment, goal } where T: Zip);
struct_zip!(ApplicationTy { name, parameters });
struct_zip!(ProjectionTy { associated_ty_id, parameters });
struct_zip!(Normalize { projection, ty });
struct_zip!(EqGoal { a, b });

impl Zip for Environment {
    fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
        assert_eq!(a.universe, b.universe); // it's wrong to zip 2 env's with distinct universes!
        assert_eq!(a.clauses.len(), b.clauses.len()); // or different numbers of clauses
        Zip::zip_with(zipper, &a.clauses, &b.clauses)?;
        Ok(())
    }
}

macro_rules! enum_zip {
    ($t:ident$([$($param:tt)*])* { $( $variant:ident ),* } $($w:tt)*) => {
        impl$(<$($param)*>)* Zip for $t $(<$($param)*>)* $($w)* {
            fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
                match (a, b) {
                    $(
                        (&$t :: $variant (ref f_a), &$t :: $variant (ref f_b)) => {
                            Zip::zip_with(zipper, f_a, f_b)
                        }
                    )*

                    $((&$t :: $variant ( .. ), _))|* => {
                        bail!("cannot zip `{:?}` and `{:?}`", a, b)
                    }
                }
            }
        }
    }
}

enum_zip!(DomainGoal { Implemented, KnownProjection, Normalize, WellFormed });
enum_zip!(LeafGoal { DomainGoal, EqGoal });
enum_zip!(WellFormed { Ty, TraitRef });
