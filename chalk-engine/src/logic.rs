use crate::{DelayedLiteral, DelayedLiteralSet, DepthFirstNumber, ExClause, Literal, Minimums,
            TableIndex};
use crate::fallible::NoSolution;
use crate::context::{WithInstantiatedExClause, WithInstantiatedUCanonicalGoal, prelude::*};
use crate::forest::Forest;
use crate::hh::HhGoal;
use crate::stack::StackIndex;
use crate::strand::{CanonicalStrand, SelectedSubgoal, Strand};
use crate::table::{Answer, AnswerIndex};
use std::collections::HashSet;
use std::mem;

type RootSearchResult<T> = Result<T, RootSearchFail>;

/// The different ways that a *root* search (which potentially pursues
/// many strands) can fail. A root search is one that begins with an
/// empty stack.
///
/// (This is different from `RecursiveSearchFail` because nothing can
/// be on the stack, so cycles are ruled out.)
#[derive(Debug)]
pub(super) enum RootSearchFail {
    /// The subgoal we were trying to solve cannot succeed.
    NoMoreSolutions,

    /// We did not find a solution, but we still have things to try.
    /// Repeat the request, and we'll give one of those a spin.
    ///
    /// (In a purely depth-first-based solver, like Prolog, this
    /// doesn't appear.)
    QuantumExceeded,
}

type RecursiveSearchResult<T> = Result<T, RecursiveSearchFail>;

/// The different ways that a recursive search (which potentially
/// pursues many strands) can fail -- a "recursive" search is one that
/// did not start with an empty stack.
#[derive(Debug)]
enum RecursiveSearchFail {
    /// The subgoal we were trying to solve cannot succeed.
    NoMoreSolutions,

    /// **All** avenues to solve the subgoal we were trying solve
    /// encountered a cyclic dependency on something higher up in the
    /// stack. The `Minimums` encodes how high up (and whether
    /// positive or negative).
    Cycle(Minimums),

    /// We did not find a solution, but we still have things to try.
    /// Repeat the request, and we'll give one of those a spin.
    ///
    /// (In a purely depth-first-based solver, like Prolog, this
    /// doesn't appear.)
    QuantumExceeded,
}

type StrandResult<C, T> = Result<T, StrandFail<C>>;

/// Possible failures from pursuing a particular strand.
#[derive(Debug)]
pub(super) enum StrandFail<C: Context> {
    /// The strand has no solution.
    NoSolution,

    /// We did not yet figure out a solution; the strand will have
    /// been rescheduled for later.
    QuantumExceeded,

    /// The strand hit a cyclic dependency. In this case,
    /// we return the strand, as well as a `Minimums` struct.
    Cycle(CanonicalStrand<C>, Minimums),
}

#[derive(Debug)]
enum EnsureSuccess {
    AnswerAvailable,
    Coinductive,
}

impl<C: Context> Forest<C> {
    /// Ensures that answer with the given index is available from the
    /// given table. This may require activating a strand. Returns
    /// `Ok(())` if the answer is available and otherwise a
    /// `RootSearchFail` result.
    pub(super) fn ensure_root_answer(
        &mut self,
        table: TableIndex,
        answer: AnswerIndex,
    ) -> RootSearchResult<()> {
        assert!(self.stack.is_empty());

        match self.ensure_answer_recursively(table, answer) {
            Ok(EnsureSuccess::AnswerAvailable) => Ok(()),
            Err(RecursiveSearchFail::NoMoreSolutions) => Err(RootSearchFail::NoMoreSolutions),
            Err(RecursiveSearchFail::QuantumExceeded) => Err(RootSearchFail::QuantumExceeded),

            // Things involving cycles should be impossible since our
            // stack was empty on entry:
            Ok(EnsureSuccess::Coinductive) | Err(RecursiveSearchFail::Cycle(..)) => {
                panic!("ensure_root_answer: nothing on the stack but cyclic result")
            }
        }
    }

    pub(super) fn any_future_answer(
        &mut self,
        table: TableIndex,
        answer: AnswerIndex,
        mut test: impl FnMut(&C::InferenceNormalizedSubst) -> bool,
    ) -> bool {
        if let Some(answer) = self.tables[table].answer(answer) {
            info!("answer cached = {:?}", answer);
            return test(C::inference_normalized_subst_from_subst(&answer.subst));
        }

        self.tables[table].strands_mut().any(|strand| {
            test(C::inference_normalized_subst_from_ex_clause(&strand.canonical_ex_clause))
        })
    }

