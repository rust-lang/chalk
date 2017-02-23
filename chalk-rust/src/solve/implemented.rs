use errors::*;
use ir::*;
use solve::environment::InEnvironment;
use solve::match_any::MatchAny;
use solve::solver::Solver;
use solve::Solution;

pub struct Implemented<'s> {
    solver: &'s mut Solver,
    env_goal: Query<InEnvironment<TraitRef>>,
}

enum Technique {
    WithClause(WhereClause),
    WithImpl(ItemId),
}

impl<'s> Implemented<'s> {
    pub fn new(solver: &'s mut Solver, env_goal: Query<InEnvironment<TraitRef>>) -> Self {
        Implemented {
            solver: solver,
            env_goal: env_goal,
        }
    }

    pub fn solve(self) -> Result<Solution<InEnvironment<TraitRef>>> {
        let Implemented { solver, env_goal } = self;
        MatchAny::new(solver, &env_goal).solve().chain_err(|| {
            format!("`{:?}{:?}` is not implemented for `{:?}` in environment `{:?}`",
                    env_goal.value.goal.trait_id,
                    debug::Angle(&env_goal.value.goal.parameters[1..]),
                    &env_goal.value.goal.parameters[0],
                    env_goal.value.environment)
        })
    }
}
