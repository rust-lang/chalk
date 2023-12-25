//! Fast matching check for zippable values.

use crate::interner::HasInterner;
use crate::zip::{Zip, Zipper};
use crate::*;

/// A fast check to see whether two things could ever possibly match.
pub trait CouldMatch<T: ?Sized + HasInterner> {
    /// Checks whether `self` and `other` could possibly match.
    fn could_match(
        &self,
        interner: T::Interner,
        db: &dyn UnificationDatabase<T::Interner>,
        other: &T,
    ) -> bool;
}

#[allow(unreachable_code, unused_variables)]
impl<T, I> CouldMatch<T> for T
where
    T: Zip<I> + ?Sized + HasInterner<Interner = I>,
    I: Interner,
{
    fn could_match(&self, interner: I, db: &dyn UnificationDatabase<I>, other: &T) -> bool {
        return Zip::zip_with(
            &mut MatchZipper { interner, db },
            Variance::Invariant,
            self,
            other,
        )
        .is_ok();

        struct MatchZipper<'i, I> {
            interner: I,
            db: &'i dyn UnificationDatabase<I>,
        }

        impl<'i, I: Interner> Zipper<I> for MatchZipper<'i, I> {
            fn zip_tys(&mut self, variance: Variance, a: &Ty<I>, b: &Ty<I>) -> Fallible<()> {
                let interner = self.interner;
                let matches = |a: &Substitution<I>, b: &Substitution<I>| {
                    a.iter(interner)
                        .zip(b.iter(interner))
                        .all(|(p_a, p_b)| p_a.could_match(interner, self.db, p_b))
                };
                let could_match = match (a.kind(interner), b.kind(interner)) {
                    (TyKind::Adt(id_a, substitution_a), TyKind::Adt(id_b, substitution_b)) => {
                        id_a == id_b
                            && self
                                .zip_substs(
                                    variance,
                                    Some(self.unification_database().adt_variance(*id_a)),
                                    substitution_a.as_slice(interner),
                                    substitution_b.as_slice(interner),
                                )
                                .is_ok()
                    }
                    (
                        TyKind::AssociatedType(assoc_ty_a, substitution_a),
                        TyKind::AssociatedType(assoc_ty_b, substitution_b),
                    ) => assoc_ty_a == assoc_ty_b && matches(substitution_a, substitution_b),
                    (TyKind::Scalar(scalar_a), TyKind::Scalar(scalar_b)) => scalar_a == scalar_b,
                    (TyKind::Str, TyKind::Str) => true,
                    (
                        TyKind::Tuple(arity_a, substitution_a),
                        TyKind::Tuple(arity_b, substitution_b),
                    ) => arity_a == arity_b && matches(substitution_a, substitution_b),
                    (
                        TyKind::OpaqueType(opaque_ty_a, substitution_a),
                        TyKind::OpaqueType(opaque_ty_b, substitution_b),
                    ) => opaque_ty_a == opaque_ty_b && matches(substitution_a, substitution_b),
                    (TyKind::Slice(ty_a), TyKind::Slice(ty_b)) => {
                        ty_a.could_match(interner, self.db, ty_b)
                    }
                    (
                        TyKind::FnDef(fn_def_a, substitution_a),
                        TyKind::FnDef(fn_def_b, substitution_b),
                    ) => {
                        fn_def_a == fn_def_b
                            && self
                                .zip_substs(
                                    variance,
                                    Some(self.unification_database().fn_def_variance(*fn_def_a)),
                                    substitution_a.as_slice(interner),
                                    substitution_b.as_slice(interner),
                                )
                                .is_ok()
                    }
                    (
                        TyKind::Ref(mutability_a, lifetime_a, ty_a),
                        TyKind::Ref(mutability_b, lifetime_b, ty_b),
                    ) => {
                        mutability_a == mutability_b
                            && lifetime_a.could_match(interner, self.db, lifetime_b)
                            && ty_a.could_match(interner, self.db, ty_b)
                    }
                    (TyKind::Raw(mutability_a, ty_a), TyKind::Raw(mutability_b, ty_b)) => {
                        mutability_a == mutability_b && ty_a.could_match(interner, self.db, ty_b)
                    }
                    (TyKind::Never, TyKind::Never) => true,
                    (TyKind::Array(ty_a, const_a), TyKind::Array(ty_b, const_b)) => {
                        ty_a.could_match(interner, self.db, ty_b)
                            && const_a.could_match(interner, self.db, const_b)
                    }
                    (
                        TyKind::Closure(id_a, substitution_a),
                        TyKind::Closure(id_b, substitution_b),
                    ) => id_a == id_b && matches(substitution_a, substitution_b),
                    (
                        TyKind::Coroutine(coroutine_a, substitution_a),
                        TyKind::Coroutine(coroutine_b, substitution_b),
                    ) => {
                        coroutine_a == coroutine_b
                            && self
                                .zip_substs(
                                    variance,
                                    None,
                                    substitution_a.as_slice(interner),
                                    substitution_b.as_slice(interner),
                                )
                                .is_ok()
                    }
                    (
                        TyKind::CoroutineWitness(coroutine_a, substitution_a),
                        TyKind::CoroutineWitness(coroutine_b, substitution_b),
                    ) => {
                        coroutine_a == coroutine_b
                            && self
                                .zip_substs(
                                    variance,
                                    None,
                                    substitution_a.as_slice(interner),
                                    substitution_b.as_slice(interner),
                                )
                                .is_ok()
                    }
                    (TyKind::Foreign(foreign_ty_a), TyKind::Foreign(foreign_ty_b)) => {
                        foreign_ty_a == foreign_ty_b
                    }
                    (TyKind::Error, TyKind::Error) => true,

                    _ => true,
                };

                if could_match {
                    Ok(())
                } else {
                    Err(NoSolution)
                }
            }

            fn zip_lifetimes(
                &mut self,
                variance: Variance,
                _: &Lifetime<I>,
                _: &Lifetime<I>,
            ) -> Fallible<()> {
                Ok(())
            }

            fn zip_consts(
                &mut self,
                variance: Variance,
                _: &Const<I>,
                _: &Const<I>,
            ) -> Fallible<()> {
                Ok(())
            }

            fn zip_binders<T>(
                &mut self,
                variance: Variance,
                a: &Binders<T>,
                b: &Binders<T>,
            ) -> Fallible<()>
            where
                T: HasInterner + Zip<I>,
            {
                Zip::zip_with(self, variance, &a.value, &b.value)
            }

            fn interner(&self) -> I {
                self.interner
            }

            fn unification_database(&self) -> &dyn UnificationDatabase<I> {
                self.db
            }
        }
    }
}

impl<I: Interner> CouldMatch<DomainGoal<I>> for ProgramClauseData<I> {
    fn could_match(
        &self,
        interner: I,
        db: &dyn UnificationDatabase<I>,
        other: &DomainGoal<I>,
    ) -> bool {
        self.0.value.consequence.could_match(interner, db, other)
    }
}

impl<I: Interner> CouldMatch<DomainGoal<I>> for ProgramClause<I> {
    fn could_match(
        &self,
        interner: I,
        db: &dyn UnificationDatabase<I>,
        other: &DomainGoal<I>,
    ) -> bool {
        self.data(interner).could_match(interner, db, other)
    }
}
