use crate::RustIrDatabase;
use chalk_ir::*;

pub trait IsCoinductive {
    /// A goal G has coinductive semantics if proving G is allowed to
    /// assume G is true (very roughly speaking). In the case of
    /// chalk-ir, this is true for goals of the form `T: AutoTrait`,
    /// or if it is of the form `WellFormed(T: Trait)` where `Trait`
    /// is any trait. The latter is needed for dealing with WF
    /// requirements and cyclic traits, which generates cycles in the
    /// proof tree which must not be rejected but instead must be
    /// treated as a success.
    fn is_coinductive(&self, db: &dyn RustIrDatabase) -> bool;
}

impl IsCoinductive for Goal {
    fn is_coinductive(&self, db: &dyn RustIrDatabase) -> bool {
        match self {
            Goal::Leaf(LeafGoal::DomainGoal(DomainGoal::Holds(wca))) => match wca {
                WhereClause::Implemented(tr) => {
                    db.trait_datum(tr.trait_id).binders.value.flags.auto
                }
                WhereClause::ProjectionEq(..) => false,
            },
            Goal::Leaf(LeafGoal::DomainGoal(DomainGoal::WellFormed(WellFormed::Trait(..)))) => true,
            Goal::Quantified(QuantifierKind::ForAll, goal) => goal.value.is_coinductive(db),
            _ => false,
        }
    }
}

impl IsCoinductive for UCanonical<InEnvironment<Goal>> {
    fn is_coinductive(&self, db: &dyn RustIrDatabase) -> bool {
        self.canonical.value.goal.is_coinductive(db)
    }
}
