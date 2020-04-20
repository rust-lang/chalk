use super::*;

mod fulfill;
mod search_graph;
mod stack;

use self::fulfill::Fulfill;
use self::search_graph::{DepthFirstNumber, SearchGraph};
use self::stack::{Stack, StackDepth};
use chalk_engine::{
    context::Floundered,
    fallible::{Fallible, NoSolution},
};
use clauses::program_clauses_for_goal;
use rustc_hash::FxHashMap;

type UCanonicalGoal<I> = UCanonical<InEnvironment<Goal<I>>>;

pub(crate) struct RecursiveContext<I: Interner> {
    stack: Stack,
    search_graph: SearchGraph<I>,
    cache: FxHashMap<UCanonicalGoal<I>, Fallible<Solution<I>>>,

    caching_enabled: bool,
}

/// A Solver is the basic context in which you can propose goals for a given
/// program. **All questions posed to the solver are in canonical, closed form,
/// so that each question is answered with effectively a "clean slate"**. This
/// allows for better caching, and simplifies management of the inference
/// context.
pub(crate) struct Solver<'me, I: Interner> {
    program: &'me dyn RustIrDatabase<I>,
    context: &'me mut RecursiveContext<I>,
}

/// The `minimums` struct is used while solving to track whether we encountered
/// any cycles in the process.
#[derive(Copy, Clone, Debug)]
struct Minimums {
    positive: DepthFirstNumber,
}

/// An extension trait for merging `Result`s
trait MergeWith<T> {
    fn merge_with<F>(self, other: Self, f: F) -> Self
    where
        F: FnOnce(T, T) -> T;
}

impl<T> MergeWith<T> for Fallible<T> {
    fn merge_with<F>(self: Fallible<T>, other: Fallible<T>, f: F) -> Fallible<T>
    where
        F: FnOnce(T, T) -> T,
    {
        match (self, other) {
            (Err(_), Ok(v)) | (Ok(v), Err(_)) => Ok(v),
            (Ok(v1), Ok(v2)) => Ok(f(v1, v2)),
            (Err(_), Err(e)) => Err(e),
        }
    }
}

impl<I: Interner> RecursiveContext<I> {
    pub(crate) fn new(overflow_depth: usize, caching_enabled: bool) -> Self {
        RecursiveContext {
            stack: Stack::new(overflow_depth),
            search_graph: SearchGraph::new(),
            cache: FxHashMap::default(),
            caching_enabled,
        }
    }

    pub(crate) fn solver<'me>(
        &'me mut self,
        program: &'me dyn RustIrDatabase<I>,
    ) -> Solver<'me, I> {
        Solver {
            program,
            context: self,
        }
    }
}

impl<'me, I: Interner> Solver<'me, I> {
    /// Solves a canonical goal. The substitution returned in the
    /// solution will be for the fully decomposed goal. For example, given the
    /// program
    ///
    /// ```ignore
    /// struct u8 { }
    /// struct SomeType<T> { }
    /// trait Foo<T> { }
    /// impl<U> Foo<u8> for SomeType<U> { }
    /// ```
    ///
    /// and the goal `exists<V> { forall<U> { SomeType<U>: Foo<V> }
    /// }`, `into_peeled_goal` can be used to create a canonical goal
    /// `SomeType<!1>: Foo<?0>`. This function will then return a
    /// solution with the substitution `?0 := u8`.
    pub(crate) fn solve_root_goal(
        &mut self,
        canonical_goal: &UCanonicalGoal<I>,
    ) -> Fallible<Solution<I>> {
        debug!("solve_root_goal(canonical_goal={:?})", canonical_goal);
        assert!(self.context.stack.is_empty());
        let minimums = &mut Minimums::new();
        self.solve_goal(canonical_goal.clone(), minimums)
    }

