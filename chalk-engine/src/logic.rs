use crate::forest::Forest;
use crate::normalize_deep::DeepNormalizer;
use crate::slg::{ResolventOps, SlgContext, SlgContextOps};
use crate::stack::{Stack, StackIndex};
use crate::strand::{CanonicalStrand, SelectedSubgoal, Strand};
use crate::table::{AnswerIndex, Table};
use crate::{
    Answer, AnswerMode, CompleteAnswer, ExClause, FlounderedSubgoal, Literal, Minimums, TableIndex,
    TimeStamp,
};

use chalk_ir::could_match::CouldMatch;
use chalk_ir::interner::Interner;
use chalk_ir::{
    AnswerSubst, Canonical, ConstrainedSubst, Constraints, FallibleOrFloundered, Floundered, Goal,
    GoalData, InEnvironment, NoSolution, ProgramClause, Substitution, UCanonical, UniverseMap,
};
use chalk_solve::clauses::program_clauses_that_could_match;
use chalk_solve::coinductive_goal::IsCoinductive;
use chalk_solve::infer::ucanonicalize::UCanonicalized;
use chalk_solve::infer::InferenceTable;
use chalk_solve::solve::truncate;
use tracing::{debug, debug_span, info, instrument};

type RootSearchResult<T> = Result<T, RootSearchFail>;

/// The different ways that a *root* search (which potentially pursues
/// many strands) can fail. A root search is one that begins with an
/// empty stack.
#[derive(Debug)]
pub(super) enum RootSearchFail {
    /// The table we were trying to solve cannot succeed.
    NoMoreSolutions,

    /// The table cannot be solved without more type information.
    Floundered,

    /// We did not find a solution, but we still have things to try.
    /// Repeat the request, and we'll give one of those a spin.
    ///
    /// (In a purely depth-first-based solver, like Prolog, this
    /// doesn't appear.)
    QuantumExceeded,

    /// A negative cycle was found. This is fail-fast, so even if there was
    /// possibly a solution (ambiguous or not), it may not have been found.
    NegativeCycle,

    /// The current answer index is not useful. Currently, this is returned
    /// because the current answer needs refining.
    InvalidAnswer,
}

/// This is returned when we try to select a subgoal for a strand.
#[derive(PartialEq)]
enum SubGoalSelection {
    /// A subgoal was successfully selected. It has already been checked
    /// to not be floundering. However, it may have an answer already, be
    /// coinductive, or create a cycle.
    Selected,

    /// This strand has no remaining subgoals, but there may still be
    /// floundered subgoals.
    NotSelected,
}

/// This is returned `on_no_remaining_subgoals`
enum NoRemainingSubgoalsResult {
    /// There is an answer available for the root table
    RootAnswerAvailable,

    /// There was a `RootSearchFail`
    RootSearchFail(RootSearchFail),

    // This was a success
    Success,
}

impl<I: Interner> Forest<I> {
    /// Returns an answer with a given index for the given table. This
    /// may require activating a strand and following it. It returns
    /// `Ok(answer)` if they answer is available and otherwise a
    /// `RootSearchFail` result.
    pub(super) fn root_answer(
        &mut self,
        context: &SlgContextOps<I>,
        table: TableIndex,
        answer_index: AnswerIndex,
    ) -> RootSearchResult<CompleteAnswer<I>> {
        let stack = Stack::default();

        let mut state = SolveState {
            forest: self,
            context,
            stack,
        };

        match state.ensure_root_answer(table, answer_index) {
            Ok(()) => {
                assert!(state.stack.is_empty());
                let answer = state.forest.answer(table, answer_index);
                if !answer.subst.value.delayed_subgoals.is_empty() {
                    return Err(RootSearchFail::InvalidAnswer);
                }
                Ok(CompleteAnswer {
                    subst: Canonical {
                        binders: answer.subst.binders.clone(),
                        value: ConstrainedSubst {
                            subst: answer.subst.value.subst.clone(),
                            constraints: answer.subst.value.constraints.clone(),
                        },
                    },
                    ambiguous: answer.ambiguous,
                })
            }
            Err(err) => Err(err),
        }
    }

    pub(super) fn any_future_answer(
        &self,
        table: TableIndex,
        mut answer_index: AnswerIndex,
        mut test: impl FnMut(&Substitution<I>) -> bool,
    ) -> bool {
        // Check any cached answers, starting at `answer_index`.
        while let Some(answer) = self.tables[table].answer(answer_index) {
            info!("answer cached = {:?}", answer);
            if test(&answer.subst.value.subst) {
                return true;
            }
            answer_index.increment();
        }

        // Check any unsolved strands, which may give further answers.
        self.tables[table]
            .strands()
            .any(|strand| test(&strand.value.ex_clause.subst))
    }

    pub(crate) fn answer(&self, table: TableIndex, answer: AnswerIndex) -> &Answer<I> {
        self.tables[table].answer(answer).unwrap()
    }

    fn canonicalize_strand_from(
        context: &SlgContextOps<I>,
        infer: &mut InferenceTable<I>,
        strand: &Strand<I>,
    ) -> CanonicalStrand<I> {
        infer
            .canonicalize(context.program().interner(), strand.clone())
            .quantified
    }

    /// Given a subgoal, converts the literal into u-canonical form
    /// and searches for an existing table. If one is found, it is
    /// returned, but otherwise a new table is created (and populated
    /// with its initial set of strands).
    ///
    /// Returns `None` if the literal cannot be converted into a table
    /// -- for example, this can occur when we have selected a
    /// negative literal with free existential variables, in which
    /// case the execution is said to "flounder".
    ///
    /// In terms of the NFTD paper, creating a new table corresponds
    /// to the *New Subgoal* step as well as the *Program Clause
    /// Resolution* steps.
    #[instrument(level = "debug", skip(self, context, infer))]
    fn get_or_create_table_for_subgoal(
        &mut self,
        context: &SlgContextOps<I>,
        infer: &mut InferenceTable<I>,
        subgoal: &Literal<I>,
    ) -> Option<(TableIndex, UniverseMap)> {
        // Subgoal abstraction:
        let (ucanonical_subgoal, universe_map) = match subgoal {
            Literal::Positive(subgoal) => {
                Forest::abstract_positive_literal(context, infer, subgoal.clone())?
            }
            Literal::Negative(subgoal) => {
                Forest::abstract_negative_literal(context, infer, subgoal.clone())?
            }
        };

        debug!(?ucanonical_subgoal, ?universe_map);

        let table = self.get_or_create_table_for_ucanonical_goal(context, ucanonical_subgoal);

        Some((table, universe_map))
    }

    /// Given a u-canonical goal, searches for an existing table. If
    /// one is found, it is returned, but otherwise a new table is
    /// created (and populated with its initial set of strands).
    ///
    /// In terms of the NFTD paper, creating a new table corresponds
    /// to the *New Subgoal* step as well as the *Program Clause
    /// Resolution* steps.
    #[instrument(level = "debug", skip(self, context))]
    pub(crate) fn get_or_create_table_for_ucanonical_goal(
        &mut self,
        context: &SlgContextOps<I>,
        goal: UCanonical<InEnvironment<Goal<I>>>,
    ) -> TableIndex {
        if let Some(table) = self.tables.index_of(&goal) {
            debug!(?table, "found existing table");
            return table;
        }

        info!(
            table = ?self.tables.next_index(),
            "creating new table with goal = {:#?}",
            goal,
        );
        let table = Self::build_table(context, self.tables.next_index(), goal);
        self.tables.insert(table)
    }

