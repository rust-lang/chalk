use errors::*;
use fold::Fold;
use ir::*;
use std::fmt::Debug;
use std::sync::Arc;

/// When we zip types, we basically traverse the structure, ensuring
/// that it matches.  When we come to types/lifetimes, we invoke the
/// callback methods in the zipper to match them up. Primarily used
/// during unification or similar operations.
///
/// So e.g. if you had `A: Eq<B>` zipped with `X: Eq<Y>`, then the zipper
/// would get two callbacks, one pairing `A` and `X`, and the other pairing
/// `B` and `Y`.
///
/// For things other than types/lifetimes, the zip impls will
/// guarantee equality. So e.g. if you have `A: Eq<B>` zipped with `X:
/// Ord<Y>`, you would wind up with an error, no matter what zipper
/// you are using. This is because the traits `Eq` and `Ord` are
/// represented by two distinct `ItemId` values, and the impl for
/// `ItemId` requires that all `ItemId` in the two zipped values match
/// up.
pub trait Zipper {
    /// Indicates that the two types `a` and `b` were found in
    /// matching spots, beneath `binders` levels of binders.
    fn zip_tys(&mut self, a: &Ty, b: &Ty) -> Result<()>;

    /// Indicates that the two lifetimes `a` and `b` were found in
    /// matching spots, beneath `binders` levels of binders.
    fn zip_lifetimes(&mut self, a: &Lifetime, b: &Lifetime) -> Result<()>;

    /// Zips two values appearing beneath binders.
    fn zip_binders<T>(&mut self, a: &Binders<T>, b: &Binders<T>) -> Result<()>
        where T: Zip + Fold<Result = T>;
}

impl<'f, Z: Zipper> Zipper for &'f mut Z {
    fn zip_tys(&mut self, a: &Ty, b: &Ty) -> Result<()> {
        (**self).zip_tys(a, b)
    }

    fn zip_lifetimes(&mut self, a: &Lifetime, b: &Lifetime) -> Result<()> {
        (**self).zip_lifetimes(a, b)
    }

    fn zip_binders<T>(&mut self, a: &Binders<T>, b: &Binders<T>) -> Result<()>
        where T: Zip + Fold<Result = T>
    {
        (**self).zip_binders(a, b)
    }
}

/// The `Zip` trait walks two values, invoking the `Zipper` methods where
/// appropriate, but otherwise requiring strict equality.
///
/// See `Zipper` trait for more details.
///
/// To implement the trait, typically you would use one of the macros
/// like `eq_zip!`, `struct_zip!`, or `enum_zip!`.
pub trait Zip: Debug {
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

impl<T: Zip> Zip for Box<T> {
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

impl<T: Zip + Fold<Result = T>> Zip for Binders<T> {
    fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
        zipper.zip_binders(a, b)
    }
}

/// Generates a Zip impl that requires the two values be
/// equal. Suitable for atomic, scalar values.
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
eq_zip!(QuantifierKind);

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

/// Generates a Zip impl that zips each field of the struct in turn.
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

/// Generates a Zip impl that requires the two enums be the same
/// variant, then zips each field of the variant in turn. Only works
/// if all variants have a single parenthesized value right now.
enum_zip!(PolarizedTraitRef { Positive, Negative });
enum_zip!(DomainGoal { Implemented, Normalize, WellFormed });
enum_zip!(LeafGoal { DomainGoal, EqGoal });
enum_zip!(WellFormed { Ty, TraitRef });

// Annoyingly, Goal cannot use `enum_zip` because some variants have
// two parameters, and I'm too lazy to make the macro account for the
// relevant name mangling.
impl Zip for Goal {
    fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
        match (a, b) {
            (&Goal::Quantified(ref f_a, ref g_a), &Goal::Quantified(ref f_b, ref g_b)) => {
                Zip::zip_with(zipper, f_a, f_b)?;
                Zip::zip_with(zipper, g_a, g_b)
            }
            (&Goal::Implies(ref f_a, ref g_a), &Goal::Implies(ref f_b, ref g_b)) => {
                Zip::zip_with(zipper, f_a, f_b)?;
                Zip::zip_with(zipper, g_a, g_b)
            }
            (&Goal::And(ref f_a, ref g_a), &Goal::And(ref f_b, ref g_b)) => {
                Zip::zip_with(zipper, f_a, f_b)?;
                Zip::zip_with(zipper, g_a, g_b)
            }
            (&Goal::Not(ref f_a), &Goal::Not(ref f_b)) => {
                Zip::zip_with(zipper, f_a, f_b)
            }
            (&Goal::Leaf(ref f_a), &Goal::Leaf(ref f_b)) => {
                Zip::zip_with(zipper, f_a, f_b)
            }
            (&Goal::Quantified(..), _) |
            (&Goal::Implies(..), _) |
            (&Goal::And(..), _) |
            (&Goal::Not(..), _) |
            (&Goal::Leaf(..), _) => {
                bail!("cannot zip `{:?}` and `{:?}`", a, b)
            }
        }
    }
}

// I'm too lazy to make `enum_zip` support type parameters.
impl<T: Zip, L: Zip> Zip for ParameterKind<T, L> {
    fn zip_with<Z: Zipper>(zipper: &mut Z, a: &Self, b: &Self) -> Result<()> {
        match (a, b) {
            (&ParameterKind::Ty(ref a), &ParameterKind::Ty(ref b)) =>
                Zip::zip_with(zipper, a, b),
            (&ParameterKind::Lifetime(ref a), &ParameterKind::Lifetime(ref b)) =>
                Zip::zip_with(zipper, a, b),
            (&ParameterKind::Ty(_), _) |
            (&ParameterKind::Lifetime(_), _) => {
                panic!("zipping things of mixed kind")
            }
        }
    }
}
