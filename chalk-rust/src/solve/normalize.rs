use errors::*;
use ir::*;
use solve::environment::InEnvironment;
use solve::match_any::MatchAny;
use solve::normalize_application::NormalizeApplication;
use solve::solver::Solver;
use solve::Solution;

pub struct SolveNormalize<'s> {
    solver: &'s mut Solver,
    env_goal: Query<InEnvironment<Normalize>>,
}

enum Technique {
    WithClause(WhereClause),
    WithImpl(ItemId),
}

impl<'s> SolveNormalize<'s> {
    pub fn new(solver: &'s mut Solver, env_goal: Query<InEnvironment<Normalize>>) -> Self {
        SolveNormalize {
            solver: solver,
            env_goal: env_goal,
        }
    }

    pub fn solve(self) -> Result<Solution<InEnvironment<Normalize>>> {
        let SolveNormalize { solver, env_goal } = self;
        match MatchAny::new(solver, &env_goal).solve() {
            Ok(v) => Ok(v),
            Err(_) => {
                // If we can't find anything better, the fallback is to
                // normalize into an application of `Iterator::Item`.
                NormalizeApplication::new(solver, env_goal).solve()
            }
        }
    }
}