    /// When a table is first created, this function is invoked to
    /// create the initial set of strands. If the table represents a
    /// domain goal, these strands are created from the program
    /// clauses as well as the clauses found in the environment.  If
    /// the table represents a non-domain goal, such as `for<T> G`
    /// etc, then `simplify_goal` is invoked to create a strand
    /// that breaks the goal down.
    ///
    /// In terms of the NFTD paper, this corresponds to the *Program
    /// Clause Resolution* step being applied eagerly, as many times
    /// as possible.
    fn build_table(
        context: &SlgContextOps<I>,
        table_idx: TableIndex,
        goal: UCanonical<InEnvironment<Goal<I>>>,
    ) -> Table<I> {
        let coinductive = goal.is_coinductive(context.program());
        let mut table = Table::new(goal.clone(), coinductive);

        let goal_data = goal.canonical.value.goal.data(context.program().interner());
        match goal_data {
            GoalData::DomainGoal(domain_goal) => {
                let canon_domain_goal = UCanonical {
                    canonical: Canonical {
                        binders: goal.canonical.binders,
                        value: InEnvironment::new(
                            &goal.canonical.value.environment,
                            domain_goal.clone(),
                        ),
                    },
                    universes: goal.universes,
                };

                let db = context.program();
                let canon_goal = canon_domain_goal.canonical.value.goal.clone();
                let could_match = |c: &ProgramClause<I>| {
                    c.could_match(db.interner(), db.unification_database(), &canon_goal)
                };

                match program_clauses_that_could_match(db, &canon_domain_goal) {
                    Ok(mut clauses) => {
                        clauses.retain(could_match);
                        clauses.extend(db.custom_clauses().into_iter().filter(could_match));

                        let (infer, subst, goal) =
                            chalk_solve::infer::InferenceTable::from_canonical(
                                context.program().interner(),
                                canon_domain_goal.universes,
                                canon_domain_goal.canonical,
                            );

                        clauses.extend(
                            db.program_clauses_for_env(&goal.environment)
                                .iter(db.interner())
                                .cloned()
                                .filter(could_match),
                        );

                        let InEnvironment { environment, goal } = goal;

                        for clause in clauses {
                            info!("program clause = {:#?}", clause);
                            let mut infer = infer.clone();
                            if let Ok(resolvent) = infer.resolvent_clause(
                                context.unification_database(),
                                context.program().interner(),
                                &environment,
                                &goal,
                                &subst,
                                &clause,
                            ) {
                                info!("pushing initial strand with ex-clause: {:#?}", &resolvent,);
                                let strand = Strand {
                                    ex_clause: resolvent,
                                    selected_subgoal: None,
                                    last_pursued_time: TimeStamp::default(),
                                };
                                let canonical_strand =
                                    Self::canonicalize_strand_from(context, &mut infer, &strand);
                                table.enqueue_strand(canonical_strand);
                            }
                        }
                    }
                    Err(Floundered) => {
                        debug!(
                            table = ?table_idx,
                            "Marking table {:?} as floundered! (failed to create program clauses)",
                            table_idx
                        );
                        table.mark_floundered();
                    }
                }
            }

            _ => {
                let (mut infer, subst, InEnvironment { environment, goal }) =
                    chalk_solve::infer::InferenceTable::from_canonical(
                        context.program().interner(),
                        goal.universes,
                        goal.canonical,
                    );
                // The goal for this table is not a domain goal, so we instead
                // simplify it into a series of *literals*, all of which must be
                // true. Thus, in EWFS terms, we are effectively creating a
                // single child of the `A :- A` goal that is like `A :- B, C, D`
                // where B, C, and D are the simplified subgoals. You can think
                // of this as applying built-in "meta program clauses" that
                // reduce goals into Domain goals.
                match Self::simplify_goal(context, &mut infer, subst, environment, goal) {
                    FallibleOrFloundered::Ok(ex_clause) => {
                        info!(
                            ex_clause = ?DeepNormalizer::normalize_deep(
                                &mut infer,
                                context.program().interner(),
                                ex_clause.clone(),
                            ),
                            "pushing initial strand"
                        );
                        let strand = Strand {
                            ex_clause,
                            selected_subgoal: None,
                            last_pursued_time: TimeStamp::default(),
                        };
                        let canonical_strand =
                            Self::canonicalize_strand_from(context, &mut infer, &strand);
                        table.enqueue_strand(canonical_strand);
                    }
                    FallibleOrFloundered::NoSolution => {}
                    FallibleOrFloundered::Floundered => table.mark_floundered(),
                }
            }
        }

        table
    }

    /// Given a selected positive subgoal, applies the subgoal
    /// abstraction function to yield the canonical form that will be
    /// used to pick a table. Typically, this abstraction has no
    /// effect, and hence we are simply returning the canonical form
    /// of `subgoal`; but if the subgoal is getting too big, we return
    /// `None`, which causes the subgoal to flounder.
    fn abstract_positive_literal(
        context: &SlgContextOps<I>,
        infer: &mut InferenceTable<I>,
        subgoal: InEnvironment<Goal<I>>,
    ) -> Option<(UCanonical<InEnvironment<Goal<I>>>, UniverseMap)> {
        if truncate::needs_truncation(
            context.program().interner(),
            infer,
            context.max_size(),
            &subgoal,
        ) {
            None
        } else {
            let canonicalized_goal = infer
                .canonicalize(context.program().interner(), subgoal)
                .quantified;
            let UCanonicalized {
                quantified,
                universes,
            } = InferenceTable::u_canonicalize(context.program().interner(), &canonicalized_goal);
            Some((quantified, universes))
        }
    }

    /// Given a selected negative subgoal, the subgoal is "inverted"
    /// (see `InferenceTable<I, C>::invert`) and then potentially truncated
    /// (see `abstract_positive_literal`). The result subgoal is
    /// canonicalized. In some cases, this may return `None` and hence
    /// fail to yield a useful result, for example if free existential
    /// variables appear in `subgoal` (in which case the execution is
    /// said to "flounder").
    fn abstract_negative_literal(
        context: &SlgContextOps<I>,
        infer: &mut InferenceTable<I>,
        subgoal: InEnvironment<Goal<I>>,
    ) -> Option<(UCanonical<InEnvironment<Goal<I>>>, UniverseMap)> {
        // First, we have to check that the selected negative literal
        // is ground, and invert any universally quantified variables.
        //
        // DIVERGENCE -- In the RR paper, to ensure completeness, they
        // permit non-ground negative literals, but only consider
        // them to succeed when the target table has no answers at
        // all. This is equivalent inverting those free existentials
        // into universals, as discussed in the comments of
        // `invert`. This is clearly *sound*, but the completeness is
        // a subtle point. In particular, it can cause **us** to reach
        // false conclusions, because e.g. given a program like
        // (selected left-to-right):
        //
        //     not { ?T: Copy }, ?T = Vec<u32>
        //
        // we would select `not { ?T: Copy }` first. For this goal to
        // succeed we would require that -- effectively -- `forall<T>
        // { not { T: Copy } }`, which clearly doesn't hold. (In the
        // terms of RR, we would require that the table for `?T: Copy`
        // has failed before we can continue.)
        //
        // In the RR paper, this is acceptable because they assume all
        // of their input programs are both **normal** (negative
        // literals are selected after positive ones) and **safe**
        // (all free variables in negative literals occur in positive
        // literals). It is plausible for us to guarantee "normal"
        // form, we can reorder clauses as we need. I suspect we can
        // guarantee safety too, but I have to think about it.
        //
        // For now, we opt for the safer route of terming such
        // executions as floundering, because I think our use of
        // negative goals is sufficiently limited we can get away with
        // it. The practical effect is that we will judge more
        // executions as floundering than we ought to (i.e., where we
        // could instead generate an (imprecise) result). As you can
        // see a bit later, we also diverge in some other aspects that
        // affect completeness when it comes to subgoal abstraction.
        let inverted_subgoal = infer.invert(context.program().interner(), subgoal)?;

        if truncate::needs_truncation(
            context.program().interner(),
            infer,
            context.max_size(),
            &inverted_subgoal,
        ) {
            None
        } else {
            let canonicalized_goal = infer
                .canonicalize(context.program().interner(), inverted_subgoal)
                .quantified;
            let UCanonicalized {
                quantified,
                universes,
            } = InferenceTable::u_canonicalize(context.program().interner(), &canonicalized_goal);
            Some((quantified, universes))
        }
    }
}