    /// Ensures that answer with the given index is available from the
    /// given table. Returns `Ok` if there is an answer:
    ///
    /// - `EnsureSuccess::AnswerAvailable` means that the answer is
    ///   cached in the table (and can be fetched with e.g. `self.answer()`).
    /// - `EnsureSuccess::Coinductive` means that this was a cyclic
    ///   request of a coinductive goal and is thus considered true;
    ///   in this case, the answer is not cached in the table (it is
    ///   only true in this cyclic context).
    ///
    /// This function first attempts to fetch answer that is cached in
    /// the table. If none is found, then we will if the table is on
    /// the stack; if so, that constitutes a cycle (producing a new
    /// result for the table X required producing a new result for the
    /// table X), and we return a suitable result. Otherwise, we can
    /// push the table onto the stack and select the next available
    /// strand -- if none are available, then no more answers are
    /// possible.
    fn ensure_answer_recursively(
        &mut self,
        table: TableIndex,
        answer: AnswerIndex,
    ) -> RecursiveSearchResult<EnsureSuccess> {
        info_heading!(
            "ensure_answer_recursively(table={:?}, answer={:?})",
            table,
            answer
        );
        info!("table goal = {:#?}", self.tables[table].table_goal);

        // First, check for a tabled answer.
        if self.tables[table].answer(answer).is_some() {
            info!("answer cached = {:?}", self.tables[table].answer(answer));
            return Ok(EnsureSuccess::AnswerAvailable);
        }

        // If no tabled answer is present, we ought to be requesting
        // the next available index.
        assert_eq!(self.tables[table].next_answer_index(), answer);

        // Next, check if the table is already active. If so, then we
        // have a recursive attempt.
        if let Some(depth) = self.stack.is_active(table) {
            info!("ensure_answer: cycle detected at depth {:?}", depth);

            if self.top_of_stack_is_coinductive_from(depth) {
                return Ok(EnsureSuccess::Coinductive);
            }

            return Err(RecursiveSearchFail::Cycle(Minimums {
                positive: self.stack[depth].dfn,
                negative: DepthFirstNumber::MAX,
            }));
        }

        let dfn = self.next_dfn();
        let depth = self.stack.push(table, dfn);
        let result = self.pursue_next_strand(depth);
        self.stack.pop(table, depth);
        info!("ensure_answer: result = {:?}", result);
        result.map(|()| EnsureSuccess::AnswerAvailable)
    }

    crate fn answer(&self, table: TableIndex, answer: AnswerIndex) -> &Answer<C> {
        self.tables[table].answer(answer).unwrap()
    }

    /// Selects the next eligible strand from the table at depth
    /// `depth` and pursues it. If that strand encounters a cycle,
    /// then this function will loop and keep trying strands until it
    /// reaches one that did not encounter a cycle; that result is
    /// propagated.  If all strands return a cycle, then the entire
    /// subtree is "completed" by invoking `cycle`.
    fn pursue_next_strand(&mut self, depth: StackIndex) -> RecursiveSearchResult<()> {
        // This is a bit complicated because this is where we handle cycles.
        let table = self.stack[depth].table;

        // Strands that encountered a cyclic error.
        let mut cyclic_strands = vec![];

        // The minimum of all cyclic strands.
        let mut cyclic_minimums = Minimums::MAX;

        loop {
            match self.tables[table].pop_next_strand() {
                Some(canonical_strand) => {
                    let num_universes = self.tables[table].table_goal.num_universes();
                    let result = Self::with_instantiated_strand(
                        self.context.clone(),
                        num_universes,
                        &canonical_strand,
                        PursueStrand {
                            forest: self,
                            depth,
                        },
                    );
                    match result {
                        Ok(answer) => {
                            // Now that we produced an answer, these
                            // cyclic strands need to be retried.
                            self.tables[table].extend_strands(cyclic_strands);
                            return Ok(answer);
                        }

                        Err(StrandFail::NoSolution) | Err(StrandFail::QuantumExceeded) => {
                            // This strand did not produce an answer,
                            // but either it (or some other, pending
                            // strands) may do so in the
                            // future. Enqueue the cyclic strands to
                            // be retried after that point.
                            self.tables[table].extend_strands(cyclic_strands);
                            return Err(RecursiveSearchFail::QuantumExceeded);
                        }

                        Err(StrandFail::Cycle(canonical_strand, strand_minimums)) => {
                            // This strand encountered a cycle. Stash
                            // it for later and try the next one until
                            // we know that *all* available strands
                            // are hitting a cycle.
                            cyclic_strands.push(canonical_strand);
                            cyclic_minimums.take_minimums(&strand_minimums);
                        }
                    }
                }

                None => {
                    // No more strands left to try! That means either we started
                    // with no strands, or all available strands encountered a cycle.

                    if cyclic_strands.is_empty() {
                        // We started with no strands!
                        return Err(RecursiveSearchFail::NoMoreSolutions);
                    } else {
                        let c = mem::replace(&mut cyclic_strands, vec![]);
                        if let Some(err) = self.cycle(depth, c, cyclic_minimums) {
                            return Err(err);
                        }
                    }
                }
            }
        }
    }

