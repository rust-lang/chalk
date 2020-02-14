use crate::interner::HasInterner;
use crate::zip::{Zip, Zipper};
use crate::*;

/// A fast check to see whether two things could ever possibly match.
pub trait CouldMatch<T: ?Sized> {
    fn could_match(&self, other: &T) -> bool;
}

impl<T, I> CouldMatch<T> for T
where
    T: Zip<I> + ?Sized + HasInterner<Interner = I>,
    I: Interner,
{
    fn could_match(&self, other: &T) -> bool {
        return Zip::zip_with(&mut MatchZipper, self, other).is_ok();

        struct MatchZipper;

        impl<I: Interner> Zipper<I> for MatchZipper {
            fn zip_tys(&mut self, a: &Ty<I>, b: &Ty<I>) -> Fallible<()> {
                let could_match = match (a.data(), b.data()) {
                    (&TyData::Apply(ref a), &TyData::Apply(ref b)) => {
                        let names_could_match = a.name == b.name;

                        names_could_match
                            && a.substitution
                                .iter()
                                .zip(&b.substitution)
                                .all(|(p_a, p_b)| p_a.could_match(&p_b))
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
        }
    }
}

impl<I: Interner> CouldMatch<DomainGoal<I>> for ProgramClause<I> {
    fn could_match(&self, other: &DomainGoal<I>) -> bool {
        match self {
            ProgramClause::Implies(implication) => implication.consequence.could_match(other),

            ProgramClause::ForAll(clause) => clause.value.consequence.could_match(other),
        }
    }
}
