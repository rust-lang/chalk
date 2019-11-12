use crate::context::{prelude::*, Floundered, UnificationOps};
use crate::fallible::NoSolution;
use crate::forest::Forest;
use crate::hh::HhGoal;
use crate::stack::StackIndex;
use crate::strand::{SelectedSubgoal, Strand};
use crate::table::AnswerIndex;
use crate::Answer;
use crate::{ExClause, FlounderedSubgoal, Literal, Minimums, TableIndex, TimeStamp};
use std::mem;

type RootSearchResult<T> = Result<T, RootSearchFail>;

/// The different ways that a *root* search (which potentially pursues
/// many strands) can fail. A root search is one that begins with an
/// empty stack.
#[derive(Debug)]
pub(super) enum RootSearchFail {
    /// The subgoal we were trying to solve cannot succeed.
    NoMoreSolutions,

    /// The subgoal cannot be solved without more type information.
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
}

type TableCheckResult<T> = Result<T, TableCheckFail>;

/// The different ways that a table check can fail
#[derive(Debug)]
enum TableCheckFail {
    /// **All** avenues to solve the subgoal we were trying solve
    /// encountered a cyclic dependency on something higher up in the
    /// stack. The `Minimums` encodes how high up (and whether
    /// positive or negative).
    PositiveCycle(Minimums),
}

#[derive(Debug)]
enum EnsureSuccess {
    AnswerAvailable,
    Coinductive,
}

/// This is returned when we try to select a subgoal for a strand.
enum SubGoalSelection {
    /// A subgoal was successfully selected. It has already been checked
    /// to not be floundering. However, it may have an answer already, be
    /// coinductive, or create a cycle.
    Selected,

    /// This strand has no remaining subgoals.
    NoRemaingSubgoals,

    /// This strand has floundered. Either all the positive subgoals
    /// have floundered or a single negative subgoal has floundered.
    Floundered,
}

/// This is returned `on_no_remaining_subgoals`
enum NoRemainingSubgoalsResult {
    /// There is an answer available for the root table
    RootAnswerAvailable,

    /// There was a `RootSearchFail`
    RootSearchFail(RootSearchFail),

    // This was a success and the new depth is returned
    Success(StackIndex),
}

impl<C: Context> Forest<C> {
    /// Returns an answer with a given index for the given table. This
    /// may require activating a strand and following it. It returns
    /// `Ok(answer)` if they answer is available and otherwise a
    /// `RootSearchFail` result.
    pub(super) fn root_answer(
        &mut self,
        context: &impl ContextOps<C>,
        table: TableIndex,
        answer: AnswerIndex,
    ) -> RootSearchResult<&Answer<C>> {
        assert!(self.stack.is_empty());

        match self.ensure_answer(context, table, answer) {
            Ok(()) => Ok(self.tables[table].answer(answer).unwrap()),
            Err(err) => Err(err),
        }
    }

    pub(super) fn any_future_answer(
        &mut self,
        table: TableIndex,
        answer: AnswerIndex,
        mut test: impl FnMut(&C::Substitution) -> bool,
    ) -> bool {
        if let Some(answer) = self.tables[table].answer(answer) {
            info!("answer cached = {:?}", answer);
            return test(C::subst_from_canonical_subst(&answer.subst));
        }

        self.tables[table]
            .strands_mut()
            .any(|strand| test(&strand.ex_clause.subst))
    }

    /// Before we pursue a table, we need to check a couple things
    /// - Does it have an answer? -> `Some(Ok(EnsureSuccess::AnswerAvailable))`
    /// - Does it make a coinductive cycle? -> `Some(Ok(EnsureSuccess::Coinductive))`
    /// - Does it make a non-coinductive cycle? -> `Some(Err(TableCheckFail::PositiveCycle`
    /// - Otherwise, need to pursue -> `None`
    fn check_table(
        &mut self,
        table: TableIndex,
        answer: AnswerIndex,
    ) -> Option<TableCheckResult<EnsureSuccess>> {
        // Next, check for a tabled answer.
        if self.tables[table].answer(answer).is_some() {
            info!("answer cached = {:?}", self.tables[table].answer(answer));
            return Some(Ok(EnsureSuccess::AnswerAvailable));
        }

        // If no tabled answer is present, we ought to be requesting
        // the next available index.
        assert_eq!(self.tables[table].next_answer_index(), answer);

        // Next, check if the table is already active. If so, then we
        // have a recursive attempt.
        if let Some(depth) = self.stack.is_active(table) {
            info!("ensure_answer: cycle detected at depth {:?}", depth);

            if self.top_of_stack_is_coinductive_from(depth) {
                return Some(Ok(EnsureSuccess::Coinductive));
            }

            return Some(Err(TableCheckFail::PositiveCycle(Minimums {
                positive: self.stack[depth].clock,
                negative: TimeStamp::MAX,
            })));
        }

        return None;
    }