    fn with_instantiated_strand<R>(
        context: C,
        num_universes: usize,
        canonical_strand: &CanonicalStrand<C>,
        op: impl WithInstantiatedStrand<C, Output = R>,
    ) -> R {
        let CanonicalStrand {
            canonical_ex_clause,
            selected_subgoal,
        } = canonical_strand;
        return context.instantiate_ex_clause(
            num_universes,
            &canonical_ex_clause,
            With {
                op,
                selected_subgoal: selected_subgoal.clone(),
            },
        );

        struct With<C: Context, OP: WithInstantiatedStrand<C>> {
            op: OP,
            selected_subgoal: Option<SelectedSubgoal<C>>,
        }

        impl<C: Context, OP: WithInstantiatedStrand<C>> WithInstantiatedExClause<C> for With<C, OP> {
            type Output = OP::Output;

            fn with<I: InferenceContext<C>>(
                self,
                infer: &mut dyn InferenceTable<C, I>,
                ex_clause: ExClause<C, I>,
            ) -> OP::Output {
                self.op.with(Strand {
                    infer,
                    ex_clause,
                    selected_subgoal: self.selected_subgoal.clone(),
                })
            }
        }
    }

    fn canonicalize_strand(strand: Strand<'_, C, impl InferenceContext<C>>) -> CanonicalStrand<C> {
        let Strand {
            infer,
            ex_clause,
            selected_subgoal,
        } = strand;
        Self::canonicalize_strand_from(&mut *infer, &ex_clause, selected_subgoal)
    }

    fn canonicalize_strand_from<I: InferenceContext<C>>(
        infer: &mut dyn InferenceTable<C, I>,
        ex_clause: &ExClause<C, I>,
        selected_subgoal: Option<SelectedSubgoal<C>>,
    ) -> CanonicalStrand<C> {
        let canonical_ex_clause = infer.canonicalize_ex_clause(&ex_clause);
        CanonicalStrand {
            canonical_ex_clause,
            selected_subgoal,
        }
    }

    /// Invoked when all available strands for a table have
    /// encountered a cycle. In this case, the vector `strands` are
    /// the set of strands that encountered cycles, and `minimums` is
    /// the minimum stack depths that they were dependent on.
    ///
    /// Returns `None` if we have resolved the cycle and should try to
    /// pick a strand again. Returns `Some(_)` if the cycle indicates
    /// an error that we can propagate higher up.
    fn cycle(
        &mut self,
        depth: StackIndex,
        strands: Vec<CanonicalStrand<C>>,
        minimums: Minimums,
    ) -> Option<RecursiveSearchFail> {
        let table = self.stack[depth].table;
        assert!(self.tables[table].pop_next_strand().is_none());

        let dfn = self.stack[depth].dfn;
        if minimums.positive == dfn && minimums.negative == DepthFirstNumber::MAX {
            // If all the things that we recursively depend on have
            // positive dependencies on things below us in the stack,
            // then no more answers are forthcoming. We can clear all
            // the strands for those things recursively.
            self.clear_strands_after_cycle(table, strands);
            Some(RecursiveSearchFail::NoMoreSolutions)
        } else if minimums.positive >= dfn && minimums.negative >= dfn {
            let mut visited = HashSet::default();
            visited.insert(table);
            self.tables[table].extend_strands(strands);
            self.delay_strands_after_cycle(table, &mut visited);
            None
        } else {
            self.tables[table].extend_strands(strands);
            Some(RecursiveSearchFail::Cycle(minimums))
        }
    }

    /// Invoked after we have determined that every strand in `table`
    /// encounters a cycle; `strands` is the set of strands (which
    /// have been moved out of the table). This method then
    /// recursively clears the active strands from the tables
    /// referenced in `strands`, since all of them must encounter
    /// cycles too.
    fn clear_strands_after_cycle(
        &mut self,
        table: TableIndex,
        strands: impl IntoIterator<Item = CanonicalStrand<C>>,
    ) {
        assert!(self.tables[table].pop_next_strand().is_none());
        for strand in strands {
            let CanonicalStrand {
                canonical_ex_clause,
                selected_subgoal,
            } = strand;
            let selected_subgoal = selected_subgoal.unwrap_or_else(|| {
                panic!(
                    "clear_strands_after_cycle invoked on strand in table {:?} \
                     without a selected subgoal: {:?}",
                    table, canonical_ex_clause,
                )
            });

            let strand_table = selected_subgoal.subgoal_table;
            let strands = self.tables[strand_table].take_strands();
            self.clear_strands_after_cycle(strand_table, strands);
        }
    }

