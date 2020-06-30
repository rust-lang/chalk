use self::search_graph::SearchGraph;
use self::solve::{SolveDatabase, SolveIteration};
use self::stack::{Stack, StackDepth};
use crate::search_graph::DepthFirstNumber;
use chalk_ir::interner::Interner;
use chalk_ir::Fallible;
use chalk_ir::{Canonical, Constraints, ConstrainedSubst, Goal, InEnvironment, Substitution, UCanonical};
use chalk_solve::{coinductive_goal::IsCoinductive, RustIrDatabase};
use rustc_hash::FxHashMap;
use std::fmt;
use tracing::debug;
use tracing::{info, instrument};

pub type UCanonicalGoal<I> = UCanonical<InEnvironment<Goal<I>>>;

mod combine;
mod fulfill;
mod search_graph;
pub mod solve;
mod stack;
/// The `minimums` struct is used while solving to track whether we encountered
/// any cycles in the process.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Minimums {
    pub(crate) positive: DepthFirstNumber,
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

/// A (possible) solution for a proposed goal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Solution<I: Interner> {
    /// The goal indeed holds, and there is a unique value for all existential
    /// variables. In this case, we also record a set of lifetime constraints
    /// which must also hold for the goal to be valid.
    Unique(Canonical<ConstrainedSubst<I>>),

    /// The goal may be provable in multiple ways, but regardless we may have some guidance
    /// for type inference. In this case, we don't return any lifetime
    /// constraints, since we have not "committed" to any particular solution
    /// yet.
    Ambig(Guidance<I>),
}

/// When a goal holds ambiguously (e.g., because there are multiple possible
/// solutions), we issue a set of *guidance* back to type inference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Guidance<I: Interner> {
    /// The existential variables *must* have the given values if the goal is
    /// ever to hold, but that alone isn't enough to guarantee the goal will
    /// actually hold.
    Definite(Canonical<Substitution<I>>),

    /// There are multiple plausible values for the existentials, but the ones
    /// here are suggested as the preferred choice heuristically. These should
    /// be used for inference fallback only.
    Suggested(Canonical<Substitution<I>>),

    /// There's no useful information to feed back to type inference
    Unknown,
}

impl<I: Interner> Solution<I> {
    /// There are multiple candidate solutions, which may or may not agree on
    /// the values for existential variables; attempt to combine them. This
    /// operation does not depend on the order of its arguments.
    //
    // This actually isn't as precise as it could be, in two ways:
    //
    // a. It might be that while there are multiple distinct candidates, they
    //    all agree about *some things*. To be maximally precise, we would
    //    compute the intersection of what they agree on. It's not clear though
    //    that this is actually what we want Rust's inference to do, and it's
    //    certainly not what it does today.
    //
    // b. There might also be an ambiguous candidate and a successful candidate,
    //    both with the same refined-goal. In that case, we could probably claim
    //    success, since if the conditions of the ambiguous candidate were met,
    //    we know the success would apply.  Example: `?0: Clone` yields ambiguous
    //    candidate `Option<?0>: Clone` and successful candidate `Option<?0>:
    //    Clone`.
    //
    // But you get the idea.
    pub(crate) fn combine(self, other: Solution<I>, interner: &I) -> Solution<I> {
        use self::Guidance::*;

        if self == other {
            return self;
        }

        debug!(
            "combine {} with {}",
            self.display(interner),
            other.display(interner)
        );

        // Otherwise, always downgrade to Ambig:

        let guidance = match (self.into_guidance(), other.into_guidance()) {
            (Definite(ref subst1), Definite(ref subst2)) if subst1 == subst2 => {
                Definite(subst1.clone())
            }
            (Suggested(ref subst1), Suggested(ref subst2)) if subst1 == subst2 => {
                Suggested(subst1.clone())
            }
            _ => Unknown,
        };
        Solution::Ambig(guidance)
    }

    /// View this solution purely in terms of type inference guidance
    pub(crate) fn into_guidance(self) -> Guidance<I> {
        match self {
            Solution::Unique(constrained) => Guidance::Definite(Canonical {
                value: constrained.value.subst,
                binders: constrained.binders,
            }),
            Solution::Ambig(guidance) => guidance,
        }
    }

    /// Extract a constrained substitution from this solution, even if ambiguous.
    pub(crate) fn constrained_subst(&self, interner: &I) -> Option<Canonical<ConstrainedSubst<I>>> {
        match *self {
            Solution::Unique(ref constrained) => Some(constrained.clone()),
            Solution::Ambig(Guidance::Definite(ref canonical))
            | Solution::Ambig(Guidance::Suggested(ref canonical)) => {
                let value = ConstrainedSubst {
                    subst: canonical.value.clone(),
                    constraints: Constraints::empty(interner),
                };
                Some(Canonical {
                    value,
                    binders: canonical.binders.clone(),
                })
            }
            Solution::Ambig(_) => None,
        }
    }

    /// Determine whether this solution contains type information that *must*
    /// hold.
    pub(crate) fn has_definite(&self) -> bool {
        match *self {
            Solution::Unique(_) => true,
            Solution::Ambig(Guidance::Definite(_)) => true,
            _ => false,
        }
    }

