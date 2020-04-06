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
    /// Indicates that the two types `a` and `b` were found in
    /// matching spots, beneath `binders` levels of binders.
    fn zip_tys(&mut self, a: &Ty<I>, b: &Ty<I>) -> Fallible<()>;

    /// Indicates that the two lifetimes `a` and `b` were found in
    /// matching spots, beneath `binders` levels of binders.
    fn zip_lifetimes(&mut self, a: &Lifetime<I>, b: &Lifetime<I>) -> Fallible<()>;

    /// Zips two values appearing beneath binders.
    fn zip_binders<T>(&mut self, a: &Binders<T>, b: &Binders<T>) -> Fallible<()>
    where
        T: Zip<I> + Fold<I, I, Result = T>;

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

    fn zip_binders<T>(&mut self, a: &Binders<T>, b: &Binders<T>) -> Fallible<()>
    where
        T: Zip<I> + Fold<I, I, Result = T>,
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

impl<I: Interner, T: Zip<I> + Fold<I, I, Result = T>> Zip<I> for Binders<T> {
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

eq_zip!(I => StructId<I>);
eq_zip!(I => TraitId<I>);
eq_zip!(I => AssocTypeId<I>);
eq_zip!(I => TypeName<I>);
eq_zip!(I => QuantifierKind);
eq_zip!(I => PhantomData<I>);
eq_zip!(I => PlaceholderIndex);

/// Generates a Zip impl that zips each field of the struct in turn.
macro_rules! struct_zip {
    (impl[$($param:tt)*] Zip<$I:ty> for $self:ty { $($field:ident),* $(,)* } $($w:tt)*) => {
        impl<$($param)*> Zip<$I> for $self $($w)* {
            fn zip_with<'i, Z: Zipper<'i, $I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
            where
                I: 'i,
            {
                // Validate that we have indeed listed all fields
                let Self { $($field: _),* } = *a;
                $(
                    Zip::zip_with(zipper, &a.$field, &b.$field)?;
                )*
                Ok(())
            }
        }
    }
}

struct_zip!(impl[I: Interner] Zip<I> for TraitRef<I> {
    trait_id,
    substitution,
});
struct_zip!(impl[
    T: HasInterner<Interner = I> + Zip<I>,
    I: Interner,
] Zip<I> for InEnvironment<T> {
    environment,
    goal,
});
struct_zip!(impl[I: Interner] Zip<I> for ApplicationTy<I> { name, substitution });
struct_zip!(impl[I: Interner] Zip<I> for DynTy<I> { bounds });
struct_zip!(impl[I: Interner] Zip<I> for AliasTy<I> {
    associated_ty_id,
    substitution,
});
struct_zip!(impl[I: Interner] Zip<I> for Normalize<I> { alias, ty });
struct_zip!(impl[I: Interner] Zip<I> for AliasEq<I> { alias, ty });
struct_zip!(impl[I: Interner] Zip<I> for EqGoal<I> { a, b });
struct_zip!(impl[I: Interner] Zip<I> for ProgramClauseImplication<I> {
    consequence,
    conditions
});

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

/// Generates a Zip impl that requires the two enums be the same
/// variant, then zips each field of the variant in turn. Only works
/// if all variants have a single parenthesized value right now.
macro_rules! enum_zip {
    (impl<$I:ident $(, $param:ident)*> for $self:ty { $( $variant:ident ),* $(,)* } $($w:tt)*) => {
        impl<$I: Interner, $(, $param)*> Zip<$I> for $self $($w)* {
            fn zip_with<'i, Z: Zipper<'i, $I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
            where
                I: 'i,
            {
                match (a, b) {
                    $(
                        (Self :: $variant (f_a), Self :: $variant (f_b)) => {
                            Zip::zip_with(zipper, f_a, f_b)
                        }
                    )*

                    #[allow(unreachable_patterns)] // needed if there is exactly one variant
                    $((Self :: $variant ( .. ), _))|* => {
                        return Err(NoSolution);
                    }
                }
            }
        }
    }
}

enum_zip!(impl<I> for WellFormed<I> { Trait, Ty });
enum_zip!(impl<I> for FromEnv<I> { Trait, Ty });
enum_zip!(impl<I> for WhereClause<I> { Implemented, AliasEq });
enum_zip!(impl<I> for DomainGoal<I> {
    Holds,
    WellFormed,
    FromEnv,
    Normalize,
    IsLocal,
    IsUpstream,
    IsFullyVisible,
    LocalImplAllowed,
    Compatible,
    DownstreamType
});
enum_zip!(impl<I> for ProgramClauseData<I> { Implies, ForAll });

impl<I: Interner> Zip<I> for Substitution<I> {
    fn zip_with<'i, Z: Zipper<'i, I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
    where
        I: 'i,
    {
        let interner = zipper.interner();
        Zip::zip_with(zipper, a.parameters(interner), b.parameters(interner))
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

impl<I: Interner> Zip<I> for GoalData<I> {
    fn zip_with<'i, Z: Zipper<'i, I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
    where
        I: 'i,
    {
        match (a, b) {
            (&GoalData::Quantified(ref f_a, ref g_a), &GoalData::Quantified(ref f_b, ref g_b)) => {
                Zip::zip_with(zipper, f_a, f_b)?;
                Zip::zip_with(zipper, g_a, g_b)
            }
            (&GoalData::Implies(ref f_a, ref g_a), &GoalData::Implies(ref f_b, ref g_b)) => {
                Zip::zip_with(zipper, f_a, f_b)?;
                Zip::zip_with(zipper, g_a, g_b)
            }
            (&GoalData::All(ref g_a), &GoalData::All(ref g_b)) => Zip::zip_with(zipper, g_a, g_b),
            (&GoalData::Not(ref f_a), &GoalData::Not(ref f_b)) => Zip::zip_with(zipper, f_a, f_b),
            (&GoalData::EqGoal(ref f_a), &GoalData::EqGoal(ref f_b)) => {
                Zip::zip_with(zipper, f_a, f_b)
            }
            (&GoalData::DomainGoal(ref f_a), &GoalData::DomainGoal(ref f_b)) => {
                Zip::zip_with(zipper, f_a, f_b)
            }
            (&GoalData::CannotProve(()), &GoalData::CannotProve(())) => Ok(()),
            (&GoalData::Quantified(..), _)
            | (&GoalData::Implies(..), _)
            | (&GoalData::All(..), _)
            | (&GoalData::Not(..), _)
            | (&GoalData::EqGoal(..), _)
            | (&GoalData::DomainGoal(..), _)
            | (&GoalData::CannotProve(..), _) => {
                return Err(NoSolution);
            }
        }
    }
}

// I'm too lazy to make `enum_zip` support type parameters.
impl<T: Zip<I>, L: Zip<I>, I: Interner> Zip<I> for ParameterKind<T, L> {
    fn zip_with<'i, Z: Zipper<'i, I>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>
    where
        I: 'i,
    {
        match (a, b) {
            (ParameterKind::Ty(a), ParameterKind::Ty(b)) => Zip::zip_with(zipper, a, b),
            (ParameterKind::Lifetime(a), ParameterKind::Lifetime(b)) => Zip::zip_with(zipper, a, b),
            (ParameterKind::Ty(_), _) | (ParameterKind::Lifetime(_), _) => {
                panic!("zipping things of mixed kind")
            }
        }
    }
}

#[allow(unreachable_code, unused_variables)]
impl<I: Interner> Zip<I> for Parameter<I> {
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