pub(crate) struct SolveState<'forest, I: Interner> {
    forest: &'forest mut Forest<I>,
    context: &'forest SlgContextOps<'forest, I>,
    stack: Stack<I>,
}

impl<'forest, I: Interner> Drop for SolveState<'forest, I> {
    fn drop(&mut self) {
        if !self.stack.is_empty() {
            if let Some(active_strand) = self.stack.top().active_strand.take() {
                let table = self.stack.top().table;
                self.forest.tables[table].enqueue_strand(active_strand);
            }
            self.unwind_stack();
        }
    }
}

impl<'forest, I: Interner> SolveState<'forest, I> {
    /// Ensures that answer with the given index is available from the
    /// given table. Returns `Ok` if there is an answer.
    ///
    /// This function first attempts to fetch answer that is cached in
    /// the table. If none is found, then it will recursively search
    /// to find an answer.
    #[instrument(level = "info", skip(self))]
    fn ensure_root_answer(
        &mut self,
        initial_table: TableIndex,
        initial_answer: AnswerIndex,
    ) -> RootSearchResult<()> {
        info!(
            "table goal = {:#?}",
            self.forest.tables[initial_table].table_goal
        );
        // Check if this table has floundered.
        if self.forest.tables[initial_table].is_floundered() {
            return Err(RootSearchFail::Floundered);
        }
        // Check for a tabled answer.
        if let Some(answer) = self.forest.tables[initial_table].answer(initial_answer) {
            info!("answer cached = {:?}", answer);
            return Ok(());
        }

        // If no tabled answer is present, we ought to be requesting
        // the next available index.
        assert_eq!(
            self.forest.tables[initial_table].next_answer_index(),
            initial_answer
        );

        self.stack
            .push(initial_table, Minimums::MAX, self.forest.increment_clock());
        loop {
            let clock = self.stack.top().clock;
            // If we had an active strand, continue to pursue it
            let table = self.stack.top().table;
            let table_answer_mode = self.forest.tables[table].answer_mode;

            // We track when we last pursued each strand. If all the strands have been
            // pursued at this depth, then that means they all encountered a cycle.
            // We also know that if the first strand has been pursued at this depth,
            // then all have. Otherwise, an answer to any strand would have provided an
            // answer for the table.
            let forest = &mut self.forest;
            let next_strand = self.stack.top().active_strand.take().or_else(|| {
                forest.tables[table].dequeue_next_strand_that(|strand| {
                    let time_eligble = strand.value.last_pursued_time < clock;
                    let mode_eligble = match (table_answer_mode, strand.value.ex_clause.ambiguous) {
                        (AnswerMode::Complete, false) => true,
                        (AnswerMode::Complete, true) => false,
                        (AnswerMode::Ambiguous, _) => true,
                    };
                    time_eligble && mode_eligble
                })
            });
            match next_strand {
                Some(mut canonical_strand) => {
                    debug!("starting next strand = {:#?}", canonical_strand);

                    canonical_strand.value.last_pursued_time = clock;
                    match self.select_subgoal(&mut canonical_strand) {
                        SubGoalSelection::Selected => {
                            // A subgoal has been selected. We now check this subgoal
                            // table for an existing answer or if it's in a cycle.
                            // If neither of those are the case, a strand is selected
                            // and the next loop iteration happens.
                            self.on_subgoal_selected(canonical_strand)?;
                            continue;
                        }
                        SubGoalSelection::NotSelected => {
                            match self.on_no_remaining_subgoals(canonical_strand) {
                                NoRemainingSubgoalsResult::RootAnswerAvailable => return Ok(()),
                                NoRemainingSubgoalsResult::RootSearchFail(e) => return Err(e),
                                NoRemainingSubgoalsResult::Success => continue,
                            };
                        }
                    }
                }
                None => {
                    self.on_no_strands_left()?;
                    continue;
                }
            }
        }
    }

