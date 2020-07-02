use crate::RustIrDatabase;
use chalk_ir::interner::Interner;
use chalk_ir::*;

pub trait IsCoinductive<I: Interner> {
    /// A goal G has coinductive semantics if proving G is allowed to
    /// assume G is true (very roughly speaking). In the case of
    /// chalk-ir, this is true for goals of the form `T: AutoTrait`,
    /// or if it is of the form `WellFormed(T: Trait)` where `Trait`
    /// is any trait. The latter is needed for dealing with WF
    /// requirements and cyclic traits, which generates cycles in the
    /// proof tree which must not be rejected but instead must be
    /// treated as a success.
    fn is_coinductive(&self, db: &dyn RustIrDatabase<I>) -> bool;
}

impl<I: Interner> IsCoinductive<I> for Goal<I> {
    fn is_coinductive(&self, db: &dyn RustIrDatabase<I>) -> bool {
        let interner = db.interner();
        match self.data(interner) {
            GoalData::DomainGoal(DomainGoal::Holds(wca)) => match wca {
                WhereClause::Implemented(tr) => {
                    db.trait_datum(tr.trait_id).is_auto_trait()
                        || db.trait_datum(tr.trait_id).is_coinductive_trait()
                }
                WhereClause::AliasEq(..) => false,
                WhereClause::LifetimeOutlives(..) => false,
                WhereClause::TypeOutlives(..) => false,
            },
            GoalData::DomainGoal(DomainGoal::WellFormed(WellFormed::Trait(..))) => true,
            GoalData::Quantified(QuantifierKind::ForAll, goal) => {
                goal.skip_binders().is_coinductive(db)
            }
            _ => false,
        }
    }
}

impl<I: Interner> IsCoinductive<I> for UCanonical<InEnvironment<Goal<I>>> {
    fn is_coinductive(&self, db: &dyn RustIrDatabase<I>) -> bool {
        self.canonical.value.goal.is_coinductive(db)
    }
}
