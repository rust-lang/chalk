use crate::RustIrDatabase;
use chalk_ir::interner::Interner;
use chalk_ir::*;
use std::fmt;

#[cfg(feature = "slg-solver")]
pub use crate::solve::slg::SubstitutionResult;
#[cfg(feature = "slg-solver")]
use {
    crate::solve::slg::{aggregate::AggregateOps, SlgContext, SlgContextOps},
    chalk_engine::context::{AnswerResult, AnswerStream, ContextOps},
    chalk_engine::forest::Forest,
};

#[cfg(feature = "recursive-solver")]
use crate::recursive::RecursiveContext;

#[cfg(feature = "slg-solver")]
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
    #[cfg(feature = "slg-solver")]
    SLG {
        max_size: usize,
        expected_answers: Option<usize>,
    },
    /// Run the recursive solver.
    #[cfg(feature = "recursive-solver")]
    Recursive {
        overflow_depth: usize,
        caching_enabled: bool,
    },
}

impl SolverChoice {
    /// Returns specific SLG parameters.
    #[cfg(feature = "slg-solver")]
    pub fn slg(max_size: usize, expected_answers: Option<usize>) -> Self {
        SolverChoice::SLG {
            max_size,
            expected_answers,
        }
    }

    /// Returns the default SLG parameters.
    #[cfg(feature = "slg-solver")]
    pub fn slg_default() -> Self {
        SolverChoice::slg(10, None)
    }

    /// Returns the default recursive solver setup.
    #[cfg(feature = "recursive-solver")]
    pub fn recursive() -> Self {
        SolverChoice::Recursive {
            overflow_depth: 100,
            caching_enabled: true,
        }
    }

    /// Creates a solver state.
    pub fn into_solver<I: Interner>(self) -> Solver<I> {
        match self {
            #[cfg(feature = "slg-solver")]
            SolverChoice::SLG {
                max_size,
                expected_answers,
            } => Solver(SolverImpl::Slg {
                forest: Box::new(Forest::new()),
                max_size,
                expected_answers,
            }),
            #[cfg(feature = "recursive-solver")]
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

#[cfg(feature = "slg-solver")]
impl Default for SolverChoice {
    fn default() -> Self {
        SolverChoice::slg_default()
    }
}

#[cfg(all(not(feature = "slg-solver"), feature = "recursive-solver"))]
impl Default for SolverChoice {
    fn default() -> Self {
        SolverChoice::recursive()
    }
}

/// Finds the solution to "goals", or trait queries -- i.e., figures
/// out what sets of types implement which traits. Also, between
/// queries, this struct stores the cached state from previous solver
/// attempts, which can then be re-used later.
pub struct Solver<I: Interner>(SolverImpl<I>);

enum SolverImpl<I: Interner> {
    #[cfg(feature = "slg-solver")]
    Slg {
        forest: Box<Forest<I, SlgContext<I>>>,
        max_size: usize,
        /// The expected number of answers for a solution.
        /// Only really useful for tests, since `make_solution`
        /// will panic if the number of cached answers does not
        /// equal this when a solution is made.
        expected_answers: Option<usize>,
    },
    #[cfg(feature = "recursive-solver")]
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
            #[cfg(feature = "slg-solver")]
            SolverImpl::Slg {
                forest,
                max_size,
                expected_answers,
            } => {
                let ops = SlgContextOps::new(program, *max_size, *expected_answers);
                ops.make_solution(goal, forest.iter_answers(&ops, goal), || true)
            }
            #[cfg(feature = "recursive-solver")]
            SolverImpl::Recursive(ctx) => {
                ctx.solver(program)
                    .solve_root_goal(goal)
                    .ok()
                    .map(|s| match s {
                        crate::recursive::lib::Solution::Unique(c) => {
                            crate::solve::Solution::Unique(c)
                        }
                        crate::recursive::lib::Solution::Ambig(g) => {
                            crate::solve::Solution::Ambig(match g {
                                crate::recursive::lib::Guidance::Definite(g) => {
                                    crate::solve::Guidance::Definite(g)
                                }
                                crate::recursive::lib::Guidance::Suggested(g) => {
                                    crate::solve::Guidance::Suggested(g)
                                }
                                crate::recursive::lib::Guidance::Unknown => {
                                    crate::solve::Guidance::Unknown
                                }
                            })
                        }
                    })
            }
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
            #[cfg(feature = "slg-solver")]
            SolverImpl::Slg {
                forest,
                max_size,
                expected_answers,
            } => {
                let ops = SlgContextOps::new(program, *max_size, *expected_answers);
                ops.make_solution(goal, forest.iter_answers(&ops, goal), should_continue)
            }
            #[cfg(feature = "recursive-solver")]
            SolverImpl::Recursive(ctx) => {
                // TODO support should_continue in recursive solver
                ctx.solver(program)
                    .solve_root_goal(goal)
                    .ok()
                    .map(|s| match s {
                        crate::recursive::lib::Solution::Unique(c) => {
                            crate::solve::Solution::Unique(c)
                        }
                        crate::recursive::lib::Solution::Ambig(g) => {
                            crate::solve::Solution::Ambig(match g {
                                crate::recursive::lib::Guidance::Definite(g) => {
                                    crate::solve::Guidance::Definite(g)
                                }
                                crate::recursive::lib::Guidance::Suggested(g) => {
                                    crate::solve::Guidance::Suggested(g)
                                }
                                crate::recursive::lib::Guidance::Unknown => {
                                    crate::solve::Guidance::Unknown
                                }
                            })
                        }
                    })
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
    #[cfg(feature = "slg-solver")]
    pub fn solve_multiple(
        &mut self,
        program: &dyn RustIrDatabase<I>,
        goal: &UCanonical<InEnvironment<Goal<I>>>,
        mut f: impl FnMut(SubstitutionResult<Canonical<ConstrainedSubst<I>>>, bool) -> bool,
    ) -> bool {
        match &mut self.0 {
            SolverImpl::Slg {
                forest,
                max_size,
                expected_answers,
            } => {
                let ops = SlgContextOps::new(program, *max_size, *expected_answers);
                let mut answers = forest.iter_answers(&ops, goal);
                loop {
                    let subst = match answers.next_answer(|| true) {
                        AnswerResult::Answer(answer) => {
                            if !answer.ambiguous {
                                SubstitutionResult::Definite(answer.subst)
                            } else {
                                if ops.is_trivial_constrained_substitution(&answer.subst) {
                                    SubstitutionResult::Floundered
                                } else {
                                    SubstitutionResult::Ambiguous(answer.subst)
                                }
                            }
                        }
                        AnswerResult::Floundered => SubstitutionResult::Floundered,
                        AnswerResult::NoMoreSolutions => {
                            return true;
                        }
                        AnswerResult::QuantumExceeded => continue,
                    };

                    if !f(subst, !answers.peek_answer(|| true).is_no_more_solutions()) {
                        return false;
                    }
                }
            }
            #[cfg(feature = "recursive-solver")]
            SolverImpl::Recursive(_ctx) => unimplemented!(),
        }
    }
}

impl<I: Interner> std::fmt::Debug for Solver<I> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "Solver {{ .. }}")
    }
}
