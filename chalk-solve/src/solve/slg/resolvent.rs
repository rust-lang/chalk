use crate::infer::InferenceTable;
use crate::solve::slg::{self, SlgContext, TruncatingInferenceTable};
use chalk_engine::fallible::Fallible;
use chalk_ir::family::ChalkIr;
use chalk_ir::fold::shift::Shift;
use chalk_ir::fold::Fold;
use chalk_ir::zip::{Zip, Zipper};
use chalk_ir::*;

use chalk_engine::context;
use chalk_engine::{ExClause, Literal, TimeStamp};

///////////////////////////////////////////////////////////////////////////
// SLG RESOLVENTS
//
// The "SLG Resolvent" is used to combine a *goal* G with some
// clause or answer *C*.  It unifies the goal's selected literal
// with the clause and then inserts the clause's conditions into
// the goal's list of things to prove, basically. Although this is
// one operation in EWFS, we have specialized variants for merging
// a program clause and an answer (though they share some code in
// common).
//
// Terminology note: The NFTD and RR papers use the term
// "resolvent" to mean both the factor and the resolvent, but EWFS
// distinguishes the two. We follow EWFS here since -- in the code
// -- we tend to know whether there are delayed literals or not,
// and hence to know which code path we actually want.
//
// From EWFS:
//
// Let G be an X-clause A :- D | L1,...Ln, where N > 0, and Li be selected atom.
//
// Let C be an X-clause with no delayed literals. Let
//
//     C' = A' :- L'1...L'm
//
// be a variant of C such that G and C' have no variables in
// common.
//
// Let Li and A' be unified with MGU S.
//
// Then:
//
//     S(A :- D | L1...Li-1, L1'...L'm, Li+1...Ln)
//
// is the SLG resolvent of G with C.

impl context::ResolventOps<SlgContext> for TruncatingInferenceTable {
    /// Applies the SLG resolvent algorithm to incorporate a program
    /// clause into the main X-clause, producing a new X-clause that
    /// must be solved.
    ///
    /// # Parameters
    ///
    /// - `goal` is the goal G that we are trying to solve
    /// - `clause` is the program clause that may be useful to that end
    fn resolvent_clause(
        &mut self,
        environment: &Environment<ChalkIr>,
        goal: &DomainGoal<ChalkIr>,
        subst: &Substitution<ChalkIr>,
        clause: &ProgramClause<ChalkIr>,
    ) -> Fallible<Canonical<ExClause<SlgContext>>> {
        // Relating the above description to our situation:
        //
        // - `goal` G, except with binders for any existential variables.
        //   - Also, we always select the first literal in `ex_clause.literals`, so `i` is 0.
        // - `clause` is C, except with binders for any existential variables.

        debug_heading!(
            "resolvent_clause(\
             \n    goal={:?},\
             \n    clause={:?})",
            goal,
            clause,
        );

        let snapshot = self.infer.snapshot();

        // C' in the description above is `consequence :- conditions`.
        //
        // Note that G and C' have no variables in common.
        let ProgramClauseImplication {
            consequence,
            conditions,
        } = match clause {
            ProgramClause::Implies(implication) => implication.clone(),
            ProgramClause::ForAll(implication) => {
                self.infer.instantiate_binders_existentially(implication)
            }
        };
        debug!("consequence = {:?}", consequence);
        debug!("conditions = {:?}", conditions);

        // Unify the selected literal Li with C'.
        let unification_result = self.infer.unify(environment, goal, &consequence)?;

        // Final X-clause that we will return.
        let mut ex_clause = ExClause {
            subst: subst.clone(),
            delayed_literals: vec![],
            constraints: vec![],
            subgoals: vec![],
            current_time: TimeStamp::default(),
            floundered_subgoals: vec![],
        };

        // Add the subgoals/region-constraints that unification gave us.
        slg::into_ex_clause(unification_result, &mut ex_clause);

        // Add the `conditions` from the program clause into the result too.
        ex_clause
            .subgoals
            .extend(conditions.into_iter().map(|c| match c {
                Goal::Not(c) => Literal::Negative(InEnvironment::new(environment, *c)),
                c => Literal::Positive(InEnvironment::new(environment, c)),
            }));

        let canonical_ex_clause = self.infer.canonicalize(&ex_clause).quantified;

        self.infer.rollback_to(snapshot);

        Ok(canonical_ex_clause)
    }

