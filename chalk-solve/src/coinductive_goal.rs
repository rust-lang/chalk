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
    /// A "coinductive" goal `G` is a goal where `G :- G` should be considered
    /// true. When we are doing trait solving, if we encounter a cycle of goals
    /// where solving `G1` requires `G2..Gn` and solving `Gn` requires `G1`,
    /// then our behavior depends on whether each goal `Gi` in that cycle is
    /// coinductive.
    ///
    /// If all the goals are coinductive, then `G1` is considered provable,
    /// presuming that all the other subgoals for `G2..Gn` within can be fully
    /// proven.
    ///
    /// If any goal `Gi` in the cycle is inductive, however, then the cycle is
    /// considered unprovable.
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
            GoalData::DomainGoal(_) => false,

            // Goals like `forall<..> { G }` or `... => G` we will consider to
            // be coinductive if G is coinductive, although in practice I think
            // it would be ok to simply consider them coinductive all the time.
            GoalData::Quantified(_, goal) => goal.skip_binders().is_coinductive(db),
            GoalData::Implies(_, goal) => goal.is_coinductive(db),

            // The "All(...)" quantifier is considered coinductive. This could
            // be somewhat surprising as you might have `All(Gc, Gi)` where `Gc`
            // is coinductive and `Gi` is inductive. This however is really no
            // different than defining a fresh coinductive goal like
            //
            // ```notrust
            // Gx :- Gc, Gi
            // ```
            //
            // and requiring `Gx` in place of `All(Gc, Gi)`, and that would be
            // perfectly reasonable.
            GoalData::All(..) => true,

            // For simplicity, just assume these remaining types of goals must
            // be inductive, since there is no pressing reason to consider them
            // coinductive.
            GoalData::Not(_) => false,
            GoalData::EqGoal(_) => false,
            GoalData::CannotProve => false,
        }
    }
}

impl<I: Interner> IsCoinductive<I> for UCanonical<InEnvironment<Goal<I>>> {
    fn is_coinductive(&self, db: &dyn RustIrDatabase<I>) -> bool {
        self.canonical.value.goal.is_coinductive(db)
    }
}