    /// This is called when an answer is available for the selected subgoal
    /// of the strand. First, if the selected subgoal is a `Positive` subgoal,
    /// it first clones the strand pursuing the next answer. Then, it merges the
    /// answer into the provided `Strand`.
    /// On success, `Ok` is returned and the `Strand` can be continued to process
    /// On failure, `Err` is returned and the `Strand` should be discarded
    fn merge_answer_into_strand(
        &mut self,
        infer: &mut InferenceTable<I>,
        strand: &mut Strand<I>,
    ) -> RootSearchResult<()> {
        // At this point, we know we have an answer for
        // the selected subgoal of the strand.
        // Now, we have to unify that answer onto the strand.

        // If this answer is ambiguous and we don't want ambiguous answers
        // yet, then we act like this is a floundered subgoal.
        let ambiguous = {
            let selected_subgoal = strand.selected_subgoal.as_ref().unwrap();
            let answer = self.forest.answer(
                selected_subgoal.subgoal_table,
                selected_subgoal.answer_index,
            );
            answer.ambiguous
        };
        if let AnswerMode::Complete = self.forest.tables[self.stack.top().table].answer_mode {
            if ambiguous {
                // FIXME: we could try to be a little bit smarter here. This can
                // really be split into cases:
                // 1) Cases where no amount of solving will cause this ambiguity to change.
                //    (e.g. `CannnotProve`)
                // 2) Cases where we may be able to get a better answer if we
                //    solve other subgoals first.
                //    (e.g. the `non_enumerable_traits_reorder` test)
                // We really only need to delay merging an ambiguous answer for
                // case 2. Do note, though, that even if we *do* merge the answer
                // case 1, we should stop solving this strand when in
                // `AnswerMode::Complete` since we wouldn't use this answer yet
                // *anyways*.

                // The selected subgoal returned an ambiguous answer, but we don't want that.
                // So, we treat this subgoal as floundered.
                let selected_subgoal = strand.selected_subgoal.take().unwrap();
                self.flounder_subgoal(&mut strand.ex_clause, selected_subgoal.subgoal_index);
                return Ok(());
            }
        }

        // If this subgoal was a `Positive` one, whichever way this
        // particular answer turns out, there may yet be *more* answers,
        // if this isn't a trivial substitution.
        // Enqueue that alternative for later.
        // NOTE: this is separate from the match below because we `take` the selected_subgoal
        // below, but here we keep it for the new `Strand`.
        let selected_subgoal = strand.selected_subgoal.as_ref().unwrap();
        if let Literal::Positive(_) = strand.ex_clause.subgoals[selected_subgoal.subgoal_index] {
            let answer = self.forest.answer(
                selected_subgoal.subgoal_table,
                selected_subgoal.answer_index,
            );
            if !self.forest.tables[selected_subgoal.subgoal_table]
                .table_goal
                .is_trivial_substitution(self.context.program().interner(), &answer.subst)
            {
                let mut next_subgoal = selected_subgoal.clone();
                next_subgoal.answer_index.increment();
                let next_strand = Strand {
                    ex_clause: strand.ex_clause.clone(),
                    selected_subgoal: Some(next_subgoal),
                    last_pursued_time: strand.last_pursued_time,
                };
                let table = self.stack.top().table;
                let canonical_next_strand =
                    Forest::canonicalize_strand_from(self.context, infer, &next_strand);
                self.forest.tables[table].enqueue_strand(canonical_next_strand);
            }
        }

        // Deselect and remove the selected subgoal, now that we have an answer for it.
        let selected_subgoal = strand.selected_subgoal.take().unwrap();
        let subgoal = strand
            .ex_clause
            .subgoals
            .remove(selected_subgoal.subgoal_index);
        match subgoal {
            Literal::Positive(subgoal) => {
                let SelectedSubgoal {
                    subgoal_index: _,
                    subgoal_table,
                    answer_index,
                    ref universe_map,
                } = selected_subgoal;
                use chalk_solve::infer::ucanonicalize::UniverseMapExt;
                let table_goal = universe_map.map_from_canonical(
                    self.context.program().interner(),
                    &self.forest.tables[subgoal_table].table_goal.canonical,
                );
                let answer_subst = universe_map.map_from_canonical(
                    self.context.program().interner(),
                    &self.forest.answer(subgoal_table, answer_index).subst,
                );
                match infer.apply_answer_subst(
                    self.context.program().interner(),
                    self.context.unification_database(),
                    &mut strand.ex_clause,
                    &subgoal,
                    &table_goal,
                    answer_subst,
                ) {
                    Ok(()) => {
                        let ex_clause = &mut strand.ex_clause;

                        // If the answer had was ambiguous, we have to
                        // ensure that `ex_clause` is also ambiguous. This is
                        // the SLG FACTOR operation, though NFTD just makes it
                        // part of computing the SLG resolvent.
                        if self.forest.answer(subgoal_table, answer_index).ambiguous {
                            debug!("Marking Strand as ambiguous because answer to (positive) subgoal was ambiguous");
                            ex_clause.ambiguous = true;
                        }

                        // Increment the answer time for the `ex_clause`. Floundered
                        // subgoals may be eligble to be pursued again.
                        ex_clause.answer_time.increment();

                        // Ok, we've applied the answer to this Strand.
                        Ok(())
                    }

                    // This answer led nowhere. Give up for now, but of course
                    // there may still be other strands to pursue, so return
                    // `QuantumExceeded`.
                    Err(NoSolution) => {
                        info!("answer not unifiable -> NoSolution");
                        // This strand as no solution. It is no longer active,
                        // so it dropped at the end of this scope.

                        // Now we want to propogate back to the up with `QuantumExceeded`
                        self.unwind_stack();
                        Err(RootSearchFail::QuantumExceeded)
                    }
                }
            }
            Literal::Negative(_) => {
                let SelectedSubgoal {
                    subgoal_index: _,
                    subgoal_table,
                    answer_index,
                    universe_map: _,
                } = selected_subgoal;
                // We got back an answer. This is bad, because we want
                // to disprove the subgoal, but it may be
                // "conditional" (maybe true, maybe not).
                let answer = self.forest.answer(subgoal_table, answer_index);

                // By construction, we do not expect negative subgoals
                // to have delayed subgoals. This is because we do not
                // need to permit `not { L }` where `L` is a
                // coinductive goal. We could improve this if needed,
                // but it keeps things simple.
                if !answer.subst.value.delayed_subgoals.is_empty() {
                    panic!("Negative subgoal had delayed_subgoals");
                }

                if !answer.ambiguous {
                    // We want to disproval the subgoal, but we
                    // have an unconditional answer for the subgoal,
                    // therefore we have failed to disprove it.
                    info!("found unconditional answer to neg literal -> NoSolution");

                    // This strand as no solution. By returning an Err,
                    // the caller should discard this `Strand`.

                    // Now we want to propogate back to the up with `QuantumExceeded`
                    self.unwind_stack();
                    return Err(RootSearchFail::QuantumExceeded);
                }

                // Otherwise, the answer is ambiguous. We can keep going,
                // but we have to mark our strand, too, as ambiguous.
                //
                // We want to disproval the subgoal, but we
                // have an unconditional answer for the subgoal,
                // therefore we have failed to disprove it.
                debug!(?strand, "Marking Strand as ambiguous because answer to (negative) subgoal was ambiguous");
                strand.ex_clause.ambiguous = true;

                // Strand is ambigious.
                Ok(())
            }
        }
    }

    /// This is called when the selected subgoal for a strand has floundered.
    /// We have to decide what this means for the strand.
    /// - If the strand was positively dependent on the subgoal, we flounder,
    ///   the subgoal, then return `false`. This strand may be able to be
    ///   retried later.
    /// - If the strand was negatively dependent on the subgoal, then strand
    ///   has led nowhere of interest and we return `true`. This strand should
    ///   be discarded.
    ///
    /// In other words, we return whether this strand flounders.
    fn propagate_floundered_subgoal(&mut self, strand: &mut CanonicalStrand<I>) -> bool {
        // This subgoal selection for the strand is finished, so take it
        let selected_subgoal = strand.value.selected_subgoal.take().unwrap();
        match strand.value.ex_clause.subgoals[selected_subgoal.subgoal_index] {
            Literal::Positive(_) => {
                // If this strand depends on this positively, then we can
                // come back to it later. So, we mark that subgoal as
                // floundered and yield `QuantumExceeded` up the stack

                // If this subgoal floundered, push it onto the
                // floundered list, along with the time that it
                // floundered. We'll try to solve some other subgoals
                // and maybe come back to it.
                self.flounder_subgoal(&mut strand.value.ex_clause, selected_subgoal.subgoal_index);

                false
            }
            Literal::Negative(_) => {
                // Floundering on a negative literal isn't like a
                // positive search: we only pursue negative literals
                // when we already know precisely the type we are
                // looking for. So there's no point waiting for other
                // subgoals, we'll never recover more information.
                //
                // In fact, floundering on negative searches shouldn't
                // normally happen, since there are no uninferred
                // variables in the goal, but it can with forall
                // goals:
                //
                //     forall<T> { not { T: Debug } }
                //
                // Here, the table we will be searching for answers is
                // `?T: Debug`, so it could well flounder.

                // This strand has no solution. It is no longer active,
                // so it dropped at the end of this scope.

                true
            }
        }
    }

