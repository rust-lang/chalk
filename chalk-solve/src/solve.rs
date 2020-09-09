use crate::RustIrDatabase;
use chalk_derive::HasInterner;
use chalk_ir::interner::Interner;
use chalk_ir::*;
use std::fmt;

pub mod truncate;

/// A (possible) solution for a proposed goal.
#[derive(Clone, Debug, PartialEq, Eq, HasInterner)]
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

#[derive(Debug)]
pub enum SubstitutionResult<S> {
    Definite(S),
    Ambiguous(S),
    Floundered,
}

impl<S> SubstitutionResult<S> {
    pub fn as_ref(&self) -> SubstitutionResult<&S> {
        match self {
            SubstitutionResult::Definite(subst) => SubstitutionResult::Definite(subst),
            SubstitutionResult::Ambiguous(subst) => SubstitutionResult::Ambiguous(subst),
            SubstitutionResult::Floundered => SubstitutionResult::Floundered,
        }
    }
    pub fn map<U, F: FnOnce(S) -> U>(self, f: F) -> SubstitutionResult<U> {
        match self {
            SubstitutionResult::Definite(subst) => SubstitutionResult::Definite(f(subst)),
            SubstitutionResult::Ambiguous(subst) => SubstitutionResult::Ambiguous(f(subst)),
            SubstitutionResult::Floundered => SubstitutionResult::Floundered,
        }
    }
}

impl<S: fmt::Display> fmt::Display for SubstitutionResult<S> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubstitutionResult::Definite(subst) => write!(fmt, "{}", subst),
            SubstitutionResult::Ambiguous(subst) => write!(fmt, "Ambiguous({})", subst),
            SubstitutionResult::Floundered => write!(fmt, "Floundered"),
        }
    }
}

/// Finds the solution to "goals", or trait queries -- i.e., figures
/// out what sets of types implement which traits. Also, between
/// queries, this struct stores the cached state from previous solver
/// attempts, which can then be re-used later.
pub trait Solver<I: Interner>
where
    Self: fmt::Debug,
{
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
    fn solve(
        &mut self,
        program: &dyn RustIrDatabase<I>,
        goal: &UCanonical<InEnvironment<Goal<I>>>,
    ) -> Option<Solution<I>>;

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
    fn solve_limited(
        &mut self,
        program: &dyn RustIrDatabase<I>,
        goal: &UCanonical<InEnvironment<Goal<I>>>,
        should_continue: &dyn std::ops::Fn() -> bool,
    ) -> Option<Solution<I>>;

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
    fn solve_multiple(
        &mut self,
        program: &dyn RustIrDatabase<I>,
        goal: &UCanonical<InEnvironment<Goal<I>>>,
        f: &mut dyn FnMut(SubstitutionResult<Canonical<ConstrainedSubst<I>>>, bool) -> bool,
    ) -> bool;

    /// A convenience method for when one doesn't need the actual solution,
    /// only whether or not one exists.
    fn has_unique_solution(
        &mut self,
        program: &dyn RustIrDatabase<I>,
        goal: &UCanonical<InEnvironment<Goal<I>>>,
    ) -> bool {
        match self.solve(program, goal) {
            Some(sol) => sol.is_unique(),
            None => false,
        }
    }
}