    ///////////////////////////////////////////////////////////////////////////
    // apply_answer_subst
    //
    // Apply answer subst has the job of "plugging in" the answer to a
    // query into the pending ex-clause. To see how it works, it's worth stepping
    // up one level. Imagine that first we are trying to prove a goal A:
    //
    //     A :- T: Foo<Vec<?U>>, ?U: Bar
    //
    // this spawns a subgoal `T: Foo<Vec<?0>>`, and it's this subgoal that
    // has now produced an answer `?0 = u32`. When the goal A spawned the
    // subgoal, it will also have registered a `PendingExClause` with its
    // current state.  At the point where *this* method has been invoked,
    // that pending ex-clause has been instantiated with fresh variables and setup,
    // so we have four bits of incoming information:
    //
    // - `ex_clause`, which is the remaining stuff to prove for the goal A.
    //   Here, the inference variable `?U` has been instantiated with a fresh variable
    //   `?X`.
    //   - `A :- ?X: Bar`
    // - `selected_goal`, which is the thing we were trying to prove when we
    //   spawned the subgoal. It shares inference variables with `ex_clause`.
    //   - `T: Foo<Vec<?X>>`
    // - `answer_table_goal`, which is the subgoal in canonical form:
    //   - `for<type> T: Foo<Vec<?0>>`
    // - `canonical_answer_subst`, which is an answer to `answer_table_goal`.
    //   - `[?0 = u32]`
    //
    // In this case, this function will (a) unify `u32` and `?X` and then
    // (b) return `ex_clause` (extended possibly with new region constraints
    // and subgoals).
    //
    // One way to do this would be to (a) substitute
    // `canonical_answer_subst` into `answer_table_goal` (yielding `T:
    // Foo<Vec<u32>>`) and then (b) instantiate the result with fresh
    // variables (no effect in this instance) and then (c) unify that with
    // `selected_goal` (yielding, indirectly, that `?X = u32`). But that
    // is not what we do: it's inefficient, to start, but it also causes
    // problems because unification of projections can make new
    // sub-goals. That is, even if the answers don't involve any
    // projections, the table goals might, and this can create an infinite
    // loop (see also #74).
    //
    // What we do instead is to (a) instantiate the substitution, which
    // may have free variables in it (in this case, it would not, and the
    // instantiation would have no effect) and then (b) zip
    // `answer_table_goal` and `selected_goal` without having done any
    // substitution. After all, these ought to be basically the same,
    // since `answer_table_goal` was created by canonicalizing (and
    // possibly truncating, but we'll get to that later)
    // `selected_goal`. Then, whenever we reach a "free variable" in
    // `answer_table_goal`, say `?0`, we go to the instantiated answer
    // substitution and lookup the result (in this case, `u32`). We take
    // that result and unify it with whatever we find in `selected_goal`
    // (in this case, `?X`).
    //
    // Let's cover then some corner cases. First off, what is this
    // business of instantiating the answer? Well, the answer may not be a
    // simple type like `u32`, it could be a "family" of types, like
    // `for<type> Vec<?0>` -- i.e., `Vec<X>: Bar` for *any* `X`. In that
    // case, the instantiation would produce a substitution `[?0 :=
    // Vec<?Y>]` (note that the key is not affected, just the value). So
    // when we do the unification, instead of unifying `?X = u32`, we
    // would unify `?X = Vec<?Y>`.
    //
    // Next, truncation. One key thing is that the `answer_table_goal` may
    // not be *exactly* the same as the `selected_goal` -- we will
    // truncate it if it gets too deep. so, in our example, it may be that
    // instead of `answer_table_goal` being `for<type> T: Foo<Vec<?0>>`,
    // it could have been truncated to `for<type> T: Foo<?0>` (which is a
    // more general goal).  In that case, let's say that the answer is
    // still `[?0 = u32]`, meaning that `T: Foo<u32>` is true (which isn't
    // actually interesting to our original goal). When we do the zip
    // then, we will encounter `?0` in the `answer_table_goal` and pair
    // that with `Vec<?X>` from the pending goal. We will attempt to unify
    // `Vec<?X>` with `u32` (from the substitution), which will fail. That
    // failure will get propagated back up.