    /// Merges an answer into the provided `Strand`.
    /// On success, `Ok` is returned and the `Strand` can be continued to process
    /// On failure, `Err` is returned and the `Strand` should be discarded
    fn merge_answer_into_strand(
        &mut self,
        strand: &mut Strand<C>,
        depth: &mut StackIndex,
    ) -> RootSearchResult<()> {
        // At this point, we know we have an answer for
        // the selected subgoal of the strand.
        // Now, we have to unify that answer onto the strand.

        let selected_subgoal = strand.selected_subgoal.as_ref().unwrap();
        if let Literal::Positive(_) = strand.ex_clause.subgoals[selected_subgoal.subgoal_index] {
            // Whichever way this particular answer turns out, there may
            // yet be *more* answers. Enqueue that alternative for later.
            let mut next_subgoal = selected_subgoal.clone();
            next_subgoal.answer_index.increment();
            let next_strand = Strand {
                infer: strand.infer.clone(),
                ex_clause: strand.ex_clause.clone(),
                selected_subgoal: Some(next_subgoal),
                last_pursued_time: strand.last_pursued_time.clone(),
            };
            let table = self.stack[*depth].table;
            self.tables[table].push_strand(next_strand);
        }

        // We got an answer for this goal. Now it matters if this
        // was a positive or negative subgoal.

        // We are done with this subgoal selection, so we can take it
        let selected_subgoal = strand.selected_subgoal.take().unwrap();
        // We've already taken care of the case where we might be able
        // to pursue this subgoal later (in a different strand). For
        // this strand, we have an answer to this subgoal
        match strand
            .ex_clause
            .subgoals
            .remove(selected_subgoal.subgoal_index)
        {
            Literal::Positive(subgoal) => {
                // OK, let's follow *this* answer and see where it leads.

                let SelectedSubgoal {
                    subgoal_index: _,
                    subgoal_table,
                    answer_index,
                    ref universe_map,
                } = selected_subgoal;
                let table_goal = &C::map_goal_from_canonical(
                    &universe_map,
                    &C::canonical(&self.tables[subgoal_table].table_goal),
                );
                let answer_subst = &C::map_subst_from_canonical(
                    &universe_map,
                    &self.answer(subgoal_table, answer_index).subst,
                );
                match strand.infer.apply_answer_subst(
                    &mut strand.ex_clause,
                    &subgoal,
                    table_goal,
                    answer_subst,
                ) {
                    Ok(()) => {
                        let Strand {
                            infer,
                            ex_clause,
                            selected_subgoal: _,
                            last_pursued_time: _,
                        } = strand;

                        // If the answer had delayed literals, we have to
                        // ensure that `ex_clause` is also delayed. This is
                        // the SLG FACTOR operation, though NFTD just makes it
                        // part of computing the SLG resolvent.
                        if self.answer(subgoal_table, answer_index).ambiguous {
                            ex_clause.ambiguous = true;
                        }

                        // Increment time counter because we received a new answer.
                        ex_clause.current_time.increment();

                        // Apply answer abstraction.
                        self.truncate_returned(ex_clause, infer);

                        // Ok, we've applied the answer to this Strand.
                        return Ok(());
                    }

                    // This answer led nowhere. Give up for now, but of course
                    // there may still be other strands to pursue, so return
                    // `QuantumExceeded`.
                    Err(NoSolution) => {
                        info!("answer not unifiable -> NoSolution");
                        // This strand as no solution. It is no longer active,
                        // so it dropped at the end of this scope.

                        // Now we want to propogate back to the up with `QuantumExceeded`
                        self.unwind_stack(*depth);
                        return Err(RootSearchFail::QuantumExceeded);
                    }
                }
            }
            Literal::Negative(_) => {
                if self
                    .answer(
                        selected_subgoal.subgoal_table,
                        selected_subgoal.answer_index,
                    )
                    .is_unconditional()
                {
                    // We want to disproval the subgoal, but we
                    // have an unconditional answer for the subgoal,
                    // therefore we have failed to disprove it.
                    info!("found unconditional answer to neg literal -> NoSolution");

                    // This strand as no solution. By returning an Err,
                    // the caller should discard this `Strand`.

                    // Now we want to propogate back to the up with `QuantumExceeded`
                    self.unwind_stack(*depth);
                    return Err(RootSearchFail::QuantumExceeded);
                }

                // Got back a conditional answer. We neither succeed
                // nor fail yet, so just mark as ambiguous.
                //
                // This corresponds to the Delaying action in NFTD.
                // It also interesting to compare this with the EWFS
                // paper; there, when we encounter a delayed cached
                // answer in `negative_subgoal`, we do not immediately
                // convert to a delayed literal, but instead simply
                // stop. However, in EWFS, we *do* add the strand to
                // the table as a negative pending subgoal, and also
                // update the link to depend negatively on the
                // table. Then later, when all pending work from that
                // table is completed, all negative links are
                // converted to delays.
                //
                // Previously, this introduced a `delayed_literal` that
                // we could follow and potentially resolve later. However,
                // for simplicity, we now just mark the strand as ambiguous.

                // We've already removed the selected subgoal
                // Now we just have to mark this Strand as ambiguous
                strand.ex_clause.ambiguous = true;

                // Strand is ambigious.
                return Ok(());
            }
        };
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
    fn should_strand_flounder(&mut self, strand: &mut Strand<C>) -> bool {
        // This subgoal selection for the strand is finished, so take it
        let selected_subgoal = strand.selected_subgoal.take().unwrap();
        match strand.ex_clause.subgoals[selected_subgoal.subgoal_index] {
            Literal::Positive(_) => {
                // If this strand depends on this positively, then we can
                // come back to it later. So, we mark that subgoal as
                // floundered and yield `QuantumExceeded` up the stack

                // If this subgoal floundered, push it onto the
                // floundered list, along with the time that it
                // floundered. We'll try to solve some other subgoals
                // and maybe come back to it.
                self.flounder_subgoal(&mut strand.ex_clause, selected_subgoal.subgoal_index);

                return false;
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

                return true;
            }
        }
    }

    fn on_subgoal_selected(
        &mut self,
        mut depth: StackIndex,
        mut strand: Strand<C>,
    ) -> Result<StackIndex, RootSearchFail> {
        // This may be a newly selected subgoal or an existing selected subgoal.

        let SelectedSubgoal {
            subgoal_index: _,
            subgoal_table,
            answer_index,
            universe_map: _,
        } = *strand.selected_subgoal.as_ref().unwrap();

        debug!(
            "table selection {:?} with goal: {:#?}",
            subgoal_table, self.tables[subgoal_table].table_goal
        );

        // This is checked inside_select_subgoal
        assert!(!self.tables[subgoal_table].is_floundered());

        // Now, let's check the table
        match self.check_table(subgoal_table, answer_index) {
            Some(Ok(EnsureSuccess::AnswerAvailable)) => {
                debug!("previous answer available");
                // There was a previous answer available for this table
                // We need to check if
                match self.merge_answer_into_strand(&mut strand, &mut depth) {
                    Err(e) => {
                        debug!("could not merge into current strand");
                        drop(strand);
                        return Err(e);
                    }
                    Ok(_) => {
                        debug!("merged answer into current strand");
                        self.stack[depth].active_strand = Some(strand);
                        return Ok(depth);
                    }
                }
            }
            Some(Ok(EnsureSuccess::Coinductive)) => {
                debug!("table is coinductive");

                // This is a co-inductive cycle. That is, this table
                // appears somewhere higher on the stack, and has now
                // recursively requested an answer for itself. That
                // means that our subgoal is unconditionally true.

                // This subgoal selection for the strand is finished, so take it
                let selected_subgoal = strand.selected_subgoal.take().unwrap();
                match strand
                    .ex_clause
                    .subgoals
                    .remove(selected_subgoal.subgoal_index)
                {
                    Literal::Positive(_) => {
                        // We can drop this subgoal and pursue the next thing.
                        let table = self.stack[depth].table;
                        assert!(
                            self.tables[table].coinductive_goal
                                && self.tables[selected_subgoal.subgoal_table].coinductive_goal
                        );

                        self.stack[depth].active_strand = Some(strand);
                        return Ok(depth);
                    }
                    Literal::Negative(_) => {
                        // Our negative goal fails

                        // We discard the current strand
                        drop(strand);

                        // Now we yield with `QuantumExceeded`
                        self.unwind_stack(depth);
                        return Err(RootSearchFail::QuantumExceeded);
                    }
                }
            }
            Some(Err(TableCheckFail::PositiveCycle(minimums))) => {
                debug!("table encountered a positive cycle");

                // The selected subgoal causes a positive cycle

                // We can't take this because we might need it later to clear the cycle
                let selected_subgoal = strand.selected_subgoal.as_ref().unwrap();

                match strand.ex_clause.subgoals[selected_subgoal.subgoal_index] {
                    Literal::Positive(_) => {
                        self.stack[depth].cyclic_minimums.take_minimums(&minimums);
                    }
                    Literal::Negative(_) => {
                        // We depend on `not(subgoal)`. For us to continue,
                        // `subgoal` must be completely evaluated. Therefore,
                        // we depend (negatively) on the minimum link of
                        // `subgoal` as a whole -- it doesn't matter whether
                        // it's pos or neg.
                        let mins = Minimums {
                            positive: self.stack[depth].clock,
                            negative: minimums.minimum_of_pos_and_neg(),
                        };
                        self.stack[depth].cyclic_minimums.take_minimums(&mins);
                    }
                }

                // Ok, we've taken the minimums from this cycle above. Now,
                // we just return the strand to the table. The table only
                // pulls strands if they have not been checked this at this
                // depth.
                //
                // We also can't mark these and return early from this
                // because the stack above us might change.
                let table = self.stack[depth].table;
                self.tables[table].push_strand(strand);

                // The strand isn't active, but the table is, so just continue
                return Ok(depth);
            }
            None => {
                // We don't know anything about the selected subgoal table.
                // Set this strand as active and push it onto the stack.
                self.stack[depth].active_strand = Some(strand);

                let clock = self.increment_clock();
                let cyclic_minimums = Minimums::MAX;
                depth = self.stack.push(subgoal_table, clock, cyclic_minimums);
                return Ok(depth);
            }
        }
    }

    fn on_no_remaining_subgoals(
        &mut self,
        mut depth: StackIndex,
        strand: Strand<C>,
    ) -> NoRemainingSubgoalsResult {
        debug!("no remaining subgoals for the table");

        match self.pursue_answer(depth, strand) {
            Some(()) => {
                debug!("answer is available");

                // We found an answer for this strand, and therefore an
                // answer for this table. Now, this table was either a
                // subgoal for another strand, or was the root table.
                let mut strand = {
                    let prev_index = self.stack.pop(depth);
                    if let Some(index) = prev_index {
                        // The table was a subgoal for another strand,
                        // which is still active.
                        // We need to merge the answer into it.
                        depth = index;
                        self.stack[depth].active_strand.take().unwrap()
                    } else {
                        // That was the root table, so we are done.
                        return NoRemainingSubgoalsResult::RootAnswerAvailable;
                    }
                };

                match self.merge_answer_into_strand(&mut strand, &mut depth) {
                    Err(e) => {
                        drop(strand);
                        return NoRemainingSubgoalsResult::RootSearchFail(e);
                    }
                    Ok(_) => {
                        self.stack[depth].active_strand = Some(strand);
                        return NoRemainingSubgoalsResult::Success(depth);
                    }
                }
            }
            None => {
                debug!("answer is not available (or not new)");

                // This table ned nowhere of interest

                // Now we yield with `QuantumExceeded`
                self.unwind_stack(depth);
                return NoRemainingSubgoalsResult::RootSearchFail(RootSearchFail::QuantumExceeded);
            }
        };
    }

    fn on_subgoal_selection_flounder(
        &mut self,
        mut depth: StackIndex,
        strand: Strand<C>,
    ) -> RootSearchFail {
        debug!("all subgoals floundered");

        // We were unable to select a subgoal for this strand
        // because all of them had floundered or because any one
        // that we dependended on negatively floundered

        // We discard this strand because it led nowhere of interest
        drop(strand);

        loop {
            // This table is marked as floundered
            let table = self.stack[depth].table;
            debug!("Marking table {:?} as floundered!", table);
            self.tables[table].mark_floundered();

            let prev_index = self.stack.pop(depth);
            if let Some(index) = prev_index {
                // The table was a subgoal for another strand,
                // which is still active.
                // We need to decide what a floundered subgoal means
                depth = index;
            } else {
                // That was the root table, so we are done.
                return RootSearchFail::Floundered;
            }
            let mut strand = self.stack[depth].active_strand.take().unwrap();

            match self.should_strand_flounder(&mut strand) {
                false => {
                    // We want to maybe pursue this strand later
                    let table = self.stack[depth].table;
                    self.tables[table].push_strand(strand);

                    // Now we yield with `QuantumExceeded`
                    self.unwind_stack(depth);
                    return RootSearchFail::QuantumExceeded;
                }
                true => {
                    // This strand will never lead anywhere of interest
                    drop(strand);

                    // Because a subgoal that we depended on negatively floundered,
                    // this table flounders (continue loop).
                }
            }
        }
    }

    fn on_no_strands_left(&mut self, mut depth: StackIndex) -> Result<StackIndex, RootSearchFail> {
        debug!("no more strands available (or all cycles)");

        // No more strands left to try! That means either we started
        // with no strands, or all available strands encountered a cycle.

        let table = self.stack[depth].table;
        if self.tables[table].strands_mut().count() == 0 {
            debug!("no more strands available");

            // We started with no strands!

            // This table has no solutions, so we have to check what
            // this means for the subgoal containing this strand
            let strand = {
                let prev_index = self.stack.pop(depth);
                if let Some(index) = prev_index {
                    // The table was a subgoal for another strand,
                    // which is still active.
                    depth = index;
                    self.stack[depth].active_strand.as_mut().unwrap()
                } else {
                    debug!("no more solutions");

                    // That was the root table, so we are done.
                    return Err(RootSearchFail::NoMoreSolutions);
                }
            };

            // This subgoal selection for the strand is finished, so take it
            let selected_subgoal = strand.selected_subgoal.take().unwrap();
            match strand.ex_clause.subgoals[selected_subgoal.subgoal_index] {
                Literal::Positive(_) => {
                    debug!("discarding strand because positive literal");

                    // There is no solution for this strand, so discard it
                    self.stack[depth].active_strand.take();

                    // Now we yield with `QuantumExceeded`
                    self.unwind_stack(depth);
                    return Err(RootSearchFail::QuantumExceeded);
                }
                Literal::Negative(_) => {
                    debug!("subgoal was proven because negative literal");

                    // There is no solution for this strand
                    // But, this is what we want, so can remove
                    // this subgoal
                    strand
                        .ex_clause
                        .subgoals
                        .remove(selected_subgoal.subgoal_index);

                    // This strand is still active, so continue
                    return Ok(depth);
                }
            }
        }

        let clock = self.stack[depth].clock;
        let cyclic_minimums = self.stack[depth].cyclic_minimums;
        if cyclic_minimums.positive >= clock && cyclic_minimums.negative >= clock {
            debug!("cycle with no new answers");

            if cyclic_minimums.negative < TimeStamp::MAX {
                // This is a negative cycle.
                self.unwind_stack(depth);
                return Err(RootSearchFail::NegativeCycle);
            }

            // If all the things that we recursively depend on have
            // positive dependencies on things below us in the stack,
            // then no more answers are forthcoming. We can clear all
            // the strands for those things recursively.
            let table = self.stack[depth].table;
            let cyclic_strands = self.tables[table].take_strands();
            self.clear_strands_after_cycle(cyclic_strands);

            // Now we yield with `QuantumExceeded`
            self.unwind_stack(depth);
            return Err(RootSearchFail::QuantumExceeded);
        } else {
            debug!("table part of a cycle");

            // This table resulted in a positive cycle, so we have
            // to check what this means for the subgoal containing
            // this strand
            let strand = {
                let prev_index = self.stack.pop(depth);
                if let Some(index) = prev_index {
                    // The table was a subgoal for another strand,
                    // which is still active.
                    // We need to merge the answer into it.
                    depth = index;
                    self.stack[depth].active_strand.as_mut().unwrap()
                } else {
                    panic!("nothing on the stack but cyclic result");
                }
            };

            // We can't take this because we might need it later to clear the cycle
            let selected_subgoal = strand.selected_subgoal.as_ref().unwrap();
            match strand.ex_clause.subgoals[selected_subgoal.subgoal_index] {
                Literal::Positive(_) => {
                    self.stack[depth]
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
                        positive: self.stack[depth].clock,
                        negative: cyclic_minimums.minimum_of_pos_and_neg(),
                    };
                    self.stack[depth].cyclic_minimums.take_minimums(&mins);
                }
            }

            // We can't pursue this strand anymore, so push it back onto the table
            let active_strand = self.stack[depth].active_strand.take().unwrap();
            let table = self.stack[depth].table;
            self.tables[table].push_strand(active_strand);

            // The strand isn't active, but the table is, so just continue
            return Ok(depth);
        }
    }