    /// Invoked after we have determined that every strand in `table`
    /// encounters a cycle, and that some of those cycles involve
    /// negative edges. In that case, walks all negative edges and
    /// converts them to delayed literals.
    fn delay_strands_after_cycle(&mut self, table: TableIndex, visited: &mut HashSet<TableIndex>) {
        let mut tables = vec![];

        let num_universes = self.tables[table].table_goal.num_universes();
        for canonical_strand in self.tables[table].strands_mut() {
            // FIXME if CanonicalExClause were not held abstract, we
            // could do this in place like we used to (and
            // `instantiate_strand` could take ownership), since we
            // don't really need to instantiate here to do this
            // operation.
            let (delayed_strand, subgoal_table) = Self::with_instantiated_strand(
                self.context.clone(),
                num_universes,
                canonical_strand,
                DelayStrandAfterCycle { table },
            );

            *canonical_strand = delayed_strand;

            if visited.insert(subgoal_table) {
                tables.push(subgoal_table);
            }
        }

        for table in tables {
            self.delay_strands_after_cycle(table, visited);
        }
    }

    fn delay_strand_after_cycle(
        table: TableIndex,
        mut strand: Strand<'_, C, impl InferenceContext<C>>,
    ) -> (CanonicalStrand<C>, TableIndex) {
        let (subgoal_index, subgoal_table) = match &strand.selected_subgoal {
            Some(selected_subgoal) => (
                selected_subgoal.subgoal_index,
                selected_subgoal.subgoal_table,
            ),
            None => {
                panic!(
                    "delay_strands_after_cycle invoked on strand in table {:?} \
                     without a selected subgoal: {:?}",
                    table, strand,
                );
            }
        };

        // Delay negative literals.
        if let Literal::Negative(_) = strand.ex_clause.subgoals[subgoal_index] {
            strand.ex_clause.subgoals.remove(subgoal_index);
            strand
                .ex_clause
                .delayed_literals
                .push(DelayedLiteral::Negative(subgoal_table));
            strand.selected_subgoal = None;
        }

        (Self::canonicalize_strand(strand), subgoal_table)
    }

    /// Pursues `strand` to see if it leads us to a new answer, either
    /// by selecting a new subgoal or by checking to see if the
    /// selected subgoal has an answer. `strand` is associated with
    /// the table on the stack at the given `depth`.
    fn pursue_strand(
        &mut self,
        depth: StackIndex,
        mut strand: Strand<'_, C, impl InferenceContext<C>>,
    ) -> StrandResult<C, ()> {
        info_heading!(
            "pursue_strand(table={:?}, depth={:?}, ex_clause={:#?}, selected_subgoal={:?})",
            self.stack[depth].table,
            depth,
            strand.infer.debug_ex_clause(&strand.ex_clause),
            strand.selected_subgoal,
        );

        // If no subgoal has yet been selected, select one.
        while strand.selected_subgoal.is_none() {
            if strand.ex_clause.subgoals.len() == 0 {
                return self.pursue_answer(depth, strand);
            }

            // For now, we always pick the last subgoal in the
            // list.
            //
            // FIXME(rust-lang-nursery/chalk#80) -- we should be more
            // selective. For example, we don't want to pick a
            // negative literal that will flounder, and we don't want
            // to pick things like `?T: Sized` if we can help it.
            let subgoal_index = strand.ex_clause.subgoals.len() - 1;

            // Get or create table for this subgoal.
            match self.get_or_create_table_for_subgoal(
                &mut *strand.infer,
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
                    // then the execution has "floundered" (cannot yield
                    // a complete result). We choose to handle this by
                    // removing the subgoal and inserting a
                    // `CannotProve` result. This can only happen with
                    // ill-formed negative literals or with overflow.
                    strand.ex_clause.subgoals.remove(subgoal_index);
                    strand
                        .ex_clause
                        .delayed_literals
                        .push(DelayedLiteral::CannotProve(()));
                }
            }
        }