    /// This is called if the selected subgoal for a `Strand` is
    /// a coinductive cycle.
    fn on_coinductive_subgoal(
        &mut self,
        mut canonical_strand: CanonicalStrand<I>,
    ) -> Result<(), RootSearchFail> {
        // This is a co-inductive cycle. That is, this table
        // appears somewhere higher on the stack, and has now
        // recursively requested an answer for itself. This
        // means that we have to delay this subgoal until we
        // reach a trivial self-cycle.

        // This subgoal selection for the strand is finished, so take it
        let selected_subgoal = canonical_strand.value.selected_subgoal.take().unwrap();
        match canonical_strand
            .value
            .ex_clause
            .subgoals
            .remove(selected_subgoal.subgoal_index)
        {
            Literal::Positive(subgoal) => {
                // We delay this subgoal
                let table = self.stack.top().table;
                assert!(
                    self.forest.tables[table].coinductive_goal
                        && self.forest.tables[selected_subgoal.subgoal_table].coinductive_goal
                );

                canonical_strand
                    .value
                    .ex_clause
                    .delayed_subgoals
                    .push(subgoal);

                self.stack.top().active_strand = Some(canonical_strand);
                Ok(())
            }
            Literal::Negative(_) => {
                // We don't allow coinduction for negative literals
                info!("found coinductive answer to negative literal");
                panic!("Coinductive cycle with negative literal");
            }
        }
    }

    /// This is called if the selected subgoal for `strand` is
    /// a positive, non-coinductive cycle.
    ///
    /// # Parameters
    ///
    /// * `strand` the strand from the top of the stack we are pursuing
    /// * `minimums` is the collected minimum clock times
    fn on_positive_cycle(
        &mut self,
        canonical_strand: CanonicalStrand<I>,
        minimums: Minimums,
    ) -> Result<(), RootSearchFail> {
        // We can't take this because we might need it later to clear the cycle
        let selected_subgoal = canonical_strand.value.selected_subgoal.as_ref().unwrap();

        match canonical_strand.value.ex_clause.subgoals[selected_subgoal.subgoal_index] {
            Literal::Positive(_) => {
                self.stack.top().cyclic_minimums.take_minimums(&minimums);
            }
            Literal::Negative(_) => {
                // We depend on `not(subgoal)`. For us to continue,
                // `subgoal` must be completely evaluated. Therefore,
                // we depend (negatively) on the minimum link of
                // `subgoal` as a whole -- it doesn't matter whether
                // it's pos or neg.
                let mins = Minimums {
                    positive: self.stack.top().clock,
                    negative: minimums.minimum_of_pos_and_neg(),
                };
                self.stack.top().cyclic_minimums.take_minimums(&mins);
            }
        }

        // Ok, we've taken the minimums from this cycle above. Now,
        // we just return the strand to the table. The table only
        // pulls strands if they have not been checked at this
        // depth.
        //
        // We also can't mark these and return early from this
        // because the stack above us might change.
        let table = self.stack.top().table;
        self.forest.tables[table].enqueue_strand(canonical_strand);

        // The strand isn't active, but the table is, so just continue
        Ok(())
    }

    /// Invoked after we've selected a (new) subgoal for the top-most
    /// strand. Attempts to pursue this selected subgoal.
    ///
    /// Returns:
    ///
    /// * `Ok` if we should keep searching.
    /// * `Err` if the subgoal failed in some way such that the strand can be abandoned.
    fn on_subgoal_selected(
        &mut self,
        mut canonical_strand: CanonicalStrand<I>,
    ) -> Result<(), RootSearchFail> {
        // This may be a newly selected subgoal or an existing selected subgoal.

        let SelectedSubgoal {
            subgoal_index: _,
            subgoal_table,
            answer_index,
            universe_map: _,
        } = *canonical_strand.value.selected_subgoal.as_ref().unwrap();

        debug!(
            ?subgoal_table,
            goal = ?self.forest.tables[subgoal_table].table_goal,
            "table selection {:?} with goal: {:?}",
            subgoal_table, self.forest.tables[subgoal_table].table_goal
        );

        // This is checked inside select_subgoal
        assert!(!self.forest.tables[subgoal_table].is_floundered());

        // Check for a tabled answer.
        if let Some(answer) = self.forest.tables[subgoal_table].answer(answer_index) {
            info!("answer cached = {:?}", answer);

            // There was a previous answer available for this table
            // We need to check if we can merge it into the current `Strand`.
            let num_universes = self.forest.tables[self.stack.top().table]
                .table_goal
                .universes;
            let (mut infer, _, mut strand) = chalk_solve::infer::InferenceTable::from_canonical(
                self.context.program().interner(),
                num_universes,
                canonical_strand.clone(),
            );
            match self.merge_answer_into_strand(&mut infer, &mut strand) {
                Err(e) => {
                    debug!(?strand, "could not merge into current strand");
                    drop(strand);
                    return Err(e);
                }
                Ok(_) => {
                    debug!(?strand, "merged answer into current strand");
                    canonical_strand =
                        Forest::canonicalize_strand_from(self.context, &mut infer, &strand);
                    self.stack.top().active_strand = Some(canonical_strand);
                    return Ok(());
                }
            }
        }

        // If no tabled answer is present, we ought to be requesting
        // the next available index.
        assert_eq!(
            self.forest.tables[subgoal_table].next_answer_index(),
            answer_index
        );

        // Next, check if the table is already active. If so, then we
        // have a recursive attempt.
        if let Some(cyclic_depth) = self.stack.is_active(subgoal_table) {
            info!("cycle detected at depth {:?}", cyclic_depth);
            let minimums = Minimums {
                positive: self.stack[cyclic_depth].clock,
                negative: TimeStamp::MAX,
            };

            if self.top_of_stack_is_coinductive_from(cyclic_depth) {
                debug!("table is coinductive");
                return self.on_coinductive_subgoal(canonical_strand);
            }

            debug!("table encountered a positive cycle");
            return self.on_positive_cycle(canonical_strand, minimums);
        }

        // We don't know anything about the selected subgoal table.
        // Set this strand as active and push it onto the stack.
        self.stack.top().active_strand = Some(canonical_strand);

        let cyclic_minimums = Minimums::MAX;
        self.stack.push(
            subgoal_table,
            cyclic_minimums,
            self.forest.increment_clock(),
        );
        Ok(())
    }