    /// Attempt to solve a goal that has been fully broken down into leaf form
    /// and canonicalized. This is where the action really happens, and is the
    /// place where we would perform caching in rustc (and may eventually do in Chalk).
    fn solve_goal(
        &mut self,
        goal: UCanonicalGoal<I>,
        minimums: &mut Minimums,
    ) -> Fallible<Solution<I>> {
        info_heading!("solve_goal({:?})", goal);

        // First check the cache.
        if let Some(value) = self.context.cache.get(&goal) {
            debug!("solve_reduced_goal: cache hit, value={:?}", value);
            return value.clone();
        }

        // Next, check if the goal is in the search tree already.
        if let Some(dfn) = self.context.search_graph.lookup(&goal) {
            // Check if this table is still on the stack.
            if let Some(depth) = self.context.search_graph[dfn].stack_depth {
                // Is this a coinductive goal? If so, that is success,
                // so we can return normally. Note that this return is
                // not tabled.
                //
                // XXX how does caching with coinduction work?
                if self.context.stack.coinductive_cycle_from(depth) {
                    let value = ConstrainedSubst {
                        subst: goal.trivial_substitution(self.program.interner()),
                        constraints: vec![],
                    };
                    debug!("applying coinductive semantics");
                    return Ok(Solution::Unique(Canonical {
                        value,
                        binders: goal.canonical.binders,
                    }));
                }

                self.context.stack[depth].flag_cycle();
            }

            minimums.update_from(self.context.search_graph[dfn].links);

            // Return the solution from the table.
            let previous_solution = self.context.search_graph[dfn].solution.clone();
            let previous_solution_priority = self.context.search_graph[dfn].solution_priority;
            info!(
                "solve_goal: cycle detected, previous solution {:?} with prio {:?}",
                previous_solution, previous_solution_priority
            );
            previous_solution
        } else {
            // Otherwise, push the goal onto the stack and create a table.
            // The initial result for this table is error.
            let depth = self.context.stack.push(self.program, &goal);
            let dfn = self.context.search_graph.insert(&goal, depth);
            let subgoal_minimums = self.solve_new_subgoal(goal, depth, dfn);
            self.context.search_graph[dfn].links = subgoal_minimums;
            self.context.search_graph[dfn].stack_depth = None;
            self.context.stack.pop(depth);
            minimums.update_from(subgoal_minimums);

            // Read final result from table.
            let result = self.context.search_graph[dfn].solution.clone();
            let priority = self.context.search_graph[dfn].solution_priority;

            // If processing this subgoal did not involve anything
            // outside of its subtree, then we can promote it to the
            // cache now. This is a sort of hack to alleviate the
            // worst of the repeated work that we do during tabling.
            if subgoal_minimums.positive >= dfn {
                if self.context.caching_enabled {
                    self.context
                        .search_graph
                        .move_to_cache(dfn, &mut self.context.cache);
                    debug!("solve_reduced_goal: SCC head encountered, moving to cache");
                } else {
                    debug!(
                        "solve_reduced_goal: SCC head encountered, rolling back as caching disabled"
                    );
                    self.context.search_graph.rollback_to(dfn);
                }
            }

            info!("solve_goal: solution = {:?} prio {:?}", result, priority);
            result
        }
    }

    fn solve_new_subgoal(
        &mut self,
        canonical_goal: UCanonicalGoal<I>,
        depth: StackDepth,
        dfn: DepthFirstNumber,
    ) -> Minimums {
        debug_heading!(
            "solve_new_subgoal(canonical_goal={:?}, depth={:?}, dfn={:?})",
            canonical_goal,
            depth,
            dfn,
        );

        // We start with `answer = None` and try to solve the goal. At the end of the iteration,
        // `answer` will be updated with the result of the solving process. If we detect a cycle
        // during the solving process, we cache `answer` and try to solve the goal again. We repeat
        // until we reach a fixed point for `answer`.
        // Considering the partial order:
        // - None < Some(Unique) < Some(Ambiguous)
        // - None < Some(CannotProve)
        // the function which maps the loop iteration to `answer` is a nondecreasing function
        // so this function will eventually be constant and the loop terminates.
        let minimums = &mut Minimums::new();
        loop {
            let UCanonical {
                universes,
                canonical:
                    Canonical {
                        binders,
                        value: InEnvironment { environment, goal },
                    },
            } = canonical_goal.clone();

            let (current_answer, current_prio) = match goal.data(self.program.interner()) {
                GoalData::DomainGoal(domain_goal) => {
                    let canonical_goal = UCanonical {
                        universes,
                        canonical: Canonical {
                            binders,
                            value: InEnvironment {
                                environment,
                                goal: domain_goal.clone(),
                            },
                        },
                    };

                    // "Domain" goals (i.e., leaf goals that are Rust-specific) are
                    // always solved via some form of implication. We can either
                    // apply assumptions from our environment (i.e. where clauses),
                    // or from the lowered program, which includes fallback
                    // clauses. We try each approach in turn:

                    let InEnvironment { environment, goal } = &canonical_goal.canonical.value;

                    let (prog_solution, prog_prio) = {
                        debug_heading!("prog_clauses");

                        let prog_clauses = self.program_clauses_for_goal(environment, &goal);
                        match prog_clauses {
                            Ok(clauses) => {
                                self.solve_from_clauses(&canonical_goal, clauses, minimums)
                            }
                            Err(Floundered) => {
                                (Ok(Solution::Ambig(Guidance::Unknown)), ClausePriority::High)
                            }
                        }
                    };
                    debug!("prog_solution={:?}", prog_solution);

                    (prog_solution, prog_prio)
                }

                _ => {
                    let canonical_goal = UCanonical {
                        universes,
                        canonical: Canonical {
                            binders,
                            value: InEnvironment { environment, goal },
                        },
                    };

                    self.solve_via_simplification(&canonical_goal, minimums)
                }
            };

            debug!(
                "solve_new_subgoal: loop iteration result = {:?} with minimums {:?}",
                current_answer, minimums
            );

            if !self.context.stack[depth].read_and_reset_cycle_flag() {
                // None of our subgoals depended on us directly.
                // We can return.
                self.context.search_graph[dfn].solution = current_answer;
                self.context.search_graph[dfn].solution_priority = current_prio;
                return *minimums;
            }

            let old_answer = &self.context.search_graph[dfn].solution;
            let old_prio = self.context.search_graph[dfn].solution_priority;

            let (current_answer, current_prio) = combine_with_priorities_for_goal(
                self.program.interner(),
                &canonical_goal.canonical.value.goal,
                old_answer.clone(),
                old_prio,
                current_answer,
                current_prio,
            );

            // Some of our subgoals depended on us. We need to re-run
            // with the current answer.
            if self.context.search_graph[dfn].solution == current_answer {
                // Reached a fixed point.
                return *minimums;
            }

            let current_answer_is_ambig = match &current_answer {
                Ok(s) => s.is_ambig(),
                Err(_) => false,
            };

            self.context.search_graph[dfn].solution = current_answer;
            self.context.search_graph[dfn].solution_priority = current_prio;

            // Subtle: if our current answer is ambiguous, we can just stop, and
            // in fact we *must* -- otherwise, we sometimes fail to reach a
            // fixed point. See `multiple_ambiguous_cycles` for more.
            if current_answer_is_ambig {
                return *minimums;
            }

            // Otherwise: rollback the search tree and try again.
            self.context.search_graph.rollback_to(dfn + 1);
        }
    }

