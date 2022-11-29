use std::fmt::Debug;
use std::hash::Hash;
use tracing::debug;
use tracing::{info, instrument};

mod cache;
mod search_graph;
mod stack;

pub use cache::Cache;
use search_graph::{DepthFirstNumber, SearchGraph};
use stack::{Stack, StackDepth};

pub(super) struct RecursiveContext<K, V>
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

pub(super) trait SolverStuff<K, V>: Copy
where
    K: Hash + Eq + Debug + Clone,
    V: Debug + Clone,
{
    fn is_coinductive_goal(self, goal: &K) -> bool;
    fn initial_value(self, goal: &K, coinductive_goal: bool) -> V;
    fn solve_iteration(
        self,
        context: &mut RecursiveContext<K, V>,
        goal: &K,
        minimums: &mut Minimums,
        should_continue: impl std::ops::Fn() -> bool + Clone,
    ) -> V;
    fn reached_fixed_point(self, old_value: &V, new_value: &V) -> bool;
    fn error_value(self) -> V;
}

/// The `minimums` struct is used while solving to track whether we encountered
/// any cycles in the process.
#[derive(Copy, Clone, Debug)]
pub(super) struct Minimums {
    positive: DepthFirstNumber,
}

impl Minimums {
    pub fn new() -> Self {
        Minimums {
            positive: DepthFirstNumber::MAX,
        }
    }

    pub fn update_from(&mut self, minimums: Minimums) {
        self.positive = ::std::cmp::min(self.positive, minimums.positive);
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

    pub fn max_size(&self) -> usize {
        self.max_size
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
    pub fn solve_root_goal(
        &mut self,
        canonical_goal: &K,
        solver_stuff: impl SolverStuff<K, V>,
        should_continue: impl std::ops::Fn() -> bool + Clone,
    ) -> V {
        debug!("solve_root_goal(canonical_goal={:?})", canonical_goal);
        assert!(self.stack.is_empty());
        let minimums = &mut Minimums::new();
        self.solve_goal(canonical_goal, minimums, solver_stuff, should_continue)
    }

    /// Attempt to solve a goal that has been fully broken down into leaf form
    /// and canonicalized. This is where the action really happens, and is the
    /// place where we would perform caching in rustc (and may eventually do in Chalk).
    #[instrument(level = "info", skip(self, minimums, solver_stuff, should_continue))]
    pub fn solve_goal(
        &mut self,
        goal: &K,
        minimums: &mut Minimums,
        solver_stuff: impl SolverStuff<K, V>,
        should_continue: impl std::ops::Fn() -> bool + Clone,
    ) -> V {
        // First check the cache.
        if let Some(cache) = &self.cache {
            if let Some(value) = cache.get(goal) {
                debug!("solve_reduced_goal: cache hit, value={:?}", value);
                return value;
            }
        }

        // Next, check if the goal is in the search tree already.
        if let Some(dfn) = self.search_graph.lookup(goal) {
            // Check if this table is still on the stack.
            if let Some(depth) = self.search_graph[dfn].stack_depth {
                self.stack[depth].flag_cycle();
                // Mixed cycles are not allowed. For more information about this
                // see the corresponding section in the coinduction chapter:
                // https://rust-lang.github.io/chalk/book/recursive/coinduction.html#mixed-co-inductive-and-inductive-cycles
                if self.stack.mixed_inductive_coinductive_cycle_from(depth) {
                    return solver_stuff.error_value();
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
            let coinductive_goal = solver_stuff.is_coinductive_goal(goal);
            let initial_solution = solver_stuff.initial_value(goal, coinductive_goal);
            let depth = self.stack.push(coinductive_goal);
            let dfn = self.search_graph.insert(goal, depth, initial_solution);

            let subgoal_minimums =
                self.solve_new_subgoal(goal, depth, dfn, solver_stuff, should_continue);

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

    #[instrument(level = "debug", skip(self, solver_stuff, should_continue))]
    fn solve_new_subgoal(
        &mut self,
        canonical_goal: &K,
        depth: StackDepth,
        dfn: DepthFirstNumber,
        solver_stuff: impl SolverStuff<K, V>,
        should_continue: impl std::ops::Fn() -> bool + Clone,
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
            let current_answer = solver_stuff.solve_iteration(
                self,
                canonical_goal,
                minimums,
                should_continue.clone(), // Note: cloning required as workaround for https://github.com/rust-lang/rust/issues/95734
            );

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

            if solver_stuff.reached_fixed_point(&old_answer, &self.search_graph[dfn].solution) {
                return *minimums;
            }

            // Otherwise: rollback the search tree and try again.
            self.search_graph.rollback_to(dfn + 1);
        }
    }
}