    /// This is called when there are no remaining subgoals for a strand, so
    /// it represents an answer. If the strand is ambiguous and we don't want
    /// it yet, we just enqueue it again to pick it up later. Otherwise, we
    /// add the answer from the strand onto the table.
    fn on_no_remaining_subgoals(
        &mut self,
        canonical_strand: CanonicalStrand<I>,
    ) -> NoRemainingSubgoalsResult {
        let ambiguous = canonical_strand.value.ex_clause.ambiguous;
        if let AnswerMode::Complete = self.forest.tables[self.stack.top().table].answer_mode {
            if ambiguous {
                // The strand can only return an ambiguous answer, but we don't
                // want that right now, so requeue and we'll deal with it later.
                self.forest.tables[self.stack.top().table].enqueue_strand(canonical_strand);
                return NoRemainingSubgoalsResult::RootSearchFail(RootSearchFail::QuantumExceeded);
            }
        }
        let floundered = !canonical_strand
            .value
            .ex_clause
            .floundered_subgoals
            .is_empty();
        if floundered {
            debug!("all remaining subgoals floundered for the table");
        } else {
            debug!("no remaining subgoals for the table");
        };
        match self.pursue_answer(canonical_strand) {
            Some(answer_index) => {
                debug!("answer is available");

                // We found an answer for this strand, and therefore an
                // answer for this table. Now, this table was either a
                // subgoal for another strand, or was the root table.
                let table = self.stack.top().table;
                match self.stack.pop_and_take_caller_strand() {
                    Some(caller_strand) => {
                        self.stack.top().active_strand = Some(caller_strand);
                        NoRemainingSubgoalsResult::Success
                    }
                    None => {
                        // That was the root table, so we are done --
                        // *well*, unless there were delayed
                        // subgoals. In that case, we want to evaluate
                        // those delayed subgoals to completion, so we
                        // have to create a fresh strand that will
                        // take them as goals. Note that we *still
                        // need the original answer in place*, because
                        // we might have to build on it (see the
                        // Delayed Trivial Self Cycle, Variant 3
                        // example).

                        let answer = self.forest.answer(table, answer_index);
                        if let Some(strand) = self.create_refinement_strand(table, answer) {
                            self.forest.tables[table].enqueue_strand(strand);
                        }

                        NoRemainingSubgoalsResult::RootAnswerAvailable
                    }
                }
            }
            None => {
                debug!("answer is not available (or not new)");

                // This strand led nowhere of interest. There might be *other*
                // answers on this table, but we don't care right now, we'll
                // try again at another time.

                // Now we yield with `QuantumExceeded`
                self.unwind_stack();
                NoRemainingSubgoalsResult::RootSearchFail(RootSearchFail::QuantumExceeded)
            }
        }
    }

    /// A "refinement" strand is used in coinduction. When the root
    /// table on the stack publishes an answer has delayed subgoals,
    /// we create a new strand that will attempt to prove out those
    /// delayed subgoals (the root answer here is not *special* except
    /// in so far as that there is nothing above it, and hence we know
    /// that the delayed subgoals (which resulted in some cycle) must
    /// be referring to a table that now has completed).
    ///
    /// Note that it is important for this to be a *refinement* strand
    /// -- meaning that the answer with delayed subgoals has been
    /// published. This is necessary because sometimes the strand must
    /// build on that very answer that it is refining. See Delayed
    /// Trivial Self Cycle, Variant 3.
    fn create_refinement_strand(
        &self,
        table: TableIndex,
        answer: &Answer<I>,
    ) -> Option<CanonicalStrand<I>> {
        // If there are no delayed subgoals, then there is no need for
        // a refinement strand.
        if answer.subst.value.delayed_subgoals.is_empty() {
            return None;
        }

        let num_universes = self.forest.tables[table].table_goal.universes;
        let (
            mut infer,
            _,
            AnswerSubst {
                subst,
                constraints,
                delayed_subgoals,
            },
        ) = chalk_solve::infer::InferenceTable::from_canonical(
            self.context.program().interner(),
            num_universes,
            answer.subst.clone(),
        );

        let delayed_subgoals = delayed_subgoals
            .into_iter()
            .map(Literal::Positive)
            .collect();

        let strand = Strand {
            ex_clause: ExClause {
                subst,
                ambiguous: answer.ambiguous,
                constraints: constraints
                    .as_slice(self.context.program().interner())
                    .to_vec(),
                subgoals: delayed_subgoals,
                delayed_subgoals: Vec::new(),
                answer_time: TimeStamp::default(),
                floundered_subgoals: Vec::new(),
            },
            selected_subgoal: None,
            last_pursued_time: TimeStamp::default(),
        };

        Some(Forest::canonicalize_strand_from(
            self.context,
            &mut infer,
            &strand,
        ))
    }

    fn on_no_strands_left(&mut self) -> Result<(), RootSearchFail> {
        let table = self.stack.top().table;
        debug!("no more strands available (or all cycles) for {:?}", table);

        // No more strands left to try! This is either because all
        // strands have failed, because all strands encountered a
        // cycle, or all strands have would give ambiguous answers.

        if self.forest.tables[table].strands_mut().count() == 0 {
            // All strands for the table T on the top of the stack
            // have **failed**. Hence we can pop it off the stack and
            // check what this means for the table T' that was just
            // below T on the stack (if any).
            debug!("no more strands available");
            let caller_strand = match self.stack.pop_and_borrow_caller_strand() {
                Some(s) => s,
                None => {
                    // T was the root table, so we are done.
                    debug!("no more solutions");
                    return Err(RootSearchFail::NoMoreSolutions);
                }
            };

            // This subgoal selection for the strand is finished, so take it
            let caller_selected_subgoal = caller_strand.value.selected_subgoal.take().unwrap();
            return match caller_strand.value.ex_clause.subgoals
                [caller_selected_subgoal.subgoal_index]
            {
                // T' wanted an answer from T, but none is
                // forthcoming.  Therefore, the active strand from T'
                // has failed and can be discarded.
                Literal::Positive(_) => {
                    debug!("discarding strand because positive literal");
                    self.stack.top().active_strand.take();
                    self.unwind_stack();
                    Err(RootSearchFail::QuantumExceeded)
                }

                // T' wanted there to be no answer from T, but none is forthcoming.
                Literal::Negative(_) => {
                    debug!("subgoal was proven because negative literal");

                    // There is no solution for this strand. But, this
                    // is what we want, so can remove this subgoal and
                    // keep going.
                    caller_strand
                        .value
                        .ex_clause
                        .subgoals
                        .remove(caller_selected_subgoal.subgoal_index);

                    // This strand is still active, so continue
                    Ok(())
                }
            };
        }

        // We can't consider this table as part of a cycle unless we've handled
        // all strands, not just non-ambiguous ones. See chalk#571.
        if let AnswerMode::Complete = self.forest.tables[table].answer_mode {
            debug!("Allowing ambiguous answers.");
            self.forest.tables[table].answer_mode = AnswerMode::Ambiguous;
            return Err(RootSearchFail::QuantumExceeded);
        }

        let clock = self.stack.top().clock;
        let cyclic_minimums = self.stack.top().cyclic_minimums;
        if cyclic_minimums.positive >= clock && cyclic_minimums.negative >= clock {
            debug!("cycle with no new answers");

            if cyclic_minimums.negative < TimeStamp::MAX {
                // This is a negative cycle.
                self.unwind_stack();
                return Err(RootSearchFail::NegativeCycle);
            }

            // If all the things that we recursively depend on have
            // positive dependencies on things below us in the stack,
            // then no more answers are forthcoming. We can clear all
            // the strands for those things recursively.
            let table = self.stack.top().table;
            let cyclic_strands = self.forest.tables[table].take_strands();
            self.clear_strands_after_cycle(cyclic_strands);

            // Now we yield with `QuantumExceeded`
            self.unwind_stack();
            Err(RootSearchFail::QuantumExceeded)
        } else {
            debug!("table part of a cycle");

            // This table resulted in a positive cycle, so we have
            // to check what this means for the subgoal containing
            // this strand
            let caller_strand = match self.stack.pop_and_borrow_caller_strand() {
                Some(s) => s,
                None => {
                    panic!("nothing on the stack but cyclic result");
                }
            };

            // We can't take this because we might need it later to clear the cycle
            let caller_selected_subgoal = caller_strand.value.selected_subgoal.as_ref().unwrap();
            match caller_strand.value.ex_clause.subgoals[caller_selected_subgoal.subgoal_index] {
                Literal::Positive(_) => {
                    self.stack
                        .top()
                        .cyclic_minimums
                        .take_minimums(&cyclic_minimums);
                }
                Literal::Negative(_) => {
                    // We depend on `not(subgoal)`. For us to continue,
                    // `subgoal` must be completely evaluated. Therefore,
                    // we depend (negatively) on the minimum link of
                    // `subgoal` as a whole -- it doesn't matter whether
                    // it's pos or neg.
                    let mins = Minimums {
                        positive: self.stack.top().clock,
                        negative: cyclic_minimums.minimum_of_pos_and_neg(),
                    };
                    self.stack.top().cyclic_minimums.take_minimums(&mins);
                }
            }

            // We can't pursue this strand anymore, so push it back onto the table
            let active_strand = self.stack.top().active_strand.take().unwrap();
            let table = self.stack.top().table;
            self.forest.tables[table].enqueue_strand(active_strand);

            // The strand isn't active, but the table is, so just continue
            Ok(())
        }
    }