        // Find the selected subgoal and ask it for the next answer.
        let selected_subgoal = strand.selected_subgoal.clone().unwrap();
        match strand.ex_clause.subgoals[selected_subgoal.subgoal_index] {
            Literal::Positive(_) => self.pursue_positive_subgoal(depth, strand, &selected_subgoal),
            Literal::Negative(_) => self.pursue_negative_subgoal(depth, strand, &selected_subgoal),
        }
    }

    /// Invoked when a strand represents an **answer**. This means
    /// that the strand has no subgoals left. There are two possibilities:
    ///
    /// - the strand may represent an answer we have already found; in
    ///   that case, we can return `StrandFail::NoSolution`, as this
    ///   strand led nowhere of interest.
    /// - the strand may represent a new answer, in which case it is
    ///   added to the table and `Ok` is returned.
    fn pursue_answer(
        &mut self,
        depth: StackIndex,
        strand: Strand<'_, C, impl InferenceContext<C>>,
    ) -> StrandResult<C, ()> {
        let table = self.stack[depth].table;
        let Strand {
            infer,
            ex_clause:
                ExClause {
                    subst,
                    constraints,
                    delayed_literals,
                    subgoals,
                },
            selected_subgoal: _,
        } = strand;
        assert!(subgoals.is_empty());

        let answer_subst = infer.canonicalize_constrained_subst(subst, constraints);
        debug!("answer: table={:?}, answer_subst={:?}", table, answer_subst);

        let delayed_literals = {
            let mut delayed_literals: Vec<_> = delayed_literals.into_iter().collect();
            delayed_literals.sort();
            delayed_literals.dedup();
            DelayedLiteralSet { delayed_literals }
        };
        debug!("answer: delayed_literals={:?}", delayed_literals);

        let answer = Answer {
            subst: answer_subst,
            delayed_literals,
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
            answer.delayed_literals.is_empty()
                && self.tables[table]
                    .table_goal
                    .is_trivial_substitution(&answer.subst)
                && C::empty_constraints(&answer.subst)
        };

        if self.tables[table].push_answer(answer) {
            if is_trivial_answer {
                self.tables[table].take_strands();
            }

            Ok(())
        } else {
            info!("answer: not a new answer, returning StrandFail::NoSolution");
            Err(StrandFail::NoSolution)
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
    fn get_or_create_table_for_subgoal<I: InferenceContext<C>>(
        &mut self,
        infer: &mut dyn InferenceTable<C, I>,
        subgoal: &Literal<C, I>,
    ) -> Option<(TableIndex, C::UniverseMap)> {
        debug_heading!("get_or_create_table_for_subgoal(subgoal={:?})", subgoal);

        // Subgoal abstraction:
        let canonical_subgoal = match subgoal {
            Literal::Positive(subgoal) => self.abstract_positive_literal(infer, subgoal),
            Literal::Negative(subgoal) => self.abstract_negative_literal(infer, subgoal)?,
        };

        debug!("canonical_subgoal={:?}", canonical_subgoal);

        let (ucanonical_subgoal, universe_map) = infer.u_canonicalize_goal(&canonical_subgoal);

        let table = self.get_or_create_table_for_ucanonical_goal(ucanonical_subgoal);

        Some((table, universe_map))
    }

    /// Given a u-canonical goal, searches for an existing table. If
    /// one is found, it is returned, but otherwise a new table is
    /// created (and populated with its initial set of strands).
    ///
    /// In terms of the NFTD paper, creating a new table corresponds
    /// to the *New Subgoal* step as well as the *Program Clause
    /// Resolution* steps.
    crate fn get_or_create_table_for_ucanonical_goal(
        &mut self,
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
        let coinductive_goal = self.context.is_coinductive(&goal);
        let table = self.tables.insert(goal, coinductive_goal);
        self.push_initial_strands(table);
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
    fn push_initial_strands(&mut self, table: TableIndex) {
        // Instantiate the table goal with fresh inference variables.
        let table_goal = self.tables[table].table_goal.clone();
        self.context.clone().instantiate_ucanonical_goal(
            &table_goal,
            PushInitialStrandsInstantiated { table, this: self },
        );

        struct PushInitialStrandsInstantiated<'a, C: Context + 'a> {
            table: TableIndex,
            this: &'a mut Forest<C>,
        }

        impl<C: Context> WithInstantiatedUCanonicalGoal<C> for PushInitialStrandsInstantiated<'a, C> {
            type Output = ();

            fn with<I: InferenceContext<C>>(
                self,
                infer: &mut dyn InferenceTable<C, I>,
                subst: I::Substitution,
                environment: I::Environment,
                goal: I::Goal,
            ) {
                let PushInitialStrandsInstantiated { table, this } = self;
                this.push_initial_strands_instantiated(table, infer, subst, environment, goal);
            }
        }
    }

    fn push_initial_strands_instantiated<I: InferenceContext<C>>(
        &mut self,
        table: TableIndex,
        infer: &mut dyn InferenceTable<C, I>,
        subst: I::Substitution,
        environment: I::Environment,
        goal: I::Goal,
    ) {
        let table_ref = &mut self.tables[table];
        match I::into_hh_goal(goal) {
            HhGoal::DomainGoal(domain_goal) => {
                let clauses = infer.program_clauses(&environment, &domain_goal);
                for clause in clauses {
                    debug!("program clause = {:#?}", clause);
                    if let Ok(resolvent) =
                        infer.resolvent_clause(&environment, &domain_goal, &subst, &clause)
                    {
                        info!("pushing initial strand with ex-clause: {:#?}", &resolvent,);
                        table_ref.push_strand(CanonicalStrand {
                            canonical_ex_clause: resolvent,
                            selected_subgoal: None,
                        });
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
                    Self::simplify_hh_goal(&mut *infer, subst, &environment, hh_goal)
                {
                    info!(
                        "pushing initial strand with ex-clause: {:#?}",
                        infer.debug_ex_clause(&ex_clause),
                    );
                    table_ref.push_strand(Self::canonicalize_strand(Strand {
                        infer,
                        ex_clause,
                        selected_subgoal: None,
                    }));
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
    fn abstract_positive_literal<I: InferenceContext<C>>(
        &mut self,
        infer: &mut dyn InferenceTable<C, I>,
        subgoal: &I::GoalInEnvironment,
    ) -> C::CanonicalGoalInEnvironment {
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
            None => infer.canonicalize_goal(subgoal),
            Some(truncated_subgoal) => {
                debug!("truncated={:?}", truncated_subgoal);
                infer.canonicalize_goal(&truncated_subgoal)
            }
        }
    }

    /// Given a selected negative subgoal, the subgoal is "inverted"
    /// (see `InferenceTable<C, I>::invert`) and then potentially truncated
    /// (see `abstract_positive_literal`). The result subgoal is
    /// canonicalized. In some cases, this may return `None` and hence
    /// fail to yield a useful result, for example if free existential
    /// variables appear in `subgoal` (in which case the execution is
    /// said to "flounder").
    fn abstract_negative_literal<I: InferenceContext<C>>(
        &mut self,
        infer: &mut dyn InferenceTable<C, I>,
        subgoal: &I::GoalInEnvironment,
    ) -> Option<C::CanonicalGoalInEnvironment> {
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
        // untracted literal.  Suppose that we truncate the selected
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
            None => Some(infer.canonicalize_goal(&inverted_subgoal)),
        }
    }

    /// Invoked when we have selected a positive literal, created its
    /// table, and selected a particular answer index N we are looking
    /// for. Searches for that answer. If we find one, we can do two things:
    ///
    /// - create a new strand with the same selected subgoal, but searching for the
    ///   answer with index N+1
    /// - use the answer to resolve our selected literal and select the next subgoal
    ///   in this strand to pursue
    ///
    /// When an answer is found, that corresponds to *Positive Return*
    /// from the NFTD paper.
    fn pursue_positive_subgoal(
        &mut self,
        depth: StackIndex,
        mut strand: Strand<'_, C, impl InferenceContext<C>>,
        selected_subgoal: &SelectedSubgoal<C>,
    ) -> StrandResult<C, ()> {
        let table = self.stack[depth].table;
        let SelectedSubgoal {
            subgoal_index,
            subgoal_table,
            answer_index,
            ref universe_map,
        } = *selected_subgoal;

        match self.ensure_answer_recursively(subgoal_table, answer_index) {
            Ok(EnsureSuccess::AnswerAvailable) => {
                // The given answer is available; we'll process it below.
            }
            Ok(EnsureSuccess::Coinductive) => {
                // This is a co-inductive cycle. That is, this table
                // appears somewhere higher on the stack, and has now
                // recursively requested an answer for itself. That
                // means that our subgoal is unconditionally true, so
                // we can drop it and pursue the next thing.
                assert!(
                    self.tables[table].coinductive_goal
                        && self.tables[subgoal_table].coinductive_goal
                );
                let Strand {
                    infer,
                    mut ex_clause,
                    selected_subgoal: _,
                } = strand;
                ex_clause.subgoals.remove(subgoal_index);
                return self.pursue_strand_recursively(
                    depth,
                    Strand {
                        infer,
                        ex_clause,
                        selected_subgoal: None,
                    },
                );
            }
            Err(RecursiveSearchFail::NoMoreSolutions) => {
                info!("pursue_positive_subgoal: no more solutions");
                return Err(StrandFail::NoSolution);
            }
            Err(RecursiveSearchFail::QuantumExceeded) => {
                // We'll have to revisit this strand later
                info!("pursue_positive_subgoal: quantum exceeded");
                self.tables[table].push_strand(Self::canonicalize_strand(strand));
                return Err(StrandFail::QuantumExceeded);
            }
            Err(RecursiveSearchFail::Cycle(minimums)) => {
                info!(
                    "pursue_positive_subgoal: cycle with minimums {:?}",
                    minimums
                );
                let canonical_strand = Self::canonicalize_strand(strand);
                return Err(StrandFail::Cycle(canonical_strand, minimums));
            }
        }

        // Whichever way this particular answer turns out, there may
        // yet be *more* answers. Enqueue that alternative for later.
        self.push_strand_pursuing_next_answer(depth, &mut strand, selected_subgoal);

        // OK, let's follow *this* answer and see where it leads.
        let Strand {
            infer,
            mut ex_clause,
            selected_subgoal: _,
        } = strand;
        let subgoal = match ex_clause.subgoals.remove(subgoal_index) {
            Literal::Positive(g) => g,
            Literal::Negative(g) => panic!(
                "pursue_positive_subgoal invoked with negative selected literal: {:?}",
                g
            ),
        };

        let table_goal = &universe_map
            .map_goal_from_canonical(&self.tables[subgoal_table].table_goal.canonical());
        let answer_subst =
            &universe_map.map_subst_from_canonical(&self.answer(subgoal_table, answer_index).subst);
        match infer.apply_answer_subst(ex_clause, &subgoal, table_goal, answer_subst) {
            Ok(mut ex_clause) => {
                // If the answer had delayed literals, we have to
                // ensure that `ex_clause` is also delayed. This is
                // the SLG FACTOR operation, though NFTD just makes it
                // part of computing the SLG resolvent.
                {
                    let answer = self.answer(subgoal_table, answer_index);
                    if !answer.delayed_literals.is_empty() {
                        ex_clause.delayed_literals.push(DelayedLiteral::Positive(
                            subgoal_table,
                            answer.subst.clone(),
                        ));
                    }
                }

                // Apply answer abstraction.
                let ex_clause = self.truncate_returned(ex_clause, &mut *infer);

                self.pursue_strand_recursively(
                    depth,
                    Strand {
                        infer,
                        ex_clause,
                        selected_subgoal: None,
                    },
                )
            }

            // This answer led nowhere. Give up for now, but of course
            // there may still be other strands to pursue, so return
            // `QuantumExceeded`.
            Err(NoSolution) => {
                info!("pursue_positive_subgoal: answer not unifiable -> NoSolution");
                Err(StrandFail::NoSolution)
            }
        }
    }

    /// Used whenever we process an answer (whether new or cached) on
    /// a positive edge (the SLG POSITIVE RETURN operation). Truncates
    /// the resolvent (or factor) if it has grown too large.
    fn truncate_returned<I: InferenceContext<C>>(
        &self,
        ex_clause: ExClause<C, I>,
        infer: &mut dyn InferenceTable<C, I>,
    ) -> ExClause<C, I> {
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
            // No need to truncate? Just propagate the resolvent back.
            None => ex_clause,

            // Resolvent got too large. Have to introduce approximation.
            Some(truncated_subst) => {
                // DIVERGENCE
                //
                // In RR, `self.delayed_literals` would be
                // preserved. I have chosen to drop them. Keeping
                // them does allow for the possibility of
                // eliminating this answer if any of them turn out
                // to be satisfiable. However, it also introduces
                // an annoying edge case I didn't want to think
                // about -- one which, interestingly, the paper
                // did not discuss, which may indicate it is
                // impossible for some subtle reason. In
                // particular, a truncated delayed literal has a
                // sort of inverse semantics. i.e. if we convert
                // `Foo :- ~Bar(Rc<Rc<u32>>) |` to `Foo :-
                // ~Bar(Rc<X>), Unknown |`, then this could be
                // invalidated by an instance of `Bar(Rc<i32>)`,
                // which is irrelevant to the original
                // clause. (There is an additional annoyance,
                // which is that we may not have tried to solve
                // `Bar(Rc<X>)` at all.)

                ExClause {
                    subst: truncated_subst,
                    delayed_literals: vec![DelayedLiteral::CannotProve(())],
                    constraints: vec![],
                    subgoals: vec![],
                }
            }
        }
    }

    // We can recursive arbitrarily deep while pursuing a strand, so
    // check in case we have to grow the stack.
    fn pursue_strand_recursively(
        &mut self,
        depth: StackIndex,
        strand: Strand<'_, C, impl InferenceContext<C>>,
    ) -> StrandResult<C, ()> {
        ::crate::maybe_grow_stack(|| self.pursue_strand(depth, strand))
    }

    /// Invoked when we have found a successful answer to the given
    /// table. Queues up a strand to look for the *next* answer from
    /// that table.
    fn push_strand_pursuing_next_answer(
        &mut self,
        depth: StackIndex,
        strand: &mut Strand<'_, C, impl InferenceContext<C>>,
        selected_subgoal: &SelectedSubgoal<C>,
    ) {
        let table = self.stack[depth].table;
        let mut selected_subgoal = selected_subgoal.clone();
        selected_subgoal.answer_index.increment();
        self.tables[table].push_strand(Self::canonicalize_strand_from(
            &mut *strand.infer,
            &strand.ex_clause,
            Some(selected_subgoal),
        ));
    }

    fn pursue_negative_subgoal(
        &mut self,
        depth: StackIndex,
        strand: Strand<'_, C, impl InferenceContext<C>>,
        selected_subgoal: &SelectedSubgoal<C>,
    ) -> StrandResult<C, ()> {
        let table = self.stack[depth].table;
        let SelectedSubgoal {
            subgoal_index: _,
            subgoal_table,
            answer_index,
            universe_map: _,
        } = *selected_subgoal;

        // In the match below, we will either (a) return early with an
        // error or some kind or (b) continue on to pursue this strand
        // further. We continue onward in the case where we either
        // proved that `answer_index` does not exist (in which case
        // the negative literal is true) or if we found a delayed
        // literal (in which case the negative literal *may* be true).
        // Before exiting the match, then, we set `delayed_literal` to
        // either `Some` or `None` depending.
        let delayed_literal: Option<DelayedLiteral<C>>;
        match self.ensure_answer_recursively(subgoal_table, answer_index) {
            Ok(EnsureSuccess::AnswerAvailable) => {
                if self.answer(subgoal_table, answer_index).is_unconditional() {
                    // We want to disproval the subgoal, but we
                    // have an unconditional answer for the subgoal,
                    // therefore we have failed to disprove it.
                    info!("pursue_negative_subgoal: found unconditional answer to neg literal -> NoSolution");
                    return Err(StrandFail::NoSolution);
                }

                // Got back a conditional answer. We neither succeed
                // nor fail yet; so what we do is to delay the
                // selected literal and keep going.
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
                delayed_literal = Some(DelayedLiteral::Negative(subgoal_table));
            }

            Ok(EnsureSuccess::Coinductive) => {
                // This is a co-inductive cycle. That is, this table
                // appears somewhere higher on the stack, and has now
                // recursively requested an answer for itself. That
                // means that our subgoal is unconditionally true, so
                // our negative goal fails.
                info!("pursue_negative_subgoal: found coinductive answer to neg literal -> NoSolution");
                return Err(StrandFail::NoSolution);
            }

            Err(RecursiveSearchFail::Cycle(minimums)) => {
                // We depend on `not(subgoal)`. For us to continue,
                // `subgoal` must be completely evaluated. Therefore,
                // we depend (negatively) on the minimum link of
                // `subgoal` as a whole -- it doesn't matter whether
                // it's pos or neg.
                let min = minimums.minimum_of_pos_and_neg();
                info!(
                    "pursue_negative_subgoal: found neg cycle at depth {:?}",
                    min
                );
                let canonical_strand = Self::canonicalize_strand(strand);
                return Err(StrandFail::Cycle(
                    canonical_strand,
                    Minimums {
                        positive: self.stack[depth].dfn,
                        negative: min,
                    },
                ));
            }

            Err(RecursiveSearchFail::NoMoreSolutions) => {
                // This answer does not exist. Huzzah, happy days are
                // here again! =) We can just remove this subgoal and continue
                // with no need for a delayed literal.
                delayed_literal = None;
            }

            // Learned nothing yet. Have to try again some other time.
            Err(RecursiveSearchFail::QuantumExceeded) => {
                info!("pursue_negative_subgoal: quantum exceeded");
                self.tables[table].push_strand(Self::canonicalize_strand(strand));
                return Err(StrandFail::QuantumExceeded);
            }
        }

        // We have found that there is at least a *chance* that
        // `answer_index` of the subgoal is a failure, so let's keep
        // going. We can just remove the subgoal from the list without
        // any need to unify things, because the subgoal must be
        // ground (i). We may need to add a delayed literal, though (ii).
        let Strand {
            infer,
            mut ex_clause,
            selected_subgoal: _,
        } = strand;
        ex_clause.subgoals.remove(selected_subgoal.subgoal_index); // (i)
        ex_clause.delayed_literals.extend(delayed_literal); // (ii)
        self.pursue_strand_recursively(
            depth,
            Strand {
                infer,
                ex_clause,
                selected_subgoal: None,
            },
        )
    }
}

trait WithInstantiatedStrand<C: Context> {
    type Output;

    fn with(self, strand: Strand<'_, C, impl InferenceContext<C>>) -> Self::Output;
}

struct PursueStrand<'a, C: Context + 'a> {
    forest: &'a mut Forest<C>,
    depth: StackIndex,
}

impl<C: Context> WithInstantiatedStrand<C> for PursueStrand<'a, C> {
    type Output = StrandResult<C, ()>;

    fn with(self, strand: Strand<'_, C, impl InferenceContext<C>>) -> Self::Output {
        self.forest.pursue_strand(self.depth, strand)
    }
}

struct DelayStrandAfterCycle {
    table: TableIndex,
}

impl<C: Context> WithInstantiatedStrand<C> for DelayStrandAfterCycle {
    type Output = (CanonicalStrand<C>, TableIndex);

    fn with(self, strand: Strand<'_, C, impl InferenceContext<C>>) -> Self::Output {
        <Forest<C>>::delay_strand_after_cycle(self.table, strand)
    }
}
