use errors::*;
use ir::*;
use solve::environment::{Environment, InEnvironment};
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
            (&ParameterKind::Ty(_), &ParameterKind::Lifetime(_)) |
            (&ParameterKind::Lifetime(_), &ParameterKind::Ty(_)) => {
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
eq_zip!(CrateId);
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
struct_zip!(LocalTo[T] { value, crate_id } where T: Zip);

impl Zip for Environment {
    fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
        assert_eq!(a.universe, b.universe); // it's wrong to zip 2 env's with distinct universes!
        assert_eq!(a.clauses.len(), b.clauses.len()); // or different numbers of clauses
        Zip::zip_with(zipper, &a.clauses, &b.clauses)?;
        Ok(())
    }
}

impl Zip for WhereClause {
    fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
        match (a, b) {
            (&WhereClause::Implemented(ref a), &WhereClause::Implemented(ref b)) => {
                Zip::zip_with(zipper, a, b)
            }
            (&WhereClause::Normalize(ref a), &WhereClause::Normalize(ref b)) => {
                Zip::zip_with(zipper, a, b)
            }
            (&WhereClause::Implemented(_), &WhereClause::Normalize(_)) |
            (&WhereClause::Normalize(_), &WhereClause::Implemented(_)) => {
                bail!("cannot zip where-clauses `{:?}` and `{:?}`", a, b)
            }
        }
    }
}

impl Zip for WhereClauseGoal {
    fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
        match (a, b) {
            (&WhereClauseGoal::Implemented(ref a), &WhereClauseGoal::Implemented(ref b)) => {
                Zip::zip_with(zipper, a, b)
            }
            (&WhereClauseGoal::Normalize(ref a), &WhereClauseGoal::Normalize(ref b)) => {
                Zip::zip_with(zipper, a, b)
            }
            (&WhereClauseGoal::UnifyTys(ref a), &WhereClauseGoal::UnifyTys(ref b)) => {
                Zip::zip_with(zipper, a, b)
            }
            (&WhereClauseGoal::WellFormed(ref a), &WhereClauseGoal::WellFormed(ref b)) => {
                Zip::zip_with(zipper, a, b)
            }
            (&WhereClauseGoal::TyLocalTo(ref a), &WhereClauseGoal::TyLocalTo(ref b)) => {
                Zip::zip_with(zipper, a, b)
            }
            (&WhereClauseGoal::Implemented(_), _) |
            (&WhereClauseGoal::Normalize(_), _) |
            (&WhereClauseGoal::UnifyTys(_), _) |
            (&WhereClauseGoal::TyLocalTo(_), _) |
            (&WhereClauseGoal::WellFormed(_), _) => {
                bail!("cannot zip where-clause-goals `{:?}` and `{:?}`", a, b)
            }
        }
    }
}

impl Zip for WellFormed {
    fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
        match (a, b) {
            (&WellFormed::Ty(ref a), &WellFormed::Ty(ref b)) => {
                Zip::zip_with(zipper, a, b)
            }
            (&WellFormed::TraitRef(ref a), &WellFormed::TraitRef(ref b)) => {
                Zip::zip_with(zipper, a, b)
            }
            (&WellFormed::Ty(_), _) |
            (&WellFormed::TraitRef(_), _) => {
                bail!("cannot zip `{:?}` and `{:?}`", a, b)
            }
        }
    }
}


impl<T: Zip> Zip for Unify<T> {
    fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
        let Unify { a: ref a_a, b: ref a_b } = *a;
        let Unify { a: ref b_a, b: ref b_b } = *b;
        Zip::zip_with(zipper, &a_a, &b_a)?;
        Zip::zip_with(zipper, &a_b, &b_b)?;
        Ok(())
    }
}
