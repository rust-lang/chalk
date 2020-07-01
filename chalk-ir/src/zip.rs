//! Traits for "zipping" types, walking through two structures and checking that they match.

use crate::fold::Fold;
use crate::*;
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
pub trait Zipper<'i, I: Interner> {
    /// Indicates that the two types `a` and `b` were found in matching spots.
    fn zip_tys(&mut self, a: &Ty<I>, b: &Ty<I>) -> Fallible<()>;

    /// Indicates that the two lifetimes `a` and `b` were found in matching spots.
    fn zip_lifetimes(&mut self, a: &Lifetime<I>, b: &Lifetime<I>) -> Fallible<()>;

    /// Indicates that the two consts `a` and `b` were found in matching spots.
    fn zip_consts(&mut self, a: &Const<I>, b: &Const<I>) -> Fallible<()>;

    /// Zips two values appearing beneath binders.
    fn zip_binders<T>(&mut self, a: &Binders<T>, b: &Binders<T>) -> Fallible<()>
    where
        T: HasInterner<Interner = I> + Zip<I> + Fold<I, I, Result = T>;

    /// Retreives the interner from the underlying zipper object
    fn interner(&self) -> &'i I;
}

impl<'f, 'i, Z, I> Zipper<'i, I> for &'f mut Z
where
    I: Interner,
    Z: Zipper<'i, I>,
{
    fn zip_tys(&mut self, a: &Ty<I>, b: &Ty<I>) -> Fallible<()> {
        (**self).zip_tys(a, b)
    }

    fn zip_lifetimes(&mut self, a: &Lifetime<I>, b: &Lifetime<I>) -> Fallible<()> {
        (**self).zip_lifetimes(a, b)
    }

    fn zip_consts(&mut self, a: &Const<I>, b: &Const<I>) -> Fallible<()> {
        (**self).zip_consts(a, b)
    }

    fn zip_binders<T>(&mut self, a: &Binders<T>, b: &Binders<T>) -> Fallible<()>
    where
        T: HasInterner<Interner = I> + Zip<I> + Fold<I, I, Result = T>,
    {
        (**self).zip_binders(a, b)
    }

    fn interner(&self) -> &'i I {
        Z::interner(*self)
    }
}

/// The `Zip` trait walks two values, invoking the `Zipper` methods where
/// appropriate, but otherwise requiring strict equality.
///
/// See `Zipper` trait for more details.
///
/// To implement the trait, typically you would use one of the macros
/// like `eq_zip!`, `struct_zip!`, or `enum_zip!`.
pub trait Zip<I>: Debug
where
    I: Interner,
{
    /// Uses the zipper to walk through two values, ensuring that they match.
    fn zip_with<'i, Z: Zipper<'i, I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
    where
        I: 'i;
}

impl<'a, T: ?Sized + Zip<I>, I: Interner> Zip<I> for &'a T {
    fn zip_with<'i, Z: Zipper<'i, I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
    where
        I: 'i,
    {
        <T as Zip<I>>::zip_with(zipper, a, b)
    }
}

impl<I: Interner> Zip<I> for () {
    fn zip_with<'i, Z: Zipper<'i, I>>(_: &mut Z, _: &Self, _: &Self) -> Fallible<()> {
        Ok(())
    }
}

impl<T: Zip<I>, I: Interner> Zip<I> for Vec<T> {
    fn zip_with<'i, Z: Zipper<'i, I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
    where
        I: 'i,
    {
        <[T] as Zip<I>>::zip_with(zipper, a, b)
    }
}

impl<T: Zip<I>, I: Interner> Zip<I> for [T] {
    fn zip_with<'i, Z: Zipper<'i, I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
    where
        I: 'i,
    {
        if a.len() != b.len() {
            return Err(NoSolution);
        }

        for (a_elem, b_elem) in a.iter().zip(b) {
            Zip::zip_with(zipper, a_elem, b_elem)?;
        }

        Ok(())
    }
}

impl<T: Zip<I>, I: Interner> Zip<I> for Arc<T> {
    fn zip_with<'i, Z: Zipper<'i, I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
    where
        I: 'i,
    {
        <T as Zip<I>>::zip_with(zipper, a, b)
    }
}

impl<T: Zip<I>, I: Interner> Zip<I> for Box<T> {
    fn zip_with<'i, Z: Zipper<'i, I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
    where
        I: 'i,
    {
        <T as Zip<I>>::zip_with(zipper, a, b)
    }
}

impl<T: Zip<I>, U: Zip<I>, I: Interner> Zip<I> for (T, U) {
    fn zip_with<'i, Z: Zipper<'i, I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
    where
        I: 'i,
    {
        Zip::zip_with(zipper, &a.0, &b.0)?;
        Zip::zip_with(zipper, &a.1, &b.1)?;
        Ok(())
    }
}

impl<I: Interner> Zip<I> for Ty<I> {
    fn zip_with<'i, Z: Zipper<'i, I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
    where
        I: 'i,
    {
        zipper.zip_tys(a, b)
    }
}

impl<I: Interner> Zip<I> for Lifetime<I> {
    fn zip_with<'i, Z: Zipper<'i, I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
    where
        I: 'i,
    {
        zipper.zip_lifetimes(a, b)
    }
}

