use crate::RustIrDatabase;
use chalk_derive::HasInterner;
use chalk_ir::interner::Interner;
use chalk_ir::*;
use std::fmt;
use tracing::debug;

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
    /// There are multiple candidate solutions, which may or may not agree on
    /// the values for existential variables; attempt to combine them. This
    /// operation does not depend on the order of its arguments.
    ///
    /// This actually isn't as precise as it could be, in two ways:
    ///
    /// a. It might be that while there are multiple distinct candidates, they
    ///    all agree about *some things*. To be maximally precise, we would
    ///    compute the intersection of what they agree on. It's not clear though
    ///    that this is actually what we want Rust's inference to do, and it's
    ///    certainly not what it does today.
    ///
    /// b. There might also be an ambiguous candidate and a successful candidate,
    ///    both with the same refined-goal. In that case, we could probably claim
    ///    success, since if the conditions of the ambiguous candidate were met,
    ///    we know the success would apply.  Example: `?0: Clone` yields ambiguous
    ///    candidate `Option<?0>: Clone` and successful candidate `Option<?0>:
    ///    Clone`.
    ///
    /// But you get the idea.
    pub fn combine(self, other: Solution<I>, interner: I) -> Solution<I> {
        use self::Guidance::*;

        if self == other {
            return self;
        }

        // Special case hack: if one solution is "true" without any constraints,
        // that is always the combined result.
        //
        // This is not as general as it could be: ideally, if we had one solution
        // that is Unique with a simpler substitution than the other one, or region constraints
        // which are a subset, we'd combine them.
        if self.is_trivial_and_always_true(interner) {
            return self;
        }
        if other.is_trivial_and_always_true(interner) {
            return other;
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

    pub fn is_trivial_and_always_true(&self, interner: I) -> bool {
        match self {
            Solution::Unique(constrained_subst) => {
                constrained_subst.value.subst.is_identity_subst(interner)
                    && constrained_subst.value.constraints.is_empty(interner)
            }
            Solution::Ambig(_) => false,
        }
    }

    /// View this solution purely in terms of type inference guidance
    pub fn into_guidance(self) -> Guidance<I> {
        match self {
            Solution::Unique(constrained) => Guidance::Definite(Canonical {
                value: constrained.value.subst,
                binders: constrained.binders,
            }),
            Solution::Ambig(guidance) => guidance,
        }
    }

    /// Extract a constrained substitution from this solution, even if ambiguous.
    pub fn constrained_subst(&self, interner: I) -> Option<Canonical<ConstrainedSubst<I>>> {
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
    /// hold, and returns the subst in that case.
    pub fn definite_subst(&self, interner: I) -> Option<Canonical<ConstrainedSubst<I>>> {
        match self {
            Solution::Unique(constrained) => Some(constrained.clone()),
            Solution::Ambig(Guidance::Definite(canonical)) => {
                let value = ConstrainedSubst {
                    subst: canonical.value.clone(),
                    constraints: Constraints::empty(interner),
                };
                Some(Canonical {
                    value,
                    binders: canonical.binders.clone(),
                })
            }
            _ => None,
        }
    }

    pub fn is_unique(&self) -> bool {
        matches!(*self, Solution::Unique(..))
    }

    pub fn is_ambig(&self) -> bool {
        matches!(*self, Solution::Ambig(_))
    }

    pub fn display(&self, interner: I) -> SolutionDisplay<'_, I> {
        SolutionDisplay {
            solution: self,
            interner,
        }
    }
}

pub struct SolutionDisplay<'a, I: Interner> {
    solution: &'a Solution<I>,
    interner: I,
}

impl<'a, I: Interner> fmt::Display for SolutionDisplay<'a, I> {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let SolutionDisplay { solution, interner } = self;
        match solution {
            // If a `Unique` solution has no associated data, omit the trailing semicolon.
            // This makes blessed test output nicer to read.
            Solution::Unique(Canonical { binders, value: ConstrainedSubst { subst, constraints } } )
                if interner.constraints_data(constraints.interned()).is_empty()
                    && interner.substitution_data(subst.interned()).is_empty()
                    && interner.canonical_var_kinds_data(binders.interned()).is_empty()
                => write!(f, "Unique"),

            Solution::Unique(constrained) => write!(f, "Unique; {}", constrained.display(*interner)),

            Solution::Ambig(Guidance::Definite(subst)) => write!(
                f,
                "Ambiguous; definite substitution {}",
                subst.display(*interner)
            ),
            Solution::Ambig(Guidance::Suggested(subst)) => write!(
                f,
                "Ambiguous; suggested substitution {}",
                subst.display(*interner)
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
    ///   - second argument is the next solution present
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
