use crate::cast::{Cast, CastTo};
use crate::RustIrDatabase;
use chalk_ir::fold::Fold;
use chalk_ir::interner::{HasInterner, Interner};
use chalk_ir::*;
use chalk_rust_ir::*;
use std::marker::PhantomData;

/// The "clause builder" is a useful tool for building up sets of
/// program clauses. It takes ownership of the output vector while it
/// lasts, and offers methods like `push_clause` and so forth to
/// append to it.
pub struct ClauseBuilder<'me, I: Interner> {
    pub db: &'me dyn RustIrDatabase<I>,
    clauses: &'me mut Vec<ProgramClause<I>>,
    binders: Vec<ParameterKind<()>>,
    parameters: Vec<Parameter<I>>,
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

    /// Pushes a clause `forall<..> { consequence :- conditions }`
    /// into the set of program clauses, meaning that `consequence`
    /// can be proven if `conditions` are all true.  The `forall<..>`
    /// binders will be whichever binders have been pushed (see `push_binders`).
    pub fn push_clause(
        &mut self,
        consequence: impl CastTo<DomainGoal<I>>,
        conditions: impl IntoIterator<Item = impl CastTo<Goal<I>>>,
    ) {
        let clause = ProgramClauseImplication {
            consequence: consequence.cast(self.db.interner()),
            conditions: Goals::from(self.db.interner(), conditions),
        };

        if self.binders.len() == 0 {
            self.clauses.push(ProgramClause::Implies(clause));
        } else {
            self.clauses.push(ProgramClause::ForAll(Binders {
                binders: self.binders.clone(),
                value: clause,
            }));
        }

        debug!("pushed clause {:?}", self.clauses.last());
    }

    /// Accesses the placeholders for the current list of parameters in scope.
    pub fn placeholders_in_scope(&self) -> &[Parameter<I>] {
        &self.parameters
    }

    /// Accesses the placeholders for the current list of parameters in scope,
    /// in the form of a `Substitution`.
    pub fn substitution_in_scope(&self) -> Substitution<I> {
        Substitution::from(
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
    pub fn push_binders<V>(&mut self, binders: &Binders<V>, op: impl FnOnce(&mut Self, V::Result))
    where
        V: Fold<I> + HasInterner<Interner = I>,
    {
        let old_len = self.binders.len();
        self.binders.extend(binders.binders.clone());
        let params: Vec<_> = binders
            .binders
            .iter()
            .zip(old_len..)
            .map(|p| p.to_parameter(self.interner()))
            .collect();
        self.parameters.extend(params);

        let value = binders.substitute(self.interner(), &self.parameters[old_len..]);
        op(self, value);

        self.binders.truncate(old_len);
        self.parameters.truncate(old_len);
    }

    /// Push a single binder, for a type, at the end of the binder
    /// list.  The indices of previously bound variables are
    /// unaffected and hence the context remains usable. Invokes `op`,
    /// passing a type representing this new type variable in as an
    /// argument.
    #[allow(dead_code)]
    pub fn push_bound_ty(&mut self, op: impl FnOnce(&mut Self, Ty<I>)) {
        let binders = Binders {
            binders: vec![ParameterKind::Ty(())],
            value: PhantomData::<I>,
        };
        self.push_binders(&binders, |this, PhantomData| {
            let ty = this
                .placeholders_in_scope()
                .last()
                .unwrap()
                .assert_ty_ref()
                .clone();
            op(this, ty)
        });
    }

    pub fn interner(&self) -> &'me I {
        self.db.interner()
    }
}
