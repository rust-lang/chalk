use crate::interner::HasInterner;
use crate::zip::{Zip, Zipper};
use crate::*;

/// A fast check to see whether two things could ever possibly match.
pub trait CouldMatch<T: ?Sized + HasInterner> {
    fn could_match(&self, interner: &T::Interner, other: &T) -> bool;
}

#[allow(unreachable_code, unused_variables)]
impl<T, I> CouldMatch<T> for T
where
    T: Zip<I> + ?Sized + HasInterner<Interner = I>,
    I: Interner,
{
    fn could_match(&self, interner: &I, other: &T) -> bool {
        return Zip::zip_with(&mut MatchZipper { interner }, self, other).is_ok();

        struct MatchZipper<'i, I> {
            interner: &'i I,
        };

        impl<'i, I: Interner> Zipper<'i, I> for MatchZipper<'i, I> {
            fn zip_tys(&mut self, a: &Ty<I>, b: &Ty<I>) -> Fallible<()> {
                let could_match = match (a.data(self.interner), b.data(self.interner)) {
                    (&TyData::Apply(ref a), &TyData::Apply(ref b)) => {
                        let names_could_match = a.name == b.name;

                        names_could_match
                            && a.substitution
                                .iter()
                                .zip(&b.substitution)
                                .all(|(p_a, p_b)| p_a.could_match(self.interner, &p_b))
                    }

                    _ => true,
                };

                if could_match {
                    Ok(())
                } else {
                    Err(NoSolution)
                }
            }

            fn zip_lifetimes(&mut self, _: &Lifetime<I>, _: &Lifetime<I>) -> Fallible<()> {
                Ok(())
            }

            fn zip_binders<T>(&mut self, a: &Binders<T>, b: &Binders<T>) -> Fallible<()>
            where
                T: Zip<I>,
            {
                Zip::zip_with(self, &a.value, &b.value)
            }

            fn interner(&self) -> &'i I {
                self.interner
            }
        }
    }
}

impl<I: Interner> CouldMatch<DomainGoal<I>> for ProgramClause<I> {
    fn could_match(&self, interner: &I, other: &DomainGoal<I>) -> bool {
        match self {
            ProgramClause::Implies(implication) => {
                implication.consequence.could_match(interner, other)
            }

            ProgramClause::ForAll(clause) => clause.value.consequence.could_match(interner, other),
        }
    }
}
