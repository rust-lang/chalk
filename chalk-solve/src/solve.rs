use crate::solve::slg::SlgContext;
use chalk_engine::forest::Forest;
use chalk_ir::DomainGoal;
use chalk_ir::IsCoinductive;
use chalk_ir::ProgramClause;
use chalk_ir::*;
use std::fmt;
use std::fmt::Debug;

mod slg;
mod truncate;

#[derive(Clone, Debug, PartialEq, Eq)]
/// A (possible) solution for a proposed goal.
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
    /// Returns the default SLG parameters.
    fn slg() -> Self {
        SolverChoice::SLG { max_size: 10 }
    }

    /// Creates a solver state.
    pub fn into_solver(self) -> Solver {
        match self {
            SolverChoice::SLG { max_size } => Solver {
                forest: Forest::new(SlgContext::new(max_size)),
            },
        }
    }
}

impl Default for SolverChoice {
    fn default() -> Self {
        SolverChoice::slg()
    }
}

/// Finds the solution to "goals", or trait queries -- i.e., figures
/// out what sets of types implement which traits. Also, between
/// queries, this struct stores the cached state from previous solver
/// attempts, which can then be re-used later.
pub struct Solver {
    forest: Forest<SlgContext>,
}

impl Solver {
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
        program: &dyn ProgramClauseSet,
        goal: &UCanonical<InEnvironment<Goal>>,
    ) -> Option<Solution> {
        let ops = self.forest.context().ops(program);
        self.forest.solve(&ops, goal)
    }

    pub fn into_test(self) -> TestSolver {
        TestSolver { state: self }
    }
}

/// Wrapper around a `Solver` that exposes
/// additional methods meant only for testing.
pub struct TestSolver {
    state: Solver,
}

impl std::ops::Deref for TestSolver {
    type Target = Solver;

    fn deref(&self) -> &Solver {
        &self.state
    }
}

impl std::ops::DerefMut for TestSolver {
    fn deref_mut(&mut self) -> &mut Solver {
        &mut self.state
    }
}

impl TestSolver {
    /// Force the first `num_answers` answers. Meant only for testing,
    /// and hence the precise return type is obscured (but you can get
    /// its debug representation).
    pub fn force_answers(
        &mut self,
        program: &dyn ProgramClauseSet,
        goal: &UCanonical<InEnvironment<Goal>>,
        num_answers: usize,
    ) -> Box<std::fmt::Debug> {
        let ops = self.forest.context().ops(program);
        Box::new(self.forest.force_answers(&ops, goal.clone(), num_answers))
    }

    /// Returns then number of cached answers for `goal`. Used only in
    /// testing.
    pub fn num_cached_answers_for_goal(
        &mut self,
        program: &dyn ProgramClauseSet,
        goal: &UCanonical<InEnvironment<Goal>>,
    ) -> usize {
        let ops = self.forest.context().ops(program);
        self.forest.num_cached_answers_for_goal(&ops, goal)
    }
}

/// The trait for defining the program clauses that are in scope when
/// solving a goal.
pub trait ProgramClauseSet: Debug + IsCoinductive {
    /// Returns a set of program clauses that could possibly match
    /// `goal`. This can be any superset of the correct set, but the
    /// more precise you can make it, the more efficient solving will
    /// be.
    fn program_clauses_that_could_match(&self, goal: &DomainGoal, vec: &mut Vec<ProgramClause>);

    /// Converts a `dyn ProgramClauseSet` into a `dyn
    /// IsCoinductive`. This is a workaround for the fact that rust
    /// doesn't present permit such an upcast automatically.
    fn upcast(&self) -> &dyn IsCoinductive;
}
