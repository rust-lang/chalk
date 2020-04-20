use crate::solve::slg::SlgContext;
use crate::{recursive::RecursiveContext, RustIrDatabase};
use chalk_engine::forest::{Forest, SubstitutionResult};
use chalk_ir::interner::Interner;
use chalk_ir::*;
use std::fmt;

mod slg;
pub(crate) mod truncate;

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
    pub fn is_unique(&self) -> bool {
        match *self {
            Solution::Unique(..) => true,
            _ => false,
        }
    }

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
    pub(crate) fn constrained_subst(&self) -> Option<Canonical<ConstrainedSubst<I>>> {
        match *self {
            Solution::Unique(ref constrained) => Some(constrained.clone()),
            Solution::Ambig(Guidance::Definite(ref canonical))
            | Solution::Ambig(Guidance::Suggested(ref canonical)) => {
                let value = ConstrainedSubst {
                    subst: canonical.value.clone(),
                    constraints: vec![],
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

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum SolverChoice {
    /// Run the SLG solver, producing a Solution.
    SLG {
        max_size: usize,
        expected_answers: Option<usize>,
    },
    /// Run the recursive solver.
    Recursive {
        overflow_depth: usize,
        caching_enabled: bool,
    },
}

impl SolverChoice {
    /// Returns specific SLG parameters.
    pub fn slg(max_size: usize, expected_answers: Option<usize>) -> Self {
        SolverChoice::SLG {
            max_size,
            expected_answers,
        }
    }

    /// Returns the default SLG parameters.
    pub fn slg_default() -> Self {
        SolverChoice::slg(10, None)
    }

    /// Returns the default recursive solver setup.
    pub fn recursive() -> Self {
        SolverChoice::Recursive {
            overflow_depth: 100,
            caching_enabled: true,
        }
    }

    /// Creates a solver state.
    pub fn into_solver<I: Interner>(self) -> Solver<I> {
        match self {
            SolverChoice::SLG {
                max_size,
                expected_answers,
            } => Solver(SolverImpl::Slg {
                forest: Box::new(Forest::new(SlgContext::new(max_size, expected_answers))),
            }),
            SolverChoice::Recursive {
                overflow_depth,
                caching_enabled,
            } => Solver(SolverImpl::Recursive(Box::new(RecursiveContext::new(
                overflow_depth,
                caching_enabled,
            )))),
        }
    }
}

impl Default for SolverChoice {
    fn default() -> Self {
        // SolverChoice::recursive()
        SolverChoice::slg(10, None)
    }
}

/// Finds the solution to "goals", or trait queries -- i.e., figures
/// out what sets of types implement which traits. Also, between
/// queries, this struct stores the cached state from previous solver
/// attempts, which can then be re-used later.
pub struct Solver<I: Interner>(SolverImpl<I>);

enum SolverImpl<I: Interner> {
    Slg { forest: Box<Forest<SlgContext<I>>> },
    Recursive(Box<RecursiveContext<I>>),
}

impl<I: Interner> Solver<I> {
    /// Attempts to solve the given goal, which must be in canonical
    /// form. Returns a unique solution (if one exists).  This will do
    /// only as much work towards `goal` as it has to (and that work
    /// is cached for future attempts).
    ///
    /// # Parameters
    ///
    /// - `program` -- defines the program clauses in scope.
    ///   - **Important:** You must supply the same set of program clauses
    ///     each time you invoke `solve`, as otherwise the cached data may be
    ///     invalid.
    /// - `goal` the goal to solve
    ///
    /// # Returns
    ///
    /// - `None` is the goal cannot be proven.
    /// - `Some(solution)` if we succeeded in finding *some* answers,
    ///   although `solution` may reflect ambiguity and unknowns.
    pub fn solve(
        &mut self,
        program: &dyn RustIrDatabase<I>,
        goal: &UCanonical<InEnvironment<Goal<I>>>,
    ) -> Option<Solution<I>> {
        match &mut self.0 {
            SolverImpl::Slg { forest } => {
                let ops = forest.context().ops(program);
                forest.solve(&ops, goal, || true)
            }
            SolverImpl::Recursive(ctx) => ctx.solver(program).solve_root_goal(goal).ok(),
        }
    }

    /// Attempts to solve the given goal, which must be in canonical
    /// form. Returns a unique solution (if one exists).  This will do
    /// only as much work towards `goal` as it has to (and that work
    /// is cached for future attempts). In addition, the solving of the
    /// goal can be limited by returning `false` from `should_continue`.
    ///
    /// # Parameters
    ///
    /// - `program` -- defines the program clauses in scope.
    ///   - **Important:** You must supply the same set of program clauses
    ///     each time you invoke `solve`, as otherwise the cached data may be
    ///     invalid.
    /// - `goal` the goal to solve
    /// - `should_continue` if `false` is returned, the no further solving
    ///   will be done. A `Guidance(Suggested(...))` will be returned a
    ///   `Solution`, using any answers that were generated up to that point.
    ///
    /// # Returns
    ///
    /// - `None` is the goal cannot be proven.
    /// - `Some(solution)` if we succeeded in finding *some* answers,
    ///   although `solution` may reflect ambiguity and unknowns.
    pub fn solve_limited(
        &mut self,
        program: &dyn RustIrDatabase<I>,
        goal: &UCanonical<InEnvironment<Goal<I>>>,
        should_continue: impl std::ops::Fn() -> bool,
    ) -> Option<Solution<I>> {
        match &mut self.0 {
            SolverImpl::Slg { forest } => {
                let ops = forest.context().ops(program);
                forest.solve(&ops, goal, should_continue)
            }
            SolverImpl::Recursive(ctx) => {
                // TODO support should_continue in recursive solver
                ctx.solver(program).solve_root_goal(goal).ok()
            }
        }
    }

    /// Attempts to solve the given goal, which must be in canonical
    /// form. Provides multiple solutions to function `f`.  This will do
    /// only as much work towards `goal` as it has to (and that work
    /// is cached for future attempts).
    ///
    /// # Parameters
    ///
    /// - `program` -- defines the program clauses in scope.
    ///   - **Important:** You must supply the same set of program clauses
    ///     each time you invoke `solve`, as otherwise the cached data may be
    ///     invalid.
    /// - `goal` the goal to solve
    /// - `f` -- function to proceed solution. New solutions will be generated
    /// while function returns `true`.
    ///   - first argument is solution found
    ///   - second argument is ther next solution present
    ///   - returns true if next solution should be handled
    ///
    /// # Returns
    ///
    /// - `true` all solutions were processed with the function.
    /// - `false` the function returned `false` and solutions were interrupted.
    pub fn solve_multiple(
        &mut self,
        program: &dyn RustIrDatabase<I>,
        goal: &UCanonical<InEnvironment<Goal<I>>>,
        f: impl FnMut(SubstitutionResult<Canonical<ConstrainedSubst<I>>>, bool) -> bool,
    ) -> bool {
        match &mut self.0 {
            SolverImpl::Slg { forest } => {
                let ops = forest.context().ops(program);
                forest.solve_multiple(&ops, goal, f)
            }
            SolverImpl::Recursive(_ctx) => unimplemented!(),
        }
    }
}

impl<I: Interner> std::fmt::Debug for Solver<I> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "Solver {{ .. }}")
    }
}
