#![allow(dead_code)]

use crate::RustIrDatabase;
use cast::CastTo;
use chalk_ir::cast::Cast;
use chalk_ir::cast::Caster;
use chalk_ir::*;
use fold::shift::Shift;
use fold::Fold;
use interner::{HasInterner, Interner};

pub(crate) struct GoalBuilder<'i, I: Interner> {
    pub db: &'i dyn RustIrDatabase<I>,
}

impl<'i, I: Interner> GoalBuilder<'i, I> {
    pub(crate) fn new(db: &'i dyn RustIrDatabase<I>) -> Self {
        GoalBuilder { db }
    }

    pub(crate) fn interner(&self) -> &'i I {
        self.db.interner()
    }

    pub(crate) fn all<GS, G>(&mut self, goals: GS) -> Goal<I>
    where
        GS: IntoIterator<Item = G>,
        G: CastTo<Goal<I>>,
    {
        Goal::all(self.interner(), goals.into_iter().casted(self.interner()))
    }

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

    pub(crate) fn forall<G, B, P>(
        &mut self,
        binders: &Binders<B>,
        passthru: P,
        body: fn(&mut Self, &B, P::Result) -> G,
    ) -> Goal<I>
    where
        B: Fold<I> + HasInterner<Interner = I>,
        P: Fold<I>,
        B::Result: std::fmt::Debug,
        G: CastTo<Goal<I>>,
    {
        self.quantified(QuantifierKind::ForAll, binders, passthru, body)
    }

    pub(crate) fn exists<G, B, P>(
        &mut self,
        binders: &Binders<B>,
        passthru: P,
        body: fn(&mut Self, &B, P::Result) -> G,
    ) -> Goal<I>
    where
        B: Fold<I> + HasInterner<Interner = I>,
        P: Fold<I>,
        B::Result: std::fmt::Debug,
        G: CastTo<Goal<I>>,
    {
        self.quantified(QuantifierKind::Exists, binders, passthru, body)
    }

    pub(crate) fn quantified<G, B, P>(
        &mut self,
        quantifier_kind: QuantifierKind,
        binders: &Binders<B>,
        passthru: P,
        body: fn(&mut Self, &B, P::Result) -> G,
    ) -> Goal<I>
    where
        B: Fold<I> + HasInterner<Interner = I>,
        P: Fold<I>,
        B::Result: std::fmt::Debug,
        G: CastTo<Goal<I>>,
    {
        let bound_goal = binders.map_ref(|bound_value| {
            let passthru_shifted = passthru.shifted_in(self.interner());
            let result = body(self, bound_value, passthru_shifted);
            result.cast(self.interner())
        });
        GoalData::Quantified(quantifier_kind, bound_goal).intern(self.interner())
    }
}
