use crate::search_graph::DepthFirstNumber;
use crate::search_graph::SearchGraph;
use crate::solve::{SolveDatabase, SolveIteration};
use crate::stack::{Stack, StackDepth};
use crate::{Minimums, UCanonicalGoal};
use chalk_ir::Fallible;
use chalk_ir::{interner::Interner, NoSolution};
use chalk_ir::{Canonical, ConstrainedSubst, Goal, InEnvironment, UCanonical};
use chalk_solve::{coinductive_goal::IsCoinductive, RustIrDatabase, Solution};
use rustc_hash::FxHashMap;
use std::fmt;
use tracing::debug;
use tracing::{info, instrument};

struct RecursiveContext<I: Interner> {
    stack: Stack,

    /// The "search graph" stores "in-progress results" that are still being
    /// solved.
    search_graph: SearchGraph<I>,

    /// The "cache" stores results for goals that we have completely solved.
    /// Things are added to the cache when we have completely processed their
    /// result.
    cache: FxHashMap<UCanonicalGoal<I>, Fallible<Solution<I>>>,

    /// The maximum size for goals.
    max_size: usize,

    caching_enabled: bool,
}

/// A Solver is the basic context in which you can propose goals for a given
/// program. **All questions posed to the solver are in canonical, closed form,
/// so that each question is answered with effectively a "clean slate"**. This
/// allows for better caching, and simplifies management of the inference
/// context.
struct Solver<'me, I: Interner> {
    program: &'me dyn RustIrDatabase<I>,
    context: &'me mut RecursiveContext<I>,
}

pub struct RecursiveSolver<I: Interner> {
    ctx: Box<RecursiveContext<I>>,
}

impl<I: Interner> RecursiveSolver<I> {
    pub fn new(overflow_depth: usize, max_size: usize, caching_enabled: bool) -> Self {
        Self {
            ctx: Box::new(RecursiveContext::new(
                overflow_depth,
                max_size,
                caching_enabled,
            )),
        }
    }
}

impl<I: Interner> fmt::Debug for RecursiveSolver<I> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "RecursiveSolver")
    }
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
    pub fn new(overflow_depth: usize, max_size: usize, caching_enabled: bool) -> Self {
        RecursiveContext {
            stack: Stack::new(overflow_depth),
            search_graph: SearchGraph::new(),
            cache: FxHashMap::default(),
            max_size,
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

    #[instrument(level = "debug", skip(self))]
    fn solve_new_subgoal(
        &mut self,
        canonical_goal: UCanonicalGoal<I>,
        depth: StackDepth,
        dfn: DepthFirstNumber,
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
            let (current_answer, current_prio) = self.solve_iteration(&canonical_goal, minimums);

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
}

impl<'me, I: Interner> SolveDatabase<I> for Solver<'me, I> {
    /// Attempt to solve a goal that has been fully broken down into leaf form
    /// and canonicalized. This is where the action really happens, and is the
    /// place where we would perform caching in rustc (and may eventually do in Chalk).
    #[instrument(level = "info", skip(self, minimums))]
    fn solve_goal(
        &mut self,
        goal: UCanonicalGoal<I>,
        minimums: &mut Minimums,
    ) -> Fallible<Solution<I>> {
        // First check the cache.
        if let Some(value) = self.context.cache.get(&goal) {
            debug!("solve_reduced_goal: cache hit, value={:?}", value);
            return value.clone();
        }

        // Next, check if the goal is in the search tree already.
        if let Some(dfn) = self.context.search_graph.lookup(&goal) {
            // Check if this table is still on the stack.
            if let Some(depth) = self.context.search_graph[dfn].stack_depth {
                self.context.stack[depth].flag_cycle();
                // Mixed cycles are not allowed. For more information about this
                // see the corresponding section in the coinduction chapter:
                // https://rust-lang.github.io/chalk/book/recursive/coinduction.html#mixed-co-inductive-and-inductive-cycles
                if self
                    .context
                    .stack
                    .mixed_inductive_coinductive_cycle_from(depth)
                {
                    return Err(NoSolution);
                }
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
            // The initial result for this table depends on whether the goal is coinductive.
            let coinductive_goal = goal.is_coinductive(self.program);
            let depth = self.context.stack.push(coinductive_goal);
            let dfn = self.context.search_graph.insert(
                &goal,
                depth,
                coinductive_goal,
                self.program.interner(),
            );
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
        self.ctx.solver(program).solve_root_goal(goal).ok()
    }

    fn solve_limited(
        &mut self,
        program: &dyn RustIrDatabase<I>,
        goal: &UCanonical<InEnvironment<Goal<I>>>,
        _should_continue: &dyn std::ops::Fn() -> bool,
    ) -> Option<chalk_solve::Solution<I>> {
        // TODO support should_continue in recursive solver
        self.ctx.solver(program).solve_root_goal(goal).ok()
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
