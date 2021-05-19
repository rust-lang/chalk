use crate::cache::Cache;
use crate::search_graph::DepthFirstNumber;
use crate::search_graph::SearchGraph;
use crate::solve::{SolveDatabase, SolveIteration};
use crate::stack::{Stack, StackDepth};
use crate::{Minimums, UCanonicalGoal};
use chalk_ir::{interner::Interner, NoSolution};
use chalk_ir::{Canonical, ConstrainedSubst, Goal, InEnvironment, UCanonical};
use chalk_ir::{Constraints, Fallible};
use chalk_solve::{coinductive_goal::IsCoinductive, RustIrDatabase, Solution};
use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;
use tracing::debug;
use tracing::{info, instrument};

struct RecursiveContext<K, V>
where
    K: Hash + Eq + Debug + Clone,
    V: Debug + Clone,
{
    stack: Stack,

    /// The "search graph" stores "in-progress results" that are still being
    /// solved.
    search_graph: SearchGraph<K, V>,

    /// The "cache" stores results for goals that we have completely solved.
    /// Things are added to the cache when we have completely processed their
    /// result.
    cache: Option<Cache<K, V>>,

    /// The maximum size for goals.
    max_size: usize,
}

/// A Solver is the basic context in which you can propose goals for a given
/// program. **All questions posed to the solver are in canonical, closed form,
/// so that each question is answered with effectively a "clean slate"**. This
/// allows for better caching, and simplifies management of the inference
/// context.
struct Solver<'me, I: Interner> {
    program: &'me dyn RustIrDatabase<I>,
    context: &'me mut RecursiveContext<UCanonicalGoal<I>, Fallible<Solution<I>>>,
}

pub struct RecursiveSolver<I: Interner> {
    ctx: Box<RecursiveContext<UCanonicalGoal<I>, Fallible<Solution<I>>>>,
}

impl<I: Interner> RecursiveSolver<I> {
    pub fn new(
        overflow_depth: usize,
        max_size: usize,
        cache: Option<Cache<UCanonicalGoal<I>, Fallible<Solution<I>>>>,
    ) -> Self {
        Self {
            ctx: Box::new(RecursiveContext::new(overflow_depth, max_size, cache)),
        }
    }
}

impl<I: Interner> fmt::Debug for RecursiveSolver<I> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "RecursiveSolver")
    }
}

impl<K, V> RecursiveContext<K, V>
where
    K: Hash + Eq + Debug + Clone,
    V: Debug + Clone,
{
    pub fn new(overflow_depth: usize, max_size: usize, cache: Option<Cache<K, V>>) -> Self {
        RecursiveContext {
            stack: Stack::new(overflow_depth),
            search_graph: SearchGraph::new(),
            cache,
            max_size,
        }
    }

    /// Attempt to solve a goal that has been fully broken down into leaf form
    /// and canonicalized. This is where the action really happens, and is the
    /// place where we would perform caching in rustc (and may eventually do in Chalk).
    #[instrument(
        level = "info",
        skip(
            self,
            minimums,
            is_coinductive_goal,
            initial_value,
            solve_iteration,
            reached_fixed_point,
            error_value
        )
    )]
    fn solve_goal(
        &mut self,
        goal: &K,
        minimums: &mut Minimums,
        is_coinductive_goal: impl Fn(&K) -> bool,
        initial_value: impl Fn(&K, bool) -> V,
        solve_iteration: impl FnMut(&mut Self, &K, &mut Minimums) -> V,
        reached_fixed_point: impl Fn(&V, &V) -> bool,
        error_value: impl Fn() -> V,
    ) -> V {
        // First check the cache.
        if let Some(cache) = &self.cache {
            if let Some(value) = cache.get(&goal) {
                debug!("solve_reduced_goal: cache hit, value={:?}", value);
                return value.clone();
            }
        }

        // Next, check if the goal is in the search tree already.
        if let Some(dfn) = self.search_graph.lookup(&goal) {
            // Check if this table is still on the stack.
            if let Some(depth) = self.search_graph[dfn].stack_depth {
                self.stack[depth].flag_cycle();
                // Mixed cycles are not allowed. For more information about this
                // see the corresponding section in the coinduction chapter:
                // https://rust-lang.github.io/chalk/book/recursive/coinduction.html#mixed-co-inductive-and-inductive-cycles
                if self.stack.mixed_inductive_coinductive_cycle_from(depth) {
                    return error_value();
                }
            }

            minimums.update_from(self.search_graph[dfn].links);

            // Return the solution from the table.
            let previous_solution = self.search_graph[dfn].solution.clone();
            info!(
                "solve_goal: cycle detected, previous solution {:?}",
                previous_solution,
            );
            previous_solution
        } else {
            // Otherwise, push the goal onto the stack and create a table.
            // The initial result for this table depends on whether the goal is coinductive.
            let coinductive_goal = is_coinductive_goal(goal);
            let initial_solution = initial_value(goal, coinductive_goal);
            let depth = self.stack.push(coinductive_goal);
            let dfn = self.search_graph.insert(&goal, depth, initial_solution);

            let subgoal_minimums =
                self.solve_new_subgoal(&goal, depth, dfn, solve_iteration, reached_fixed_point);

            self.search_graph[dfn].links = subgoal_minimums;
            self.search_graph[dfn].stack_depth = None;
            self.stack.pop(depth);
            minimums.update_from(subgoal_minimums);

            // Read final result from table.
            let result = self.search_graph[dfn].solution.clone();

            // If processing this subgoal did not involve anything
            // outside of its subtree, then we can promote it to the
            // cache now. This is a sort of hack to alleviate the
            // worst of the repeated work that we do during tabling.
            if subgoal_minimums.positive >= dfn {
                if let Some(cache) = &mut self.cache {
                    self.search_graph.move_to_cache(dfn, cache);
                    debug!("solve_reduced_goal: SCC head encountered, moving to cache");
                } else {
                    debug!(
                        "solve_reduced_goal: SCC head encountered, rolling back as caching disabled"
                    );
                    self.search_graph.rollback_to(dfn);
                }
            }

            info!("solve_goal: solution = {:?}", result);
            result
        }
    }

    #[instrument(level = "debug", skip(self, solve_iteration, reached_fixed_point))]
    fn solve_new_subgoal(
        &mut self,
        canonical_goal: &K,
        depth: StackDepth,
        dfn: DepthFirstNumber,
        mut solve_iteration: impl FnMut(&mut Self, &K, &mut Minimums) -> V,
        reached_fixed_point: impl Fn(&V, &V) -> bool,
    ) -> Minimums {
        // We start with `answer = None` and try to solve the goal. At the end of the iteration,
        // `answer` will be updated with the result of the solving process. If we detect a cycle
        // during the solving process, we cache `answer` and try to solve the goal again. We repeat
        // until we reach a fixed point for `answer`.
        // Considering the partial order:
        // - None < Some(Unique) < Some(Ambiguous)
        // - None < Some(CannotProve)
        // the function which maps the loop iteration to `answer` is a nondecreasing function
        // so this function will eventually be constant and the loop terminates.
        loop {
            let minimums = &mut Minimums::new();
            let current_answer = solve_iteration(self, &canonical_goal, minimums);

            debug!(
                "solve_new_subgoal: loop iteration result = {:?} with minimums {:?}",
                current_answer, minimums
            );

            if !self.stack[depth].read_and_reset_cycle_flag() {
                // None of our subgoals depended on us directly.
                // We can return.
                self.search_graph[dfn].solution = current_answer;
                return *minimums;
            }

            let old_answer =
                std::mem::replace(&mut self.search_graph[dfn].solution, current_answer);

            if reached_fixed_point(&old_answer, &self.search_graph[dfn].solution) {
                return *minimums;
            }

            // Otherwise: rollback the search tree and try again.
            self.search_graph.rollback_to(dfn + 1);
        }
    }
}

