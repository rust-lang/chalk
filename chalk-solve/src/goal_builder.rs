#![allow(dead_code)]

use crate::RustIrDatabase;
use cast::CastTo;
use chalk_ir::cast::Cast;
use chalk_ir::cast::Caster;
use chalk_ir::*;
use chalk_rust_ir::ToParameter;
use fold::shift::Shift;
use fold::Fold;
use interner::{HasInterner, Interner};

pub(crate) struct GoalBuilder<'i, I: Interner> {
    db: &'i dyn RustIrDatabase<I>,
}

impl<'i, I: Interner> GoalBuilder<'i, I> {
    pub(crate) fn new(db: &'i dyn RustIrDatabase<I>) -> Self {
        GoalBuilder { db }
    }

    /// Returns the database within the goal builder.
    pub(crate) fn db(&self) -> &'i dyn RustIrDatabase<I> {
        self.db
    }

    /// Returns the interner within the goal builder.
    pub(crate) fn interner(&self) -> &'i I {
        self.db.interner()
    }

    /// Creates a goal that ensures all of the goals from the `goals`
    /// iterator are met (e.g., `goals[0] && ... && goals[N]`).
    pub(crate) fn all<GS, G>(&mut self, goals: GS) -> Goal<I>
    where
        GS: IntoIterator<Item = G>,
        G: CastTo<Goal<I>>,
    {
        Goal::all(self.interner(), goals.into_iter().casted(self.interner()))
    }

    /// Creates a goal `clauses => goal`. The clauses are given as an iterator
    /// and the goal is returned via the contained closure.
    pub(crate) fn implies<CS, C, G>(
        &mut self,
        clauses: CS,
        goal: impl FnOnce(&mut Self) -> G,
    ) -> Goal<I>
    where
        CS: IntoIterator<Item = C>,
        C: CastTo<ProgramClause<I>>,
        G: CastTo<Goal<I>>,
    {
        GoalData::Implies(
            clauses.into_iter().casted(self.interner()).collect(),
            goal(self).cast(self.interner()),
        )
        .intern(self.interner())
    }

    /// Given a bound value `binders` like `<P0..Pn> V`,
    /// creates a goal `forall<Q0..Qn> { G }` where
    /// the goal `G` is created by invoking a helper
    /// function `body`.
    ///
    /// # Parameters to `body`
    ///
    /// `body` will be invoked with:
    ///
    /// * the goal builder `self`
    /// * the substitution `Q0..Qn`
    /// * the bound value `[P0..Pn => Q0..Qn] V` instantiated
    ///   with the substitution
    /// * the value `passthru`, appropriately shifted so that
    ///   any debruijn indices within account for the new binder
    ///
    /// # Why is `body` a function and not a closure?
    ///
    /// This is to ensure that `body` doesn't accidentally reference
    /// values from the environment whose debruijn indices do not
    /// account for the new binder being created.
    pub(crate) fn forall<G, B, P>(
        &mut self,
        binders: &Binders<B>,
        passthru: P,
        body: fn(&mut Self, Substitution<I>, &B, P::Result) -> G,
    ) -> Goal<I>
    where
        B: Fold<I> + HasInterner<Interner = I>,
        P: Fold<I>,
        B::Result: std::fmt::Debug,
        G: CastTo<Goal<I>>,
    {
        self.quantified(QuantifierKind::ForAll, binders, passthru, body)
    }

    /// Like [`GoalBuilder::forall`], but for a `exists<Q0..Qn> { G }` goal.
    pub(crate) fn exists<G, B, P>(
        &mut self,
        binders: &Binders<B>,
        passthru: P,
        body: fn(&mut Self, Substitution<I>, &B, P::Result) -> G,
    ) -> Goal<I>
    where
        B: Fold<I> + HasInterner<Interner = I>,
        P: Fold<I>,
        B::Result: std::fmt::Debug,
        G: CastTo<Goal<I>>,
    {
        self.quantified(QuantifierKind::Exists, binders, passthru, body)
    }

    /// A combined helper functon for the various methods
    /// to create `forall` and `exists` goals. See:
    ///
    /// * [`GoalBuilder::forall`]
    /// * [`GoalBuilder::partially_forall`]
    ///
    /// for details.
    pub(crate) fn quantified<G, B, P>(
        &mut self,
        quantifier_kind: QuantifierKind,
        binders: &Binders<B>,
        passthru: P,
        body: fn(&mut Self, Substitution<I>, &B, P::Result) -> G,
    ) -> Goal<I>
    where
        B: Fold<I> + HasInterner<Interner = I>,
        P: Fold<I>,
        B::Result: std::fmt::Debug,
        G: CastTo<Goal<I>>,
    {
        let interner = self.interner();
        let bound_goal = binders.map_ref(|bound_value| {
            let substitution: Substitution<I> = Substitution::from(
                interner,
                binders
                    .binders
                    .iter()
                    .zip(0..)
                    .map(|p| p.to_parameter(interner)),
            );
            let passthru_shifted = passthru.shifted_in(self.interner());
            let result = body(self, substitution, bound_value, passthru_shifted);
            result.cast(self.interner())
        });
        GoalData::Quantified(quantifier_kind, bound_goal).intern(self.interner())
    }
}