    fn solve_via_simplification(
        &mut self,
        canonical_goal: &UCanonicalGoal<I>,
        minimums: &mut Minimums,
    ) -> (Fallible<Solution<I>>, ClausePriority) {
        debug_heading!("solve_via_simplification({:?})", canonical_goal);
        let (mut fulfill, subst, goal) = Fulfill::new(self, canonical_goal);
        if let Err(e) = fulfill.push_goal(&goal.environment, goal.goal) {
            return (Err(e), ClausePriority::High);
        }
        (fulfill.solve(subst, minimums), ClausePriority::High)
    }

    /// See whether we can solve a goal by implication on any of the given
    /// clauses. If multiple such solutions are possible, we attempt to combine
    /// them.
    fn solve_from_clauses<C>(
        &mut self,
        canonical_goal: &UCanonical<InEnvironment<DomainGoal<I>>>,
        clauses: C,
        minimums: &mut Minimums,
    ) -> (Fallible<Solution<I>>, ClausePriority)
    where
        C: IntoIterator<Item = ProgramClause<I>>,
    {
        let mut cur_solution = None;
        for program_clause in clauses {
            debug_heading!("clause={:?}", program_clause);

            // If we have a completely ambiguous answer, it's not going to get better, so stop
            if cur_solution == Some((Solution::Ambig(Guidance::Unknown), ClausePriority::High)) {
                return (Ok(Solution::Ambig(Guidance::Unknown)), ClausePriority::High);
            }

            match program_clause.data(self.program.interner()) {
                ProgramClauseData::Implies(implication) => {
                    let res = self.solve_via_implication(
                        canonical_goal,
                        &Binders::new(
                            ParameterKinds::from(self.program.interner(), vec![]),
                            implication.clone(),
                        ),
                        minimums,
                    );
                    if let (Ok(solution), priority) = res {
                        debug!("ok: solution={:?} prio={:?}", solution, priority);
                        cur_solution = Some(match cur_solution {
                            None => (solution, priority),
                            Some((cur, cur_priority)) => combine_with_priorities(
                                self.program.interner(),
                                &canonical_goal.canonical.value.goal,
                                cur,
                                cur_priority,
                                solution,
                                priority,
                            ),
                        });
                    } else {
                        debug!("error");
                    }
                }
                ProgramClauseData::ForAll(implication) => {
                    let res = self.solve_via_implication(canonical_goal, implication, minimums);
                    if let (Ok(solution), priority) = res {
                        debug!("ok: solution={:?} prio={:?}", solution, priority);
                        cur_solution = Some(match cur_solution {
                            None => (solution, priority),
                            Some((cur, cur_priority)) => combine_with_priorities(
                                self.program.interner(),
                                &canonical_goal.canonical.value.goal,
                                cur,
                                cur_priority,
                                solution,
                                priority,
                            ),
                        });
                    } else {
                        debug!("error");
                    }
                }
            }
        }
        cur_solution.map_or((Err(NoSolution), ClausePriority::High), |(s, p)| (Ok(s), p))
    }

