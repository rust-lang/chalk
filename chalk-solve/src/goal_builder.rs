use crate::RustIrDatabase;
use cast::CastTo;
use chalk_ir::cast::Cast;
use chalk_ir::cast::Caster;
use chalk_ir::*;
use fold::shift::Shift;
use fold::TypeFoldable;
use interner::{HasInterner, Interner};

pub struct GoalBuilder<'i, I: Interner> {
    db: &'i dyn RustIrDatabase<I>,
}

impl<'i, I: Interner> GoalBuilder<'i, I> {
    pub fn new(db: &'i dyn RustIrDatabase<I>) -> Self {
        GoalBuilder { db }
    }

    /// Returns the database within the goal builder.
    pub fn db(&self) -> &'i dyn RustIrDatabase<I> {
        self.db
    }

    /// Returns the interner within the goal builder.
    pub fn interner(&self) -> I {
        self.db.interner()
    }

    /// Creates a goal that ensures all of the goals from the `goals`
    /// iterator are met (e.g., `goals[0] && ... && goals[N]`).
    pub fn all<GS, G>(&mut self, goals: GS) -> Goal<I>
    where
        GS: IntoIterator<Item = G>,
        G: CastTo<Goal<I>>,
    {
        Goal::all(self.interner(), goals.into_iter().casted(self.interner()))
    }

    /// Creates a goal `clauses => goal`. The clauses are given as an iterator
    /// and the goal is returned via the contained closure.
    pub fn implies<CS, C, G>(&mut self, clauses: CS, goal: impl FnOnce(&mut Self) -> G) -> Goal<I>
    where
        CS: IntoIterator<Item = C>,
        C: CastTo<ProgramClause<I>>,
        G: CastTo<Goal<I>>,
    {
        GoalData::Implies(
            ProgramClauses::from_iter(self.interner(), clauses),
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
    pub fn forall<G, B, P>(
        &mut self,
        binders: &Binders<B>,
        passthru: P,
        body: fn(&mut Self, Substitution<I>, &B, P) -> G,
    ) -> Goal<I>
    where
        B: HasInterner<Interner = I>,
        P: TypeFoldable<I>,
        G: CastTo<Goal<I>>,
    {
        self.quantified(QuantifierKind::ForAll, binders, passthru, body)
    }

    /// Like [`GoalBuilder::forall`], but for a `exists<Q0..Qn> { G }` goal.
    pub fn exists<G, B, P>(
        &mut self,
        binders: &Binders<B>,
        passthru: P,
        body: fn(&mut Self, Substitution<I>, &B, P) -> G,
    ) -> Goal<I>
    where
        B: HasInterner<Interner = I>,
        P: TypeFoldable<I>,
        G: CastTo<Goal<I>>,
    {
        self.quantified(QuantifierKind::Exists, binders, passthru, body)
    }

    /// A combined helper functon for the various methods
    /// to create `forall` and `exists` goals. See:
    ///
    /// * [`GoalBuilder::forall`]
    /// * [`GoalBuilder::exists`]
    ///
    /// for details.
    fn quantified<G, B, P>(
        &mut self,
        quantifier_kind: QuantifierKind,
        binders: &Binders<B>,
        passthru: P,
        body: fn(&mut Self, Substitution<I>, &B, P) -> G,
    ) -> Goal<I>
    where
        B: HasInterner<Interner = I>,
        P: TypeFoldable<I>,
        G: CastTo<Goal<I>>,
    {
        let interner = self.interner();

        // Make an identity mapping `[0 => ^0.0, 1 => ^0.1, ..]`
        // and so forth. This substitution is mapping from the `<P0..Pn>` variables
        // in `binders` to the corresponding `P0..Pn` variables we're about to
        // introduce in the form of a `forall<P0..Pn>` goal. Of course, it's
        // actually an identity mapping, since this `forall` will be the innermost
        // debruijn binder and so forth, so there's no actual reason to
        // *do* the substitution, since it would effectively just be a clone.
        let substitution = Substitution::from_iter(
            interner,
            binders
                .binders
                .iter(interner)
                .enumerate()
                .map(|p| p.to_generic_arg(interner)),
        );

        // Shift passthru into one level of binder, to account for the `forall<P0..Pn>`
        // we are about to introduce.
        let passthru_shifted = passthru.shifted_in(self.interner());

        // Invoke `body` function, which returns a goal, and wrap that goal in the binders
        // from `binders`, and finally a `forall` or `exists` goal.
        let bound_goal = binders.map_ref(|bound_value| {
            body(self, substitution, bound_value, passthru_shifted).cast(interner)
        });
        GoalData::Quantified(quantifier_kind, bound_goal).intern(interner)
    }
}