impl<I: Interner> Zip<I> for Const<I> {
    fn zip_with<'i, Z: Zipper<'i, I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
    where
        I: 'i,
    {
        zipper.zip_consts(a, b)
    }
}
impl<I: Interner, T: HasInterner<Interner = I> + Zip<I> + Fold<I, I, Result = T>> Zip<I>
    for Binders<T>
{
    fn zip_with<'i, Z: Zipper<'i, I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
    where
        I: 'i,
    {
        zipper.zip_binders(a, b)
    }
}

/// Generates a Zip impl that requires the two values be
/// equal. Suitable for atomic, scalar values.
macro_rules! eq_zip {
    ($I:ident => $t:ty) => {
        impl<$I: Interner> Zip<$I> for $t {
            fn zip_with<'i, Z: Zipper<'i, $I>>(_zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
            where
                I: 'i,
            {
                if a != b {
                    return Err(NoSolution);
                }
                Ok(())
            }
        }
    };
}

eq_zip!(I => AdtId<I>);
eq_zip!(I => TraitId<I>);
eq_zip!(I => AssocTypeId<I>);
eq_zip!(I => OpaqueTyId<I>);
eq_zip!(I => TypeName<I>);
eq_zip!(I => QuantifierKind);
eq_zip!(I => PhantomData<I>);
eq_zip!(I => PlaceholderIndex);
eq_zip!(I => ClausePriority);

impl<T: HasInterner<Interner = I> + Zip<I>, I: Interner> Zip<I> for InEnvironment<T> {
    fn zip_with<'i, Z: Zipper<'i, I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
    where
        I: 'i,
    {
        Zip::zip_with(zipper, &a.environment, &b.environment)?;
        Zip::zip_with(zipper, &a.goal, &b.goal)?;
        Ok(())
    }
}

impl<I: Interner> Zip<I> for Environment<I> {
    fn zip_with<'i, Z: Zipper<'i, I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
    where
        I: 'i,
    {
        let interner = zipper.interner();
        assert_eq!(a.clauses.len(interner), b.clauses.len(interner)); // or different numbers of clauses
        Zip::zip_with(
            zipper,
            a.clauses.as_slice(interner),
            b.clauses.as_slice(interner),
        )?;
        Ok(())
    }
}

impl<I: Interner> Zip<I> for Goals<I> {
    fn zip_with<'i, Z: Zipper<'i, I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
    where
        I: 'i,
    {
        let interner = zipper.interner();
        Zip::zip_with(zipper, a.as_slice(interner), b.as_slice(interner))?;
        Ok(())
    }
}

impl<I: Interner> Zip<I> for ProgramClauses<I> {
    fn zip_with<'i, Z: Zipper<'i, I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
    where
        I: 'i,
    {
        let interner = zipper.interner();
        Zip::zip_with(zipper, a.as_slice(interner), b.as_slice(interner))?;
        Ok(())
    }
}

impl<I: Interner> Zip<I> for Constraints<I> {
    fn zip_with<'i, Z: Zipper<'i, I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
    where
        I: 'i,
    {
        let interner = zipper.interner();
        Zip::zip_with(zipper, a.as_slice(interner), b.as_slice(interner))?;
        Ok(())
    }
}

impl<I: Interner> Zip<I> for QuantifiedWhereClauses<I> {
    fn zip_with<'i, Z: Zipper<'i, I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
    where
        I: 'i,
    {
        let interner = zipper.interner();
        Zip::zip_with(zipper, a.as_slice(interner), b.as_slice(interner))?;
        Ok(())
    }
}

impl<I: Interner> Zip<I> for Substitution<I> {
    fn zip_with<'i, Z: Zipper<'i, I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
    where
        I: 'i,
    {
        let interner = zipper.interner();
        Zip::zip_with(zipper, a.as_slice(interner), b.as_slice(interner))
    }
}

// Annoyingly, Goal cannot use `enum_zip` because some variants have
// two parameters, and I'm too lazy to make the macro account for the
// relevant name mangling.
impl<I: Interner> Zip<I> for Goal<I> {
    fn zip_with<'i, Z: Zipper<'i, I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
    where
        I: 'i,
    {
        let interner = zipper.interner();
        Zip::zip_with(zipper, a.data(interner), b.data(interner))
    }
}

// I'm too lazy to make `enum_zip` support type parameters.
impl<I: Interner> Zip<I> for VariableKind<I> {
    fn zip_with<'i, Z: Zipper<'i, I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
    where
        I: 'i,
    {
        match (a, b) {
            (VariableKind::Ty(a), VariableKind::Ty(b)) if a == b => Ok(()),
            (VariableKind::Lifetime, VariableKind::Lifetime) => Ok(()),
            (VariableKind::Const(ty_a), VariableKind::Const(ty_b)) => {
                Zip::zip_with(zipper, ty_a, ty_b)
            }
            (VariableKind::Ty(_), _)
            | (VariableKind::Lifetime, _)
            | (VariableKind::Const(_), _) => panic!("zipping things of mixed kind"),
        }
    }
}

impl<I: Interner> Zip<I> for GenericArg<I> {
    fn zip_with<'i, Z: Zipper<'i, I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
    where
        I: 'i,
    {
        let interner = zipper.interner();
        Zip::zip_with(zipper, a.data(interner), b.data(interner))
    }
}

impl<I: Interner> Zip<I> for ProgramClause<I> {
    fn zip_with<'i, Z: Zipper<'i, I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
    where
        I: 'i,
    {
        let interner = zipper.interner();
        Zip::zip_with(zipper, a.data(interner), b.data(interner))
    }
}
