use fallible::*;
use ir::*;
use zip::{Zip, Zipper};

/// A fast check to see whether two things could ever possibly match.
pub trait CouldMatch<T> {
    fn could_match(&self, other: &T) -> bool;
}

impl<T: Zip> CouldMatch<T> for T {
    fn could_match(&self, other: &T) -> bool {
        return Zip::zip_with(&mut MatchZipper, self, other).is_ok();

        struct MatchZipper;

        impl Zipper for MatchZipper {
            fn zip_tys(&mut self, a: &Ty, b: &Ty) -> Fallible<()> {
                let could_match = match (a, b) {
                    (&Ty::Apply(ref a), &Ty::Apply(ref b)) => {
                        let names_could_match = match (a.name, b.name) {
                            (TypeName::ItemId(item_a), TypeName::ItemId(item_b)) =>
                                item_a == item_b,
                            _ => true
                        };

                        names_could_match &&
                            a.parameters.iter()
                                        .zip(&b.parameters)
                                        .all(|(p_a, p_b)| p_a.could_match(p_b))
                    }

                    _ => true,
                };

                if could_match { Ok(()) } else { Err(NoSolution) }
            }

            fn zip_lifetimes(&mut self, _: &Lifetime, _: &Lifetime) -> Fallible<()> {
                Ok(())
            }

            fn zip_binders<T>(&mut self, a: &Binders<T>, b: &Binders<T>) -> Fallible<()>
                where T: Zip
            {
                Zip::zip_with(self, &a.value, &b.value)
            }
        }
    }
}

impl CouldMatch<DomainGoal> for ProgramClause {
    fn could_match(&self, other: &DomainGoal) -> bool {
        self.implication.value.consequence.could_match(other)
    }
}