    fn apply_answer_subst(
        &mut self,
        ex_clause: ExClause<SlgContext>,
        selected_goal: &InEnvironment<Goal<ChalkIr>>,
        answer_table_goal: &Canonical<InEnvironment<Goal<ChalkIr>>>,
        canonical_answer_subst: &Canonical<ConstrainedSubst<ChalkIr>>,
    ) -> Fallible<ExClause<SlgContext>> {
        debug_heading!("apply_answer_subst()");
        debug!("ex_clause={:?}", ex_clause);
        debug!(
            "selected_goal={:?}",
            self.infer.normalize_deep(selected_goal)
        );
        debug!("answer_table_goal={:?}", answer_table_goal);
        debug!("canonical_answer_subst={:?}", canonical_answer_subst);

        // C' is now `answer`. No variables in common with G.
        let ConstrainedSubst {
            subst: answer_subst,

            // Assuming unification succeeds, we incorporate the
            // region constraints from the answer into the result;
            // we'll need them if this answer (which is not yet known
            // to be true) winds up being true, and otherwise (if the
            // answer is false or unknown) it doesn't matter.
            constraints: answer_constraints,
        } = self.infer.instantiate_canonical(&canonical_answer_subst);

        let mut ex_clause = AnswerSubstitutor::substitute(
            &mut self.infer,
            &selected_goal.environment,
            &answer_subst,
            ex_clause,
            &answer_table_goal.value,
            selected_goal,
        )?;
        ex_clause.constraints.extend(answer_constraints);
        Ok(ex_clause)
    }
}

struct AnswerSubstitutor<'t> {
    table: &'t mut InferenceTable,
    environment: &'t Environment<ChalkIr>,
    answer_subst: &'t Substitution<ChalkIr>,
    answer_binders: usize,
    pending_binders: usize,
    ex_clause: ExClause<SlgContext>,
}

impl<'t> AnswerSubstitutor<'t> {
    fn substitute<T: Zip<ChalkIr>>(
        table: &mut InferenceTable,
        environment: &Environment<ChalkIr>,
        answer_subst: &Substitution<ChalkIr>,
        ex_clause: ExClause<SlgContext>,
        answer: &T,
        pending: &T,
    ) -> Fallible<ExClause<SlgContext>> {
        let mut this = AnswerSubstitutor {
            table,
            environment,
            answer_subst,
            ex_clause,
            answer_binders: 0,
            pending_binders: 0,
        };
        Zip::zip_with(&mut this, answer, pending)?;
        Ok(this.ex_clause)
    }

    fn unify_free_answer_var(
        &mut self,
        answer_depth: usize,
        pending: ParameterKind<&Ty<ChalkIr>, &Lifetime<ChalkIr>>,
    ) -> Fallible<bool> {
        // This variable is bound in the answer, not free, so it
        // doesn't represent a reference into the answer substitution.
        if answer_depth < self.answer_binders {
            return Ok(false);
        }

        let answer_param = &self.answer_subst.parameters[answer_depth - self.answer_binders];

        let pending_shifted = pending
            .shifted_out(self.pending_binders)
            .unwrap_or_else(|_| {
                panic!(
                    "truncate extracted a pending value that references internal binder: {:?}",
                    pending,
                )
            });

        slg::into_ex_clause(
            self.table
                .unify(&self.environment, answer_param, &Parameter(pending_shifted))?,
            &mut self.ex_clause,
        );

        Ok(true)
    }

    /// When we encounter a variable in the answer goal, we first try
    /// `unify_free_answer_var`. Assuming that this fails, the
    /// variable must be a bound variable in the answer goal -- in
    /// that case, there should be a corresponding bound variable in
    /// the pending goal. This bit of code just checks that latter
    /// case.
    fn assert_matching_vars(&mut self, answer_depth: usize, pending_depth: usize) -> Fallible<()> {
        assert!(answer_depth < self.answer_binders);
        assert!(pending_depth < self.pending_binders);
        assert_eq!(
            self.answer_binders - answer_depth,
            self.pending_binders - pending_depth
        );
        Ok(())
    }
}

