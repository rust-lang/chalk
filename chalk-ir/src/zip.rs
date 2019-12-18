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
pub trait Zipper<TF: TypeFamily> {
    /// Indicates that the two types `a` and `b` were found in
    /// matching spots, beneath `binders` levels of binders.
    fn zip_tys(&mut self, a: &Ty<TF>, b: &Ty<TF>) -> Fallible<()>;

    /// Indicates that the two lifetimes `a` and `b` were found in
    /// matching spots, beneath `binders` levels of binders.
    fn zip_lifetimes(&mut self, a: &Lifetime<TF>, b: &Lifetime<TF>) -> Fallible<()>;

    /// Zips two values appearing beneath binders.
    fn zip_binders<T>(&mut self, a: &Binders<T>, b: &Binders<T>) -> Fallible<()>
    where
        T: Zip<TF> + Fold<TF, TF, Result = T>;
}

impl<'f, Z, TF> Zipper<TF> for &'f mut Z
where
    TF: TypeFamily,
    Z: Zipper<TF>,
{
    fn zip_tys(&mut self, a: &Ty<TF>, b: &Ty<TF>) -> Fallible<()> {
        (**self).zip_tys(a, b)
    }

    fn zip_lifetimes(&mut self, a: &Lifetime<TF>, b: &Lifetime<TF>) -> Fallible<()> {
        (**self).zip_lifetimes(a, b)
    }

    fn zip_binders<T>(&mut self, a: &Binders<T>, b: &Binders<T>) -> Fallible<()>
    where
        T: Zip<TF> + Fold<TF, TF, Result = T>,
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
pub trait Zip<TF>: Debug
where
    TF: TypeFamily,
{
    fn zip_with<Z: Zipper<TF>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()>;
}

impl<'a, T: ?Sized + Zip<TF>, TF: TypeFamily> Zip<TF> for &'a T {
    fn zip_with<Z: Zipper<TF>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()> {
        <T as Zip<TF>>::zip_with(zipper, a, b)
    }
}

impl<TF: TypeFamily> Zip<TF> for () {
    fn zip_with<Z: Zipper<TF>>(_: &mut Z, _: &Self, _: &Self) -> Fallible<()> {
        Ok(())
    }
}

impl<T: Zip<TF>, TF: TypeFamily> Zip<TF> for Vec<T> {
    fn zip_with<Z: Zipper<TF>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()> {
        <[T] as Zip<TF>>::zip_with(zipper, a, b)
    }
}

impl<T: Zip<TF>, TF: TypeFamily> Zip<TF> for [T] {
    fn zip_with<Z: Zipper<TF>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()> {
        if a.len() != b.len() {
            return Err(NoSolution);
        }

        for (a_elem, b_elem) in a.iter().zip(b) {
            Zip::zip_with(zipper, a_elem, b_elem)?;
        }

        Ok(())
    }
}

impl<T: Zip<TF>, TF: TypeFamily> Zip<TF> for Arc<T> {
    fn zip_with<Z: Zipper<TF>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()> {
        <T as Zip<TF>>::zip_with(zipper, a, b)
    }
}

impl<T: Zip<TF>, TF: TypeFamily> Zip<TF> for Box<T> {
    fn zip_with<Z: Zipper<TF>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()> {
        <T as Zip<TF>>::zip_with(zipper, a, b)
    }
}

impl<T: Zip<TF>, U: Zip<TF>, TF: TypeFamily> Zip<TF> for (T, U) {
    fn zip_with<Z: Zipper<TF>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()> {
        Zip::zip_with(zipper, &a.0, &b.0)?;
        Zip::zip_with(zipper, &a.1, &b.1)?;
        Ok(())
    }
}

impl<TF: TypeFamily> Zip<TF> for Ty<TF> {
    fn zip_with<Z: Zipper<TF>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()> {
        zipper.zip_tys(a, b)
    }
}

impl<TF: TypeFamily> Zip<TF> for Lifetime<TF> {
    fn zip_with<Z: Zipper<TF>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()> {
        zipper.zip_lifetimes(a, b)
    }
}

impl<TF: TypeFamily, T: Zip<TF> + Fold<TF, TF, Result = T>> Zip<TF> for Binders<T> {
    fn zip_with<Z: Zipper<TF>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()> {
        zipper.zip_binders(a, b)
    }
}

/// Generates a Zip impl that requires the two values be
/// equal. Suitable for atomic, scalar values.
macro_rules! eq_zip {
    ($TF:ident => $t:ty) => {
        impl<$TF: TypeFamily> Zip<$TF> for $t {
            fn zip_with<Z: Zipper<$TF>>(_zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()> {
                if a != b {
                    return Err(NoSolution);
                }
                Ok(())
            }
        }
    };
}

eq_zip!(TF => StructId<TF>);
eq_zip!(TF => TraitId<TF>);
eq_zip!(TF => TypeId<TF>);
eq_zip!(TF => TypeKindId<TF>);
eq_zip!(TF => TypeName<TF>);
eq_zip!(TF => Identifier);
eq_zip!(TF => QuantifierKind);
eq_zip!(TF => PhantomData<TF>);
eq_zip!(TF => PlaceholderIndex);
eq_zip!(TF => PlaceholderTy);

/// Generates a Zip impl that zips each field of the struct in turn.
macro_rules! struct_zip {
    (impl[$($param:tt)*] Zip<$TF:ty> for $self:ty { $($field:ident),* $(,)* } $($w:tt)*) => {
        impl<$($param)*> Zip<$TF> for $self $($w)* {
            fn zip_with<Z: Zipper<$TF>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()> {
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

struct_zip!(impl[TF: TypeFamily] Zip<TF> for TraitRef<TF> {
    trait_id,
    parameters,
});
struct_zip!(impl[
    T: HasTypeFamily<TypeFamily = TF> + Zip<TF>,
    TF: TypeFamily,
] Zip<TF> for InEnvironment<T> {
    environment,
    goal,
});
struct_zip!(impl[TF: TypeFamily] Zip<TF> for ApplicationTy<TF> { name, parameters });
struct_zip!(impl[TF: TypeFamily] Zip<TF> for ProjectionTy<TF> {
    associated_ty_id,
    parameters,
});
struct_zip!(impl[TF: TypeFamily] Zip<TF> for Normalize<TF> { projection, ty });
struct_zip!(impl[TF: TypeFamily] Zip<TF> for ProjectionEq<TF> { projection, ty });
struct_zip!(impl[TF: TypeFamily] Zip<TF> for EqGoal<TF> { a, b });
struct_zip!(impl[TF: TypeFamily] Zip<TF> for ProgramClauseImplication<TF> {
    consequence,
    conditions
});

impl<TF: TypeFamily> Zip<TF> for Environment<TF> {
    fn zip_with<Z: Zipper<TF>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()> {
        assert_eq!(a.clauses.len(), b.clauses.len()); // or different numbers of clauses
        Zip::zip_with(zipper, &a.clauses, &b.clauses)?;
        Ok(())
    }
}

/// Generates a Zip impl that requires the two enums be the same
/// variant, then zips each field of the variant in turn. Only works
/// if all variants have a single parenthesized value right now.
macro_rules! enum_zip {
    (impl<$TF:ident $(, $param:ident)*> for $self:ty { $( $variant:ident ),* $(,)* } $($w:tt)*) => {
        impl<$TF: TypeFamily, $(, $param)*> Zip<$TF> for $self $($w)* {
            fn zip_with<Z: Zipper<$TF>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()> {
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

enum_zip!(impl<TF> for WellFormed<TF> { Trait, Ty });
enum_zip!(impl<TF> for FromEnv<TF> { Trait, Ty });
enum_zip!(impl<TF> for WhereClause<TF> { Implemented, ProjectionEq });
enum_zip!(impl<TF> for DomainGoal<TF> {
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
enum_zip!(impl<TF> for LeafGoal<TF> { DomainGoal, EqGoal });
enum_zip!(impl<TF> for ProgramClause<TF> { Implies, ForAll });

// Annoyingly, Goal cannot use `enum_zip` because some variants have
// two parameters, and I'm too lazy to make the macro account for the
// relevant name mangling.
impl<TF: TypeFamily> Zip<TF> for Goal<TF> {
    fn zip_with<Z: Zipper<TF>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()> {
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
            (&Goal::Not(ref f_a), &Goal::Not(ref f_b)) => Zip::zip_with(zipper, f_a, f_b),
            (&Goal::Leaf(ref f_a), &Goal::Leaf(ref f_b)) => Zip::zip_with(zipper, f_a, f_b),
            (&Goal::CannotProve(()), &Goal::CannotProve(())) => Ok(()),
            (&Goal::Quantified(..), _)
            | (&Goal::Implies(..), _)
            | (&Goal::And(..), _)
            | (&Goal::Not(..), _)
            | (&Goal::Leaf(..), _)
            | (&Goal::CannotProve(..), _) => {
                return Err(NoSolution);
            }
        }
    }
}

// I'm too lazy to make `enum_zip` support type parameters.
impl<T: Zip<TF>, L: Zip<TF>, TF: TypeFamily> Zip<TF> for ParameterKind<T, L> {
    fn zip_with<Z: Zipper<TF>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()> {
        match (a, b) {
            (ParameterKind::Ty(a), ParameterKind::Ty(b)) => Zip::zip_with(zipper, a, b),
            (ParameterKind::Lifetime(a), ParameterKind::Lifetime(b)) => Zip::zip_with(zipper, a, b),
            (ParameterKind::Ty(_), _) | (ParameterKind::Lifetime(_), _) => {
                panic!("zipping things of mixed kind")
            }
        }
    }
}

impl<TF: TypeFamily> Zip<TF> for Parameter<TF> {
    fn zip_with<Z: Zipper<TF>>(zipper: &mut Z, a: &Self, b: &Self) -> Fallible<()> {
        Zip::zip_with(zipper, &a.0, &b.0)
    }
}
