use errors::*;
use ir::*;
use solve::match_any::MatchAny;
use solve::solver::Solver;
use solve::Solution;

pub struct SolveWellFormed<'s> {
    solver: &'s mut Solver,
    env_goal: Query<InEnvironment<WellFormed>>,
}

impl<'s> SolveWellFormed<'s> {
    pub fn new(solver: &'s mut Solver, env_goal: Query<InEnvironment<WellFormed>>) -> Self {
        SolveWellFormed {
            solver: solver,
            env_goal: env_goal,
        }
    }

    pub fn solve(self) -> Result<Solution<InEnvironment<WellFormed>>> {
        let SolveWellFormed { solver, env_goal } = self;
        MatchAny::new(solver, &env_goal).solve()
    }
}