    pub fn is_unique(&self) -> bool {
        match *self {
            Solution::Unique(..) => true,
            _ => false,
        }
    }

    pub(crate) fn is_ambig(&self) -> bool {
        match *self {
            Solution::Ambig(_) => true,
            _ => false,
        }
    }

    pub fn display<'a>(&'a self, interner: &'a I) -> SolutionDisplay<'a, I> {
        SolutionDisplay {
            solution: self,
            interner,
        }
    }
}

pub struct SolutionDisplay<'a, I: Interner> {
    solution: &'a Solution<I>,
    interner: &'a I,
}

impl<'a, I: Interner> fmt::Display for SolutionDisplay<'a, I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let SolutionDisplay { solution, interner } = self;
        match solution {
            Solution::Unique(constrained) => write!(f, "Unique; {}", constrained.display(interner)),
            Solution::Ambig(Guidance::Definite(subst)) => write!(
                f,
                "Ambiguous; definite substitution {}",
                subst.display(interner)
            ),
            Solution::Ambig(Guidance::Suggested(subst)) => write!(
                f,
                "Ambiguous; suggested substitution {}",
                subst.display(interner)
            ),
            Solution::Ambig(Guidance::Unknown) => write!(f, "Ambiguous; no inference guidance"),
        }
    }
}

pub struct RecursiveContext<I: Interner> {
    stack: Stack,

    /// The "search graph" stores "in-progress results" that are still being
    /// solved.
    search_graph: SearchGraph<I>,

    /// The "cache" stores results for goals that we have completely solved.
    /// Things are added to the cache when we have completely processed their
    /// result.
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

pub struct RecursiveSolverImpl<I: Interner> {
    pub ctx: Box<RecursiveContext<I>>,
}

impl<I: Interner> fmt::Debug for RecursiveSolverImpl<I> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "RecursiveSolverImpl")
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
    pub fn new(overflow_depth: usize, caching_enabled: bool) -> Self {
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
    #[instrument(level = "debug", skip(self))]
    pub(crate) fn solve_root_goal(
        &mut self,
        canonical_goal: &UCanonicalGoal<I>,
    ) -> Fallible<Solution<I>> {
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
                "loop iteration result = {:?} with minimums {:?}",
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

            let (current_answer, current_prio) = combine::with_priorities_for_goal(
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
            debug!(?value, "cache hit");
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
                        constraints: Constraints::empty(self.program.interner()),
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
            let coinductive_goal = goal.is_coinductive(self.program);
            let depth = self.context.stack.push(coinductive_goal);
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
                    debug!(target: "solve_goal", "SCC head encountered, moving to cache");
                } else {
                    debug!(
                        target: "solve_goal", "SCC head encountered, rolling back as caching disabled"
                    );
                    self.context.search_graph.rollback_to(dfn);
                }
            }

            info!(target = "solve_goal", solution = ?result, prio = ?priority);
            result
        }
    }

    fn interner(&self) -> &I {
        &self.program.interner()
    }

    fn db(&self) -> &dyn RustIrDatabase<I> {
        self.program
    }
}

impl<I: Interner> chalk_solve::Solver<I> for RecursiveSolverImpl<I> {
    fn solve(
        &mut self,
        program: &dyn RustIrDatabase<I>,
        goal: &UCanonical<InEnvironment<Goal<I>>>,
    ) -> Option<chalk_solve::Solution<I>> {
        self.ctx
            .solver(program)
            .solve_root_goal(goal)
            .ok()
            .map(|s| match s {
                Solution::Unique(c) => chalk_solve::Solution::Unique(c),
                Solution::Ambig(g) => chalk_solve::Solution::Ambig(match g {
                    Guidance::Definite(g) => chalk_solve::Guidance::Definite(g),
                    Guidance::Suggested(g) => chalk_solve::Guidance::Suggested(g),
                    Guidance::Unknown => chalk_solve::Guidance::Unknown,
                }),
            })
    }

    fn solve_limited(
        &mut self,
        program: &dyn RustIrDatabase<I>,
        goal: &UCanonical<InEnvironment<Goal<I>>>,
        _should_continue: impl std::ops::Fn() -> bool,
    ) -> Option<chalk_solve::Solution<I>> {
        // TODO support should_continue in recursive solver
        self.ctx
            .solver(program)
            .solve_root_goal(goal)
            .ok()
            .map(|s| match s {
                Solution::Unique(c) => chalk_solve::Solution::Unique(c),
                Solution::Ambig(g) => chalk_solve::Solution::Ambig(match g {
                    Guidance::Definite(g) => chalk_solve::Guidance::Definite(g),
                    Guidance::Suggested(g) => chalk_solve::Guidance::Suggested(g),
                    Guidance::Unknown => chalk_solve::Guidance::Unknown,
                }),
            })
    }

    fn solve_multiple(
        &mut self,
        _program: &dyn RustIrDatabase<I>,
        _goal: &UCanonical<InEnvironment<Goal<I>>>,
        _f: impl FnMut(chalk_solve::SubstitutionResult<Canonical<ConstrainedSubst<I>>>, bool) -> bool,
    ) -> bool {
        unimplemented!()
    }
}