impl<'me, I: Interner> Solver<'me, I> {
    pub(crate) fn new(
        context: &'me mut RecursiveContext<UCanonicalGoal<I>, Fallible<Solution<I>>>,
        program: &'me dyn RustIrDatabase<I>,
    ) -> Self {
        Self { program, context }
    }

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
}

impl<'me, I: Interner> SolveDatabase<I> for Solver<'me, I> {
    fn solve_goal(
        &mut self,
        goal: UCanonicalGoal<I>,
        minimums: &mut Minimums,
    ) -> Fallible<Solution<I>> {
        let program = self.program;
        let interner = program.interner();
        self.context.solve_goal(
            &goal,
            minimums,
            |goal| goal.is_coinductive(program),
            |goal, coinductive_goal| {
                if coinductive_goal {
                    Ok(Solution::Unique(Canonical {
                        value: ConstrainedSubst {
                            subst: goal.trivial_substitution(interner),
                            constraints: Constraints::empty(interner),
                        },
                        binders: goal.canonical.binders.clone(),
                    }))
                } else {
                    Err(NoSolution)
                }
            },
            |context, goal, minimums| Solver::new(context, program).solve_iteration(goal, minimums),
            |old_answer, current_answer| {
                // Some of our subgoals depended on us. We need to re-run
                // with the current answer.
                old_answer == current_answer || {
                    // Subtle: if our current answer is ambiguous, we can just stop, and
                    // in fact we *must* -- otherwise, we sometimes fail to reach a
                    // fixed point. See `multiple_ambiguous_cycles` for more.
                    match &current_answer {
                        Ok(s) => s.is_ambig(),
                        Err(_) => false,
                    }
                }
            },
            || Err(NoSolution),
        )
    }

    fn interner(&self) -> &I {
        &self.program.interner()
    }

    fn db(&self) -> &dyn RustIrDatabase<I> {
        self.program
    }

    fn max_size(&self) -> usize {
        self.context.max_size
    }
}

impl<I: Interner> chalk_solve::Solver<I> for RecursiveSolver<I> {
    fn solve(
        &mut self,
        program: &dyn RustIrDatabase<I>,
        goal: &UCanonical<InEnvironment<Goal<I>>>,
    ) -> Option<chalk_solve::Solution<I>> {
        Solver::new(&mut self.ctx, program)
            .solve_root_goal(goal)
            .ok()
    }

    fn solve_limited(
        &mut self,
        program: &dyn RustIrDatabase<I>,
        goal: &UCanonical<InEnvironment<Goal<I>>>,
        _should_continue: &dyn std::ops::Fn() -> bool,
    ) -> Option<chalk_solve::Solution<I>> {
        // TODO support should_continue in recursive solver
        Solver::new(&mut self.ctx, program)
            .solve_root_goal(goal)
            .ok()
    }

    fn solve_multiple(
        &mut self,
        _program: &dyn RustIrDatabase<I>,
        _goal: &UCanonical<InEnvironment<Goal<I>>>,
        _f: &mut dyn FnMut(
            chalk_solve::SubstitutionResult<Canonical<ConstrainedSubst<I>>>,
            bool,
        ) -> bool,
    ) -> bool {
        unimplemented!("Recursive solver doesn't support multiple answers")
    }
}