impl<'t> Zipper<ChalkIr> for AnswerSubstitutor<'t> {
    fn zip_tys(&mut self, answer: &Ty<ChalkIr>, pending: &Ty<ChalkIr>) -> Fallible<()> {
        if let Some(pending) = self.table.normalize_shallow(pending) {
            return Zip::zip_with(self, answer, &pending);
        }

        // If the answer has a variable here, then this is one of the
        // "inputs" to the subgoal table. We need to extract the
        // resulting answer that the subgoal found and unify it with
        // the value from our "pending subgoal".
        if let Ty::BoundVar(answer_depth) = answer {
            if self.unify_free_answer_var(*answer_depth, ParameterKind::Ty(pending))? {
                return Ok(());
            }
        }

        // Otherwise, the answer and the selected subgoal ought to be a perfect match for
        // one another.
        match (answer, pending) {
            (Ty::BoundVar(answer_depth), Ty::BoundVar(pending_depth)) => {
                self.assert_matching_vars(*answer_depth, *pending_depth)
            }

            (Ty::Apply(answer), Ty::Apply(pending)) => Zip::zip_with(self, answer, pending),

            (Ty::Dyn(answer), Ty::Dyn(pending)) | (Ty::Opaque(answer), Ty::Opaque(pending)) => {
                Zip::zip_with(self, answer, pending)
            }

            (Ty::Projection(answer), Ty::Projection(pending)) => {
                Zip::zip_with(self, answer, pending)
            }

            (Ty::ForAll(answer), Ty::ForAll(pending)) => {
                self.answer_binders += answer.num_binders;
                self.pending_binders += pending.num_binders;
                Zip::zip_with(self, &answer.ty, &pending.ty)?;
                self.answer_binders -= answer.num_binders;
                self.pending_binders -= pending.num_binders;
                Ok(())
            }

            (Ty::InferenceVar(_), _) | (_, Ty::InferenceVar(_)) => panic!(
                "unexpected inference var in answer `{:?}` or pending goal `{:?}`",
                answer, pending,
            ),

            (Ty::BoundVar(_), _)
            | (Ty::Apply(_), _)
            | (Ty::Dyn(_), _)
            | (Ty::Opaque(_), _)
            | (Ty::Projection(_), _)
            | (Ty::ForAll(_), _) => panic!(
                "structural mismatch between answer `{:?}` and pending goal `{:?}`",
                answer, pending,
            ),
        }
    }

    fn zip_lifetimes(
        &mut self,
        answer: &Lifetime<ChalkIr>,
        pending: &Lifetime<ChalkIr>,
    ) -> Fallible<()> {
        if let Some(pending) = self.table.normalize_lifetime(pending) {
            return Zip::zip_with(self, answer, &pending);
        }

        if let Lifetime::BoundVar(answer_depth) = answer {
            if self.unify_free_answer_var(*answer_depth, ParameterKind::Lifetime(pending))? {
                return Ok(());
            }
        }

        match (answer, pending) {
            (Lifetime::BoundVar(answer_depth), Lifetime::BoundVar(pending_depth)) => {
                self.assert_matching_vars(*answer_depth, *pending_depth)
            }

            (Lifetime::Placeholder(_), Lifetime::Placeholder(_)) => {
                assert_eq!(answer, pending);
                Ok(())
            }

            (Lifetime::InferenceVar(_), _) | (_, Lifetime::InferenceVar(_)) => panic!(
                "unexpected inference var in answer `{:?}` or pending goal `{:?}`",
                answer, pending,
            ),

            (Lifetime::BoundVar(_), _) | (Lifetime::Placeholder(_), _) => panic!(
                "structural mismatch between answer `{:?}` and pending goal `{:?}`",
                answer, pending,
            ),

            (Lifetime::Phantom(..), _) => unreachable!(),
        }
    }

    fn zip_binders<T>(&mut self, answer: &Binders<T>, pending: &Binders<T>) -> Fallible<()>
    where
        T: Zip<ChalkIr> + Fold<ChalkIr, Result = T>,
    {
        self.answer_binders += answer.binders.len();
        self.pending_binders += pending.binders.len();
        Zip::zip_with(self, &answer.value, &pending.value)?;
        self.answer_binders -= answer.binders.len();
        self.pending_binders -= pending.binders.len();
        Ok(())
    }
}
