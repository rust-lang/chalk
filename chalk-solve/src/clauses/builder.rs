use std::marker::PhantomData;

use crate::cast::{Cast, CastTo};
use crate::RustIrDatabase;
use chalk_ir::fold::{Shift, TypeFoldable};
use chalk_ir::interner::{HasInterner, Interner};
use chalk_ir::*;
use tracing::{debug, instrument};

/// The "clause builder" is a useful tool for building up sets of
/// program clauses. It takes ownership of the output vector while it
/// lasts, and offers methods like `push_clause` and so forth to
/// append to it.
pub struct ClauseBuilder<'me, I: Interner> {
    pub db: &'me dyn RustIrDatabase<I>,
    clauses: &'me mut Vec<ProgramClause<I>>,
    binders: Vec<VariableKind<I>>,
    parameters: Vec<GenericArg<I>>,
}

impl<'me, I: Interner> ClauseBuilder<'me, I> {
    pub fn new(db: &'me dyn RustIrDatabase<I>, clauses: &'me mut Vec<ProgramClause<I>>) -> Self {
        Self {
            db,
            clauses,
            binders: vec![],
            parameters: vec![],
        }
    }

    /// Pushes a "fact" `forall<..> { consequence }` into the set of
    /// program clauses, meaning something that we can assume to be
    /// true unconditionally. The `forall<..>` binders will be
    /// whichever binders have been pushed (see `push_binders`).
    pub fn push_fact(&mut self, consequence: impl CastTo<DomainGoal<I>>) {
        self.push_clause(consequence, None::<Goal<_>>);
    }

    /// Pushes a "fact" `forall<..> { consequence }` into the set of
    /// program clauses, meaning something that we can assume to be
    /// true unconditionally. The `forall<..>` binders will be
    /// whichever binders have been pushed (see `push_binders`).
    pub fn push_fact_with_priority(
        &mut self,
        consequence: impl CastTo<DomainGoal<I>>,
        constraints: impl IntoIterator<Item = InEnvironment<Constraint<I>>>,
        priority: ClausePriority,
    ) {
        self.push_clause_with_priority(consequence, None::<Goal<_>>, constraints, priority);
    }

    /// Pushes a clause `forall<..> { consequence :- conditions }`
    /// into the set of program clauses, meaning that `consequence`
    /// can be proven if `conditions` are all true.  The `forall<..>`
    /// binders will be whichever binders have been pushed (see `push_binders`).
    pub fn push_clause(
        &mut self,
        consequence: impl CastTo<DomainGoal<I>>,
        conditions: impl IntoIterator<Item = impl CastTo<Goal<I>>>,
    ) {
        self.push_clause_with_priority(consequence, conditions, None, ClausePriority::High)
    }

    pub fn push_fact_with_constraints(
        &mut self,
        consequence: impl CastTo<DomainGoal<I>>,
        constraints: impl IntoIterator<Item = InEnvironment<Constraint<I>>>,
    ) {
        self.push_fact_with_priority(consequence, constraints, ClausePriority::High)
    }

    /// Pushes a clause `forall<..> { consequence :- conditions ; constraints }`
    /// into the set of program clauses, meaning that `consequence`
    /// can be proven if `conditions` are all true and `constraints`
    /// are proven to hold.  The `forall<..>` binders will be whichever binders
    /// have been pushed (see `push_binders`).
    pub fn push_clause_with_priority(
        &mut self,
        consequence: impl CastTo<DomainGoal<I>>,
        conditions: impl IntoIterator<Item = impl CastTo<Goal<I>>>,
        constraints: impl IntoIterator<Item = InEnvironment<Constraint<I>>>,
        priority: ClausePriority,
    ) {
        let interner = self.db.interner();
        let clause = ProgramClauseImplication {
            consequence: consequence.cast(interner),
            conditions: Goals::from_iter(interner, conditions),
            constraints: Constraints::from_iter(interner, constraints),
            priority,
        };

        let clause = if self.binders.is_empty() {
            // Compensate for the added empty binder
            clause.shifted_in(interner)
        } else {
            clause
        };

        self.clauses.push(
            ProgramClauseData(Binders::new(
                VariableKinds::from_iter(interner, self.binders.clone()),
                clause,
            ))
            .intern(interner),
        );

        debug!("pushed clause {:?}", self.clauses.last());
    }

    /// Accesses the placeholders for the current list of parameters in scope.
    pub fn placeholders_in_scope(&self) -> &[GenericArg<I>] {
        &self.parameters
    }

    /// Accesses the placeholders for the current list of parameters in scope,
    /// in the form of a `Substitution`.
    pub fn substitution_in_scope(&self) -> Substitution<I> {
        Substitution::from_iter(
            self.db.interner(),
            self.placeholders_in_scope().iter().cloned(),
        )
    }

    /// Executes `op` with the `binders` in-scope; `op` is invoked
    /// with the bound value `v` as a parameter. After `op` finishes,
    /// the binders are popped from scope.
    ///
    /// The new binders are always pushed onto the end of the internal
    /// list of binders; this means that any extant values where were
    /// created referencing the *old* list of binders are still valid.
    #[instrument(level = "debug", skip(self, op))]
    pub fn push_binders<R, V>(
        &mut self,
        binders: Binders<V>,
        op: impl FnOnce(&mut Self, V) -> R,
    ) -> R
    where
        V: TypeFoldable<I> + HasInterner<Interner = I>,
        V: std::fmt::Debug,
    {
        let old_len = self.binders.len();
        let interner = self.interner();
        self.binders.extend(binders.binders.iter(interner).cloned());
        self.parameters.extend(
            binders
                .binders
                .iter(interner)
                .zip(old_len..)
                .map(|(pk, i)| (i, pk).to_generic_arg(interner)),
        );
        let value = binders.substitute(self.interner(), &self.parameters[old_len..]);
        debug!(?value);
        let res = op(self, value);

        self.binders.truncate(old_len);
        self.parameters.truncate(old_len);
        res
    }

    /// Push a single binder, for a type, at the end of the binder
    /// list.  The indices of previously bound variables are
    /// unaffected and hence the context remains usable. Invokes `op`,
    /// passing a type representing this new type variable in as an
    /// argument.
    pub fn push_bound_ty(&mut self, op: impl FnOnce(&mut Self, Ty<I>)) {
        let interner = self.interner();
        let binders = Binders::new(
            VariableKinds::from1(interner, VariableKind::Ty(TyVariableKind::General)),
            PhantomData::<I>,
        );
        self.push_binders(binders, |this, PhantomData| {
            let ty = this
                .placeholders_in_scope()
                .last()
                .unwrap()
                .assert_ty_ref(interner)
                .clone();
            op(this, ty)
        });
    }

    /// Push a single binder, for a lifetime, at the end of the binder
    /// list.  The indices of previously bound variables are
    /// unaffected and hence the context remains usable. Invokes `op`,
    /// passing a lifetime representing this new lifetime variable in as an
    /// argument.
    pub fn push_bound_lifetime(&mut self, op: impl FnOnce(&mut Self, Lifetime<I>)) {
        let interner = self.interner();
        let binders = Binders::new(
            VariableKinds::from1(interner, VariableKind::Lifetime),
            PhantomData::<I>,
        );
        self.push_binders(binders, |this, PhantomData| {
            let lifetime = this
                .placeholders_in_scope()
                .last()
                .unwrap()
                .assert_lifetime_ref(interner)
                .clone();
            op(this, lifetime)
        });
    }

    pub fn interner(&self) -> I {
        self.db.interner()
    }
}
