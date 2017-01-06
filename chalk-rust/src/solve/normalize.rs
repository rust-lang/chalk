use errors::*;
use ir::*;
use solve::environment::InEnvironment;
use solve::match_clause::MatchClause;
use solve::normalize_application::NormalizeApplication;
use solve::normalize_with_impl::NormalizeWithImpl;
use solve::solver::Solver;
use solve::Solution;

pub struct SolveNormalize<'s> {
    solver: &'s mut Solver,
    env_goal: Quantified<InEnvironment<Normalize>>,
}

impl<'s> SolveNormalize<'s> {
    pub fn new(solver: &'s mut Solver, env_goal: Quantified<InEnvironment<Normalize>>) -> Self {
        SolveNormalize {
            solver: solver,
            env_goal: env_goal,
        }
    }

    pub fn solve(self) -> Result<Solution<Quantified<InEnvironment<Normalize>>>> {
        let SolveNormalize { solver, env_goal } = self;
        let program = solver.program.clone();

        // First try to find a solution in the environment.
        let environment = env_goal.value.environment.clone();
        let num_clauses = environment.clauses.len();
        let env_result = solver.solve_any(0..num_clauses, &env_goal, |solver, clause_index| {
            MatchClause::new(solver, &env_goal, clause_index).solve()
        });

        // If we found something in the environment, even if it might
        // have caused us to infer things that are not necessarily
        // correct, then take it. This is not obviously the right
        // thing to do but it models rustc's current behavior.
        if let Ok(v) = env_result {
            return Ok(v);
        }

        // Nothing in the environment, so try impls.
        let impl_result = {
            solver.solve_any(&program.impl_data, &env_goal, |solver, (&impl_id, _impl_data)| {
                NormalizeWithImpl::new(solver, env_goal.clone(), impl_id).solve()
            })
        };
        if let Ok(v) = impl_result {
            return Ok(v);
        }

        // If we can't find anything better, the fallback is to
        // normalize into an application of `Iterator::Item`.
        NormalizeApplication::new(solver, env_goal).solve()
    }
}
