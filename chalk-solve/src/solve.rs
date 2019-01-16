use crate::solve::slg::implementation::SlgContext;
use chalk_engine::context::Context;
use chalk_engine::context::ContextOps;
use chalk_engine::fallible::*;
use chalk_engine::forest::Forest;
use chalk_ir::*;
use std::fmt;
use std::sync::Arc;

pub mod slg;
mod truncate;

#[derive(Clone, Debug, PartialEq, Eq)]
/// A (possible) solution for a proposed goal. Usually packaged in a `Result`,
/// where `Err` represents definite *failure* to prove a goal.
pub enum Solution {
    /// The goal indeed holds, and there is a unique value for all existential
    /// variables. In this case, we also record a set of lifetime constraints
    /// which must also hold for the goal to be valid.
    Unique(Canonical<ConstrainedSubst>),

    /// The goal may be provable in multiple ways, but regardless we may have some guidance
    /// for type inference. In this case, we don't return any lifetime
    /// constraints, since we have not "committed" to any particular solution
    /// yet.
    Ambig(Guidance),
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// When a goal holds ambiguously (e.g., because there are multiple possible
/// solutions), we issue a set of *guidance* back to type inference.
pub enum Guidance {
    /// The existential variables *must* have the given values if the goal is
    /// ever to hold, but that alone isn't enough to guarantee the goal will
    /// actually hold.
    Definite(Canonical<Substitution>),

    /// There are multiple plausible values for the existentials, but the ones
    /// here are suggested as the preferred choice heuristically. These should
    /// be used for inference fallback only.
    Suggested(Canonical<Substitution>),

    /// There's no useful information to feed back to type inference
    Unknown,
}

impl Solution {
    pub fn is_unique(&self) -> bool {
        match *self {
            Solution::Unique(..) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Solution {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Solution::Unique(constrained) => write!(f, "Unique; {}", constrained,),
            Solution::Ambig(Guidance::Definite(subst)) => {
                write!(f, "Ambiguous; definite substitution {}", subst)
            }
            Solution::Ambig(Guidance::Suggested(subst)) => {
                write!(f, "Ambiguous; suggested substitution {}", subst)
            }
            Solution::Ambig(Guidance::Unknown) => write!(f, "Ambiguous; no inference guidance"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum SolverChoice {
    /// Run the SLG solver, producing a Solution.
    SLG { max_size: usize },
}

impl SolverChoice {
    /// Attempts to solve the given root goal, which must be in
    /// canonical form. The solution is searching for unique answers
    /// to any free existential variables in this goal.
    ///
    /// # Returns
    ///
    /// - `Ok(None)` is the goal cannot be proven.
    /// - `Ok(Some(solution))` if we succeeded in finding *some* answers,
    ///   although `solution` may reflect ambiguity and unknowns.
    /// - `Err` if there was an internal error solving the goal, which does not
    ///   reflect success nor failure.
    pub fn solve_root_goal(
        self,
        env: &Arc<ProgramEnvironment>,
        canonical_goal: &UCanonical<InEnvironment<Goal>>,
    ) -> Fallible<Option<Solution>> {
        Ok(self.create_solver(env).solve(canonical_goal))
    }

    /// Returns the default SLG parameters.
    fn slg() -> Self {
        SolverChoice::SLG { max_size: 10 }
    }

    pub fn create_solver(self, env: &Arc<ProgramEnvironment>) -> Box<Solver> {
        match self {
            SolverChoice::SLG { max_size } => Box::new(Forest::new(SlgContext::new(env, max_size))),
        }
    }
}

impl Default for SolverChoice {
    fn default() -> Self {
        SolverChoice::slg()
    }
}

pub trait Solver {
    /// Solves a given goal, producing the solution. This will do only
    /// as much work towards `goal` as it has to (and that works is
    /// cached for future attempts).
    fn solve(&mut self, goal: &UCanonical<InEnvironment<Goal>>) -> Option<Solution>;
}

impl<C, CO> Solver for Forest<C, CO>
where
    C: Context<UCanonicalGoalInEnvironment = UCanonical<InEnvironment<Goal>>, Solution = Solution>,
    CO: ContextOps<C>,
{
    fn solve(&mut self, goal: &UCanonical<InEnvironment<Goal>>) -> Option<Solution> {
        self.solve(goal)
    }
}