    fn unwind_stack(&mut self, mut depth: StackIndex) {
        loop {
            let next_index = self.stack.pop(depth);
            if let Some(index) = next_index {
                depth = index;
            } else {
                return;
            }

            let active_strand = self.stack[depth].active_strand.take().unwrap();
            let table = self.stack[depth].table;
            self.tables[table].push_strand(active_strand);
        }
    }

    /// Ensures that answer with the given index is available from the
    /// given table. Returns `Ok` if there is an answer.
    ///
    /// This function first attempts to fetch answer that is cached in
    /// the table. If none is found, then it will recursively search
    /// to find an answer.
    fn ensure_answer(
        &mut self,
        context: &impl ContextOps<C>,
        initial_table: TableIndex,
        initial_answer: AnswerIndex,
    ) -> RootSearchResult<()> {
        info!(
            "ensure_answer(table={:?}, answer={:?})",
            initial_table, initial_answer
        );
        info!("table goal = {:#?}", self.tables[initial_table].table_goal);
        // First, check if this table has floundered.
        if self.tables[initial_table].is_floundered() {
            return Err(RootSearchFail::Floundered);
        }
        match self.check_table(initial_table, initial_answer) {
            Some(Ok(EnsureSuccess::AnswerAvailable)) => return Ok(()),
            Some(Ok(EnsureSuccess::Coinductive)) | Some(Err(TableCheckFail::PositiveCycle(..))) => {
                panic!("cycle at root")
            }
            None => {}
        }
        let next_clock = self.increment_clock();
        let mut depth = self.stack.push(initial_table, next_clock, Minimums::MAX);
        loop {
            // FIXME: use depth for debug/info printing

            let clock = self.stack[depth].clock;
            // If we had an active strand, continue to pursue it
            let table = self.stack[depth].table;

            // We track when we last pursued each strand. If all the strands have been
            // pursued at this depth, then that means they all encountered a cycle.
            // We also know that if the first strand has been pursued at this depth,
            // then all have. Otherwise, an answer to any strand would have provided an
            // answer for the table.
            let next_strand = self.stack[depth].active_strand.take().or_else(|| {
                self.tables[table].pop_next_strand_if(|strand| strand.last_pursued_time < clock)
            });
            match next_strand {
                Some(mut strand) => {
                    debug!("next strand: {:#?}", strand);

                    strand.last_pursued_time = clock;
                    match self.select_subgoal(context, &mut strand) {
                        SubGoalSelection::Selected => {
                            // A subgoal has been selected. We now check this subgoal
                            // table for an existing answer or if it's in a cycle.
                            // If neither of those are the case, a strand is selected
                            // and the next loop iteration happens.
                            depth = self.on_subgoal_selected(depth, strand)?;
                            continue;
                        }
                        SubGoalSelection::NoRemaingSubgoals => {
                            depth = match self.on_no_remaining_subgoals(depth, strand) {
                                NoRemainingSubgoalsResult::RootAnswerAvailable => return Ok(()),
                                NoRemainingSubgoalsResult::RootSearchFail(e) => return Err(e),
                                NoRemainingSubgoalsResult::Success(depth) => depth,
                            };
                            continue;
                        }
                        SubGoalSelection::Floundered => {
                            // The strand floundered when trying to select a subgoal.
                            // This will always return a `RootSearchFail`, either because the
                            // root table floundered or we yield with `QuantumExceeded`.
                            return Err(self.on_subgoal_selection_flounder(depth, strand));
                        }
                    }
                }
                None => {
                    depth = self.on_no_strands_left(depth)?;
                    continue;
                }
            }
        }
    }