    /// Modus ponens! That is: try to apply an implication by proving its premises.
    fn solve_via_implication(
        &mut self,
        canonical_goal: &UCanonical<InEnvironment<DomainGoal<I>>>,
        clause: &Binders<ProgramClauseImplication<I>>,
        minimums: &mut Minimums,
    ) -> (Fallible<Solution<I>>, ClausePriority) {
        info_heading!(
            "solve_via_implication(\
             \n    canonical_goal={:?},\
             \n    clause={:?})",
            canonical_goal,
            clause
        );
        let interner = self.program.interner();
        let (mut fulfill, subst, goal) = Fulfill::new(self, canonical_goal);
        let ProgramClauseImplication {
            consequence,
            conditions,
            priority: _,
        } = fulfill.instantiate_binders_existentially(clause);

        debug!("the subst is {:?}", subst);

        if let Err(e) = fulfill.unify(&goal.environment, &goal.goal, &consequence) {
            return (Err(e), ClausePriority::High);
        }

        // if so, toss in all of its premises
        for condition in conditions.as_slice(interner) {
            if let Err(e) = fulfill.push_goal(&goal.environment, condition.clone()) {
                return (Err(e), ClausePriority::High);
            }
        }

        // and then try to solve
        (
            fulfill.solve(subst, minimums),
            clause.skip_binders().priority,
        )
    }

    fn program_clauses_for_goal(
        &self,
        environment: &Environment<I>,
        goal: &DomainGoal<I>,
    ) -> Result<Vec<ProgramClause<I>>, Floundered> {
        program_clauses_for_goal(self.program, environment, goal)
    }
}

fn calculate_inputs<I: Interner>(
    interner: &I,
    domain_goal: &DomainGoal<I>,
    solution: &Solution<I>,
) -> Vec<Parameter<I>> {
    if let Some(subst) = solution.constrained_subst() {
        let subst_goal = subst.value.subst.apply(&domain_goal, interner);
        subst_goal.inputs(interner)
    } else {
        domain_goal.inputs(interner)
    }
}

fn combine_with_priorities_for_goal<I: Interner>(
    interner: &I,
    goal: &Goal<I>,
    a: Fallible<Solution<I>>,
    prio_a: ClausePriority,
    b: Fallible<Solution<I>>,
    prio_b: ClausePriority,
) -> (Fallible<Solution<I>>, ClausePriority) {
    let domain_goal = match goal.data(interner) {
        GoalData::DomainGoal(domain_goal) => domain_goal,
        _ => {
            // non-domain goals currently have no priorities, so we always take the new solution here
            return (b, prio_b);
        }
    };
    match (a, b) {
        (Ok(a), Ok(b)) => {
            let (solution, prio) =
                combine_with_priorities(interner, domain_goal, a, prio_a, b, prio_b);
            (Ok(solution), prio)
        }
        (Ok(solution), Err(_)) => (Ok(solution), prio_a),
        (Err(_), Ok(solution)) => (Ok(solution), prio_b),
        (Err(_), Err(e)) => (Err(e), prio_b),
    }
}

fn combine_with_priorities<I: Interner>(
    interner: &I,
    domain_goal: &DomainGoal<I>,
    a: Solution<I>,
    prio_a: ClausePriority,
    b: Solution<I>,
    prio_b: ClausePriority,
) -> (Solution<I>, ClausePriority) {
    match (prio_a, prio_b, a, b) {
        (ClausePriority::High, ClausePriority::Low, higher, lower)
        | (ClausePriority::Low, ClausePriority::High, lower, higher) => {
            // if we have a high-priority solution and a low-priority solution,
            // the high-priority solution overrides *if* they are both for the
            // same inputs -- we don't want a more specific high-priority
            // solution overriding a general low-priority one. Currently inputs
            // only matter for projections; in a goal like `AliasEq(<?0 as
            // Trait>::Type = ?1)`, ?0 is the input.
            let inputs_higher = calculate_inputs(interner, domain_goal, &higher);
            let inputs_lower = calculate_inputs(interner, domain_goal, &lower);
            if inputs_higher == inputs_lower {
                debug!(
                    "preferring solution: {:?} over {:?} because of higher prio",
                    higher, lower
                );
                (higher, ClausePriority::High)
            } else {
                (higher.combine(lower, interner), ClausePriority::High)
            }
        }
        (_, _, a, b) => (a.combine(b, interner), prio_a),
    }
}

impl Minimums {
    fn new() -> Self {
        Minimums {
            positive: DepthFirstNumber::MAX,
        }
    }

    fn update_from(&mut self, minimums: Minimums) {
        self.positive = ::std::cmp::min(self.positive, minimums.positive);
    }
}