    /// Unwinds the entire stack, returning all active strands back to
    /// their tables (this time at the end of the queue).
    fn unwind_stack(&mut self) {
        loop {
            match self.stack.pop_and_take_caller_strand() {
                Some(active_strand) => {
                    let table = self.stack.top().table;
                    self.forest.tables[table].enqueue_strand(active_strand);
                }

                None => return,
            }
        }
    }

    /// Invoked after we have determined that every strand in `table`
    /// encounters a cycle; `strands` is the set of strands (which
    /// have been moved out of the table). This method then
    /// recursively clears the active strands from the tables
    /// referenced in `strands`, since all of them must encounter
    /// cycles too.
    fn clear_strands_after_cycle(&mut self, strands: impl IntoIterator<Item = CanonicalStrand<I>>) {
        for strand in strands {
            let selected_subgoal = strand.value.selected_subgoal;
            let ex_clause = strand.value.ex_clause;
            let selected_subgoal = selected_subgoal.unwrap_or_else(|| {
                panic!(
                    "clear_strands_after_cycle invoked on strand in table \
                     without a selected subgoal: {:?}",
                    ex_clause,
                )
            });

            let strand_table = selected_subgoal.subgoal_table;
            let strands = self.forest.tables[strand_table].take_strands();
            self.clear_strands_after_cycle(strands);
        }
    }

    fn select_subgoal(&mut self, canonical_strand: &mut CanonicalStrand<I>) -> SubGoalSelection {
        loop {
            while canonical_strand.value.selected_subgoal.is_none() {
                if canonical_strand.value.ex_clause.subgoals.is_empty() {
                    if canonical_strand
                        .value
                        .ex_clause
                        .floundered_subgoals
                        .is_empty()
                    {
                        return SubGoalSelection::NotSelected;
                    }

                    self.reconsider_floundered_subgoals(&mut canonical_strand.value.ex_clause);

                    if canonical_strand.value.ex_clause.subgoals.is_empty() {
                        // All the subgoals of this strand floundered. We may be able
                        // to get helpful information from this strand still, but it
                        // will *always* be ambiguous, so mark it as so.
                        assert!(!canonical_strand
                            .value
                            .ex_clause
                            .floundered_subgoals
                            .is_empty());
                        canonical_strand.value.ex_clause.ambiguous = true;
                        return SubGoalSelection::NotSelected;
                    }

                    continue;
                }

                let subgoal_index =
                    SlgContext::next_subgoal_index(&canonical_strand.value.ex_clause);

                // Get or create table for this subgoal.
                let num_universes = self.forest.tables[self.stack.top().table]
                    .table_goal
                    .universes;
                let (mut infer, _, strand) = chalk_solve::infer::InferenceTable::from_canonical(
                    self.context.program().interner(),
                    num_universes,
                    canonical_strand.clone(),
                );
                match self.forest.get_or_create_table_for_subgoal(
                    self.context,
                    &mut infer,
                    &strand.ex_clause.subgoals[subgoal_index],
                ) {
                    Some((subgoal_table, universe_map)) => {
                        canonical_strand.value.selected_subgoal = Some(SelectedSubgoal {
                            subgoal_index,
                            subgoal_table,
                            universe_map,
                            answer_index: AnswerIndex::ZERO,
                        });
                    }

                    None => {
                        // If we failed to create a table for the subgoal,
                        // that is because we have a floundered negative
                        // literal.
                        self.flounder_subgoal(&mut canonical_strand.value.ex_clause, subgoal_index);
                    }
                }
            }

            let selected_subgoal_table = canonical_strand
                .value
                .selected_subgoal
                .as_ref()
                .unwrap()
                .subgoal_table;
            if self.forest.tables[selected_subgoal_table].is_floundered() {
                if self.propagate_floundered_subgoal(canonical_strand) {
                    // This strand will never lead anywhere of interest.
                    return SubGoalSelection::NotSelected;
                } else {
                    // This subgoal has floundered and has been marked.
                    // We previously would immediately mark the table as
                    // floundered too, and maybe come back to it. Now, we
                    // try to see if any other subgoals can be pursued first.
                    continue;
                }
            } else {
                return SubGoalSelection::Selected;
            }
        }
    }