    pub(crate) fn answer(&self, table: TableIndex, answer: AnswerIndex) -> &Answer<C> {
        self.tables[table].answer(answer).unwrap()
    }

    /// Invoked after we have determined that every strand in `table`
    /// encounters a cycle; `strands` is the set of strands (which
    /// have been moved out of the table). This method then
    /// recursively clears the active strands from the tables
    /// referenced in `strands`, since all of them must encounter
    /// cycles too.
    fn clear_strands_after_cycle(&mut self, strands: impl IntoIterator<Item = Strand<C>>) {
        for strand in strands {
            let Strand {
                mut infer,
                ex_clause,
                selected_subgoal,
                last_pursued_time: _,
            } = strand;
            let selected_subgoal = selected_subgoal.unwrap_or_else(|| {
                panic!(
                    "clear_strands_after_cycle invoked on strand in table \
                     without a selected subgoal: {:?}",
                    infer.debug_ex_clause(&ex_clause),
                )
            });

            let strand_table = selected_subgoal.subgoal_table;
            let strands = self.tables[strand_table].take_strands();
            self.clear_strands_after_cycle(strands);
        }
    }

    fn select_subgoal(
        &mut self,
        context: &impl ContextOps<C>,
        strand: &mut Strand<C>,
    ) -> SubGoalSelection {
        loop {
            while strand.selected_subgoal.is_none() {
                if strand.ex_clause.subgoals.len() == 0 {
                    if strand.ex_clause.floundered_subgoals.is_empty() {
                        return SubGoalSelection::NoRemaingSubgoals;
                    }

                    self.reconsider_floundered_subgoals(&mut strand.ex_clause);

                    if strand.ex_clause.subgoals.is_empty() {
                        assert!(!strand.ex_clause.floundered_subgoals.is_empty());
                        return SubGoalSelection::Floundered;
                    }

                    continue;
                }

                let subgoal_index = strand.infer.next_subgoal_index(&strand.ex_clause);

                // Get or create table for this subgoal.
                match self.get_or_create_table_for_subgoal(
                    context,
                    &mut strand.infer,
                    &strand.ex_clause.subgoals[subgoal_index],
                ) {
                    Some((subgoal_table, universe_map)) => {
                        strand.selected_subgoal = Some(SelectedSubgoal {
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
                        self.flounder_subgoal(&mut strand.ex_clause, subgoal_index);
                    }
                }
            }

            if self.tables[strand.selected_subgoal.as_ref().unwrap().subgoal_table].is_floundered()
            {
                match self.should_strand_flounder(strand) {
                    false => {
                        // This subgoal has floundered and has been marked.
                        // We previously would immediately mark the table as
                        // floundered too, and maybe come back to it. Now, we
                        // try to see if any other subgoals can be pursued first.
                        continue;
                    }
                    true => {
                        // This strand will never lead anywhere of interest.
                        return SubGoalSelection::Floundered;
                    }
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
    fn pursue_answer(&mut self, depth: StackIndex, strand: Strand<C>) -> Option<()> {
        let table = self.stack[depth].table;
        let Strand {
            mut infer,
            ex_clause:
                ExClause {
                    subst,
                    constraints,
                    ambiguous,
                    subgoals,
                    current_time: _,
                    floundered_subgoals,
                },
            selected_subgoal: _,
            last_pursued_time: _,
        } = strand;
        assert!(subgoals.is_empty());
        assert!(floundered_subgoals.is_empty());

        let answer_subst = infer.canonicalize_constrained_subst(subst, constraints);
        debug!("answer: table={:?}, answer_subst={:?}", table, answer_subst);

        let answer = Answer {
            subst: answer_subst,
            ambiguous: ambiguous,
        };

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
            !answer.ambiguous
                && C::is_trivial_substitution(&self.tables[table].table_goal, &answer.subst)
                && C::empty_constraints(&answer.subst)
        };

        if self.tables[table].push_answer(answer) {
            if is_trivial_answer {
                self.tables[table].take_strands();
            }

            Some(())
        } else {
            info!("answer: not a new answer, returning None");
            None
        }
    }

    fn reconsider_floundered_subgoals(&mut self, ex_clause: &mut ExClause<impl Context>) {
        info!("reconsider_floundered_subgoals(ex_clause={:#?})", ex_clause,);
        let ExClause {
            current_time,
            subgoals,
            floundered_subgoals,
            ..
        } = ex_clause;
        for i in (0..floundered_subgoals.len()).rev() {
            if floundered_subgoals[i].floundered_time < *current_time {
                let floundered_subgoal = floundered_subgoals.swap_remove(i);
                subgoals.push(floundered_subgoal.floundered_literal);
            }
        }
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
    fn get_or_create_table_for_subgoal(
        &mut self,
        context: &impl ContextOps<C>,
        infer: &mut dyn InferenceTable<C>,
        subgoal: &Literal<C>,
    ) -> Option<(TableIndex, C::UniverseMap)> {
        debug_heading!("get_or_create_table_for_subgoal(subgoal={:?})", subgoal);

        // Subgoal abstraction:
        let (ucanonical_subgoal, universe_map) = match subgoal {
            Literal::Positive(subgoal) => self.abstract_positive_literal(infer, subgoal),
            Literal::Negative(subgoal) => self.abstract_negative_literal(infer, subgoal)?,
        };

        debug!("ucanonical_subgoal={:?}", ucanonical_subgoal);
        debug!("universe_map={:?}", universe_map);

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
    pub(crate) fn get_or_create_table_for_ucanonical_goal(
        &mut self,
        context: &impl ContextOps<C>,
        goal: C::UCanonicalGoalInEnvironment,
    ) -> TableIndex {
        debug_heading!("get_or_create_table_for_ucanonical_goal({:?})", goal);

        if let Some(table) = self.tables.index_of(&goal) {
            debug!("found existing table {:?}", table);
            return table;
        }

        info_heading!(
            "creating new table {:?} and goal {:#?}",
            self.tables.next_index(),
            goal
        );
        let coinductive_goal = context.is_coinductive(&goal);
        let table = self.tables.insert(goal, coinductive_goal);
        self.push_initial_strands(context, table);
        table
    }

    /// When a table is first created, this function is invoked to
    /// create the initial set of strands. If the table represents a
    /// domain goal, these strands are created from the program
    /// clauses as well as the clauses found in the environment.  If
    /// the table represents a non-domain goal, such as `for<T> G`
    /// etc, then `simplify_hh_goal` is invoked to create a strand
    /// that breaks the goal down.
    ///
    /// In terms of the NFTD paper, this corresponds to the *Program
    /// Clause Resolution* step being applied eagerly, as many times
    /// as possible.
    fn push_initial_strands(&mut self, context: &impl ContextOps<C>, table: TableIndex) {
        // Instantiate the table goal with fresh inference variables.
        let table_goal = self.tables[table].table_goal.clone();
        context.instantiate_ucanonical_goal(&table_goal, |infer, subst, environment, goal| {
            self.push_initial_strands_instantiated(context, table, infer, subst, environment, goal);
        });
    }

    fn push_initial_strands_instantiated(
        &mut self,
        context: &impl ContextOps<C>,
        table: TableIndex,
        mut infer: C::InferenceTable,
        subst: C::Substitution,
        environment: C::Environment,
        goal: C::Goal,
    ) {
        let table_ref = &mut self.tables[table];
        match infer.into_hh_goal(goal) {
            HhGoal::DomainGoal(domain_goal) => {
                match context.program_clauses(&environment, &domain_goal, &mut infer) {
                    Ok(clauses) => {
                        for clause in clauses {
                            info!("program clause = {:#?}", clause);
                            let mut infer = infer.clone();
                            if let Ok(resolvent) =
                                infer.resolvent_clause(&environment, &domain_goal, &subst, &clause)
                            {
                                info!("pushing initial strand with ex-clause: {:#?}", &resolvent,);
                                let strand = Strand {
                                    infer,
                                    ex_clause: resolvent,
                                    selected_subgoal: None,
                                    last_pursued_time: TimeStamp::default(),
                                };
                                table_ref.push_strand(strand);
                            }
                        }
                    }
                    Err(Floundered) => {
                        debug!("Marking table {:?} as floundered!", table);
                        table_ref.mark_floundered();
                    }
                }
            }

            hh_goal => {
                // `canonical_goal` is an HH goal. We can simplify it
                // into a series of *literals*, all of which must be
                // true. Thus, in EWFS terms, we are effectively
                // creating a single child of the `A :- A` goal that
                // is like `A :- B, C, D` where B, C, and D are the
                // simplified subgoals. You can think of this as
                // applying built-in "meta program clauses" that
                // reduce HH goals into Domain goals.
                if let Ok(ex_clause) =
                    Self::simplify_hh_goal(&mut infer, subst, environment, hh_goal)
                {
                    info!(
                        "pushing initial strand with ex-clause: {:#?}",
                        infer.debug_ex_clause(&ex_clause),
                    );
                    let strand = Strand {
                        infer,
                        ex_clause,
                        selected_subgoal: None,
                        last_pursued_time: TimeStamp::default(),
                    };
                    table_ref.push_strand(strand);
                }
            }
        }
    }

    /// Given a selected positive subgoal, applies the subgoal
    /// abstraction function to yield the canonical form that will be
    /// used to pick a table. Typically, this abstraction has no
    /// effect, and hence we are simply returning the canonical form
    /// of `subgoal`, but if the subgoal is getting too big, we may
    /// truncate the goal to ensure termination.
    ///
    /// This technique is described in the SA paper.
    fn abstract_positive_literal(
        &mut self,
        infer: &mut dyn InferenceTable<C>,
        subgoal: &C::GoalInEnvironment,
    ) -> (C::UCanonicalGoalInEnvironment, C::UniverseMap) {
        // Subgoal abstraction: Rather than looking up the table for
        // `selected_goal` directly, first apply the truncation
        // function. This may introduce fresh variables, making the
        // goal that we are looking up more general, and forcing us to
        // reuse an existing table. For example, if we had a selected
        // goal of
        //
        //     // Vec<Vec<Vec<Vec<i32>>>>: Sized
        //
        // we might now produce a truncated goal of
        //
        //     // Vec<Vec<?T>>: Sized
        //
        // Obviously, the answer we are looking for -- if it exists -- will be
        // found amongst the answers of this new, truncated goal.
        //
        // Subtle point: Note that the **selected goal** remains
        // unchanged and will be carried over into the "pending
        // clause" for the positive link on the new subgoal. This
        // means that if our new, truncated subgoal produces
        // irrelevant answers (e.g., `Vec<Vec<u32>>: Sized`), they
        // will fail to unify with our selected goal, producing no
        // resolvent.
        match infer.truncate_goal(subgoal) {
            None => infer.fully_canonicalize_goal(subgoal),
            Some(truncated_subgoal) => {
                debug!("truncated={:?}", truncated_subgoal);
                infer.fully_canonicalize_goal(&truncated_subgoal)
            }
        }
    }

    /// Given a selected negative subgoal, the subgoal is "inverted"
    /// (see `InferenceTable<C>::invert`) and then potentially truncated
    /// (see `abstract_positive_literal`). The result subgoal is
    /// canonicalized. In some cases, this may return `None` and hence
    /// fail to yield a useful result, for example if free existential
    /// variables appear in `subgoal` (in which case the execution is
    /// said to "flounder").
    fn abstract_negative_literal(
        &mut self,
        infer: &mut dyn InferenceTable<C>,
        subgoal: &C::GoalInEnvironment,
    ) -> Option<(C::UCanonicalGoalInEnvironment, C::UniverseMap)> {
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
        let inverted_subgoal = infer.invert_goal(subgoal)?;

        // DIVERGENCE
        //
        // If the negative subgoal has grown so large that we would have
        // to truncate it, we currently just abort the computation
        // entirely. This is not necessary -- the SA paper makes no
        // such distinction, for example, and applies truncation equally
        // for positive/negative literals. However, there are some complications
        // that arise that I do not wish to deal with right now.
        //
        // Let's work through an example to show you what I
        // mean. Imagine we have this (negative) selected literal;
        // hence `selected_subgoal` will just be the inner part:
        //
        //     // not { Vec<Vec<Vec<Vec<i32>>>>: Sized }
        //     //       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        //     //       `selected_goal`
        //
        // (In this case, the `inverted_subgoal` would be the same,
        // since there are no free universal variables.)
        //
        // If truncation **doesn't apply**, we would go and lookup the
        // table for the selected goal (`Vec<Vec<..>>: Sized`) and see
        // whether it has any answers. If it does, and they are
        // definite, then this negative literal is false. We don't
        // really care even how many answers there are and so forth
        // (if the goal is ground, as in this case, there can be at
        // most one definite answer, but if there are universals, then
        // the inverted goal would have variables; even so, a single
        // definite answer suffices to show that the `not { .. }` goal
        // is false).
        //
        // Truncation muddies the water, because the table may
        // generate answers that are not relevant to our original,
        // untruncated literal.  Suppose that we truncate the selected
        // goal to:
        //
        //     // Vec<Vec<T>: Sized
        //
        // Clearly this table will have some solutions that don't
        // apply to us.  e.g., `Vec<Vec<u32>>: Sized` is a solution to
        // this table, but that doesn't imply that `not {
        // Vec<Vec<Vec<..>>>: Sized }` is false.
        //
        // This can be made to work -- we carry along the original
        // selected goal when we establish links between tables, and
        // we could use that to screen the resulting answers. (There
        // are some further complications around the fact that
        // selected goal may contain universally quantified free
        // variables that have been inverted, as discussed in the
        // prior paragraph above.) I just didn't feel like dealing
        // with it yet.
        match infer.truncate_goal(&inverted_subgoal) {
            Some(_) => None,
            None => Some(infer.fully_canonicalize_goal(&inverted_subgoal)),
        }
    }

    /// Removes the subgoal at `subgoal_index` from the strand's
    /// subgoal list and adds it to the strand's floundered subgoal
    /// list.
    fn flounder_subgoal(&self, ex_clause: &mut ExClause<impl Context>, subgoal_index: usize) {
        info_heading!(
            "flounder_subgoal(current_time={:?}, subgoal={:?})",
            ex_clause.current_time,
            ex_clause.subgoals[subgoal_index],
        );
        let floundered_time = ex_clause.current_time;
        let floundered_literal = ex_clause.subgoals.remove(subgoal_index);
        ex_clause.floundered_subgoals.push(FlounderedSubgoal {
            floundered_literal,
            floundered_time,
        });
        debug!("flounder_subgoal: ex_clause={:#?}", ex_clause);
    }

    /// Used whenever we process an answer (whether new or cached) on
    /// a positive edge (the SLG POSITIVE RETURN operation). Truncates
    /// the resolvent (or factor) if it has grown too large.
    fn truncate_returned(&self, ex_clause: &mut ExClause<C>, infer: &mut dyn InferenceTable<C>) {
        // DIVERGENCE
        //
        // In the original RR paper, truncation is only applied
        // when the result of resolution is a new answer (i.e.,
        // `ex_clause.subgoals.is_empty()`).  I've chosen to be
        // more aggressive here, precisely because or our extended
        // semantics for unification. In particular, unification
        // can insert new goals, so I fear that positive feedback
        // loops could still run indefinitely in the original
        // formulation. I would like to revise our unification
        // mechanism to avoid that problem, in which case this could
        // be tightened up to be more like the original RR paper.
        //
        // Still, I *believe* this more aggressive approx. should
        // not interfere with any of the properties of the
        // original paper. In particular, applying truncation only
        // when the resolvent has no subgoals seems like it is
        // aimed at giving us more times to eliminate this
        // ambiguous answer.

        match infer.truncate_answer(&ex_clause.subst) {
            // No need to truncate
            None => {}

            // Resolvent got too large. Have to introduce approximation.
            Some(truncated_subst) => {
                mem::replace(
                    ex_clause,
                    ExClause {
                        subst: truncated_subst,
                        ambiguous: true,
                        constraints: vec![],
                        subgoals: vec![],
                        current_time: TimeStamp::default(),
                        floundered_subgoals: vec![],
                    },
                );
            }
        }
    }
}
