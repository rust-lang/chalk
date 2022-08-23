use crate::fixed_point::{Cache, Minimums, RecursiveContext, SolverStuff};
use crate::solve::{SolveDatabase, SolveIteration};
use crate::UCanonicalGoal;
use chalk_ir::{interner::Interner, NoSolution};
use chalk_ir::{Canonical, ConstrainedSubst, Goal, InEnvironment, UCanonical};
use chalk_ir::{Constraints, Fallible};
use chalk_solve::{coinductive_goal::IsCoinductive, RustIrDatabase, Solution};
use std::fmt;

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

impl<'me, I: Interner> Solver<'me, I> {
    pub(crate) fn new(
        context: &'me mut RecursiveContext<UCanonicalGoal<I>, Fallible<Solution<I>>>,
        program: &'me dyn RustIrDatabase<I>,
    ) -> Self {
        Self { program, context }
    }
}

impl<I: Interner> SolverStuff<UCanonicalGoal<I>, Fallible<Solution<I>>> for &dyn RustIrDatabase<I> {
    fn is_coinductive_goal(self, goal: &UCanonicalGoal<I>) -> bool {
        goal.is_coinductive(self)
    }

    fn initial_value(
        self,
        goal: &UCanonicalGoal<I>,
        coinductive_goal: bool,
    ) -> Fallible<Solution<I>> {
        if coinductive_goal {
            Ok(Solution::Unique(Canonical {
                value: ConstrainedSubst {
                    subst: goal.trivial_substitution(self.interner()),
                    constraints: Constraints::empty(self.interner()),
                },
                binders: goal.canonical.binders.clone(),
            }))
        } else {
            Err(NoSolution)
        }
    }

    fn solve_iteration(
        self,
        context: &mut RecursiveContext<UCanonicalGoal<I>, Fallible<Solution<I>>>,
        goal: &UCanonicalGoal<I>,
        minimums: &mut Minimums,
        should_continue: impl std::ops::Fn() -> bool + Clone,
    ) -> Fallible<Solution<I>> {
        Solver::new(context, self).solve_iteration(goal, minimums, should_continue)
    }

    fn reached_fixed_point(
        self,
        old_answer: &Fallible<Solution<I>>,
        current_answer: &Fallible<Solution<I>>,
    ) -> bool {
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
    }

    fn error_value(self) -> Fallible<Solution<I>> {
        Err(NoSolution)
    }
}

impl<'me, I: Interner> SolveDatabase<I> for Solver<'me, I> {
    fn solve_goal(
        &mut self,
        goal: UCanonicalGoal<I>,
        minimums: &mut Minimums,
        should_continue: impl std::ops::Fn() -> bool + Clone,
    ) -> Fallible<Solution<I>> {
        self.context
            .solve_goal(&goal, minimums, self.program, should_continue)
    }

    fn interner(&self) -> I {
        self.program.interner()
    }

    fn db(&self) -> &dyn RustIrDatabase<I> {
        self.program
    }

    fn max_size(&self) -> usize {
        self.context.max_size()
    }
}

impl<I: Interner> chalk_solve::Solver<I> for RecursiveSolver<I> {
    fn solve(
        &mut self,
        program: &dyn RustIrDatabase<I>,
        goal: &UCanonical<InEnvironment<Goal<I>>>,
    ) -> Option<chalk_solve::Solution<I>> {
        self.ctx.solve_root_goal(goal, program, || true).ok()
    }

    fn solve_limited(
        &mut self,
        program: &dyn RustIrDatabase<I>,
        goal: &UCanonical<InEnvironment<Goal<I>>>,
        should_continue: &dyn std::ops::Fn() -> bool,
    ) -> Option<chalk_solve::Solution<I>> {
        self.ctx
            .solve_root_goal(goal, program, should_continue)
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