    /// Invoked when a strand represents an **answer**. This means
    /// that the strand has no subgoals left. There are two possibilities:
    ///
    /// - the strand may represent an answer we have already found; in
    ///   that case, we can return `None`, as this
    ///   strand led nowhere of interest.
    /// - the strand may represent a new answer, in which case it is
    ///   added to the table and `Some(())` is returned.
    fn pursue_answer(&mut self, canonical_strand: CanonicalStrand<I>) -> Option<AnswerIndex> {
        let table = self.stack.top().table;
        let Canonical {
            binders,
            value: strand,
        } = canonical_strand;
        let ExClause {
            subst,
            constraints,
            ambiguous,
            subgoals,
            delayed_subgoals,
            answer_time: _,
            floundered_subgoals,
        } = strand.ex_clause;
        // If there are subgoals left, they should be followed
        assert!(subgoals.is_empty());
        // We can still try to get an ambiguous answer if there are floundered subgoals
        let floundered = !floundered_subgoals.is_empty();
        // So let's make sure that it *really* is an ambiguous answer (this should be set previously)
        assert!(!floundered || ambiguous);

        // FIXME: If there are floundered subgoals, we *could* potentially
        // actually check if the partial answers to any of these subgoals
        // conflict. But this requires that we think about whether they are
        // positive or negative subgoals. This duplicates some of the logic
        // in `merge_answer_into_strand`, so a bit of refactoring is needed.

        // If the answer gets too large, mark the table as floundered.
        // This is the *most conservative* course. There are a few alternatives:
        // 1) Replace the answer with a truncated version of it. (This was done
        //    previously, but turned out to be more complicated than we wanted and
        //    and a source of multiple bugs.)
        // 2) Mark this *strand* as floundered. We don't currently have a mechanism
        //    for this (only floundered subgoals), so implementing this is more
        //    difficult because we don't want to just *remove* this strand from the
        //    table, because that might make the table give `NoMoreSolutions`, which
        //    is *wrong*.
        // 3) Do something fancy with delayed subgoals, effectively delayed the
        //    truncated bits to a different strand (and a more "refined" answer).
        //    (This one probably needs more thought, but is here for "completeness")
        //
        // Ultimately, the current decision to flounder the entire table mostly boils
        // down to "it works as we expect for the current tests". And, we likely don't
        // even *need* the added complexity just for potentially more answers.
        if truncate::needs_truncation(
            self.context.program().interner(),
            &mut InferenceTable::new(),
            self.context.max_size(),
            &subst,
        ) {
            self.forest.tables[table].mark_floundered();
            return None;
        }

        let table_goal = &self.forest.tables[table].table_goal;

        let filtered_delayed_subgoals = delayed_subgoals
            .into_iter()
            .filter(|delayed_subgoal| {
                let canonicalized = InferenceTable::u_canonicalize(
                    self.context.program().interner(),
                    &chalk_ir::Canonical {
                        binders: binders.clone(),
                        value: delayed_subgoal.clone(),
                    },
                )
                .quantified;
                *table_goal != canonicalized
            })
            .collect();

        let subst = Canonical {
            binders,
            value: AnswerSubst {
                subst,
                constraints: Constraints::from_iter(self.context.program().interner(), constraints),
                delayed_subgoals: filtered_delayed_subgoals,
            },
        };
        debug!(?table, ?subst, ?floundered, "found answer");

        let answer = Answer { subst, ambiguous };

        // A "trivial" answer is one that is 'just true for all cases'
        // -- in other words, it gives no information back to the
        // caller. For example, `Vec<u32>: Sized` is "just true".
        // Such answers are important because they are the most
        // general case, and after we provide a trivial answer, no
        // further answers are useful -- therefore we can clear any
        // further pending strands (this is a "green cut", in
        // Prolog parlance).
        //
        // This optimization is *crucial* for performance: for
        // example, `projection_from_env_slow` fails miserably without
        // it. The reason is that we wind up (thanks to implied bounds)
        // with a clause like this:
        //
        // ```ignore
        // forall<T> { (<T as SliceExt>::Item: Clone) :- WF(T: SliceExt) }
        // ```
        //
        // we then apply that clause to `!1: Clone`, resulting in the
        // table goal `!1: Clone :- <?0 as SliceExt>::Item = !1,
        // WF(?0: SliceExt)`.  This causes us to **enumerate all types
        // `?0` that where `Slice<?0>` normalizes to `!1` -- this is
        // an infinite set of types, effectively. Interestingly,
        // though, we only need one and we are done, because (if you
        // look) our goal (`!1: Clone`) doesn't have any output
        // parameters.
        //
        // This is actually a kind of general case. Due to Rust's rule
        // about constrained impl type parameters, generally speaking
        // when we have some free inference variable (like `?0`)
        // within our clause, it must appear in the head of the
        // clause. This means that the values we create for it will
        // propagate up to the caller, and they will quickly surmise
        // that there is ambiguity and stop requesting more answers.
        // Indeed, the only exception to this rule about constrained
        // type parameters if with associated type projections, as in
        // the case above!
        //
        // (Actually, because of the trivial answer cut off rule, we
        // never even get to the point of asking the query above in
        // `projection_from_env_slow`.)
        //
        // However, there is one fly in the ointment: answers include
        // region constraints, and you might imagine that we could
        // find future answers that are also trivial but with distinct
        // sets of region constraints. **For this reason, we only
        // apply this green cut rule if the set of generated
        // constraints is empty.**
        //
        // The limitation on region constraints is quite a drag! We
        // can probably do better, though: for example, coherence
        // guarantees that, for any given set of types, only a single
        // impl ought to be applicable, and that impl can only impose
        // one set of region constraints. However, it's not quite that
        // simple, thanks to specialization as well as the possibility
        // of proving things from the environment (though the latter
        // is a *bit* suspect; e.g., those things in the environment
        // must be backed by an impl *eventually*).
        let is_trivial_answer = {
            self.forest.tables[table]
                .table_goal
                .is_trivial_substitution(self.context.program().interner(), &answer.subst)
                && answer
                    .subst
                    .value
                    .constraints
                    .is_empty(self.context.program().interner())
        };

        if let Some(answer_index) = self.forest.tables[table].push_answer(answer) {
            // See above, if we have a *complete* and trivial answer, we don't
            // want to follow any more strands
            if !ambiguous && is_trivial_answer {
                self.forest.tables[table].take_strands();
            }

            Some(answer_index)
        } else {
            info!("answer: not a new answer, returning None");
            None
        }
    }

    fn reconsider_floundered_subgoals(&mut self, ex_clause: &mut ExClause<I>) {
        info!("reconsider_floundered_subgoals(ex_clause={:#?})", ex_clause,);
        let ExClause {
            answer_time,
            subgoals,
            floundered_subgoals,
            ..
        } = ex_clause;
        for i in (0..floundered_subgoals.len()).rev() {
            if floundered_subgoals[i].floundered_time < *answer_time {
                let floundered_subgoal = floundered_subgoals.swap_remove(i);
                subgoals.push(floundered_subgoal.floundered_literal);
            }
        }
    }

    /// Removes the subgoal at `subgoal_index` from the strand's
    /// subgoal list and adds it to the strand's floundered subgoal
    /// list.
    fn flounder_subgoal(&self, ex_clause: &mut ExClause<I>, subgoal_index: usize) {
        let _s = debug_span!(
            "flounder_subgoal",
            answer_time = ?ex_clause.answer_time,
            subgoal = ?ex_clause.subgoals[subgoal_index],
        );
        let _s = _s.enter();

        let floundered_time = ex_clause.answer_time;
        let floundered_literal = ex_clause.subgoals.remove(subgoal_index);
        ex_clause.floundered_subgoals.push(FlounderedSubgoal {
            floundered_literal,
            floundered_time,
        });
        debug!(?ex_clause);
    }

    /// True if all the tables on the stack starting from `depth` and
    /// continuing until the top of the stack are coinductive.
    ///
    /// Example: Given a program like:
    ///
    /// ```notrust
    /// struct Foo { a: Option<Box<Bar>> }
    /// struct Bar { a: Option<Box<Foo>> }
    /// trait XXX { }
    /// impl<T: Send> XXX for T { }
    /// ```
    ///
    /// and then a goal of `Foo: XXX`, we would eventually wind up
    /// with a stack like this:
    ///
    /// | StackIndex | Table Goal  |
    /// | ---------- | ----------- |
    /// | 0          | `Foo: XXX`  |
    /// | 1          | `Foo: Send` |
    /// | 2          | `Bar: Send` |
    ///
    /// Here, the top of the stack is `Bar: Send`. And now we are
    /// asking `top_of_stack_is_coinductive_from(1)` -- the answer
    /// would be true, since `Send` is an auto trait, which yields a
    /// coinductive goal. But `top_of_stack_is_coinductive_from(0)` is
    /// false, since `XXX` is not an auto trait.
    pub(super) fn top_of_stack_is_coinductive_from(&self, depth: StackIndex) -> bool {
        StackIndex::iterate_range(self.stack.top_of_stack_from(depth)).all(|d| {
            let table = self.stack[d].table;
            self.forest.tables[table].coinductive_goal
        })
    }
}
