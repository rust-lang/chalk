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

enum Technique {
    WithClause(WhereClause),
    WithImpl(ItemId),
}

impl<'s> SolveNormalize<'s> {
    pub fn new(solver: &'s mut Solver, env_goal: Quantified<InEnvironment<Normalize>>) -> Self {
        SolveNormalize {
            solver: solver,
            env_goal: env_goal,
        }
    }

    pub fn solve(self) -> Result<Solution<InEnvironment<Normalize>>> {
        let SolveNormalize { solver, env_goal } = self;
        let program = solver.program.clone();

        // First try to find a solution in the environment.
        let environment = env_goal.value.environment.clone();
        let techniques =
            environment.elaborated_clauses(&program)
                       .map(Technique::WithClause)
                       .chain(program.impl_data.keys().map(|&impl_id| Technique::WithImpl(impl_id)));
        let result = solver.solve_any(techniques, &env_goal, |solver, technique| {
            match technique {
                Technique::WithClause(clause) => {
                    MatchClause::new(solver, &env_goal, &clause).solve()
                }
                Technique::WithImpl(impl_id) => {
                    NormalizeWithImpl::new(solver, env_goal.clone(), impl_id).solve()
                }
            }
        });
        if let Ok(v) = result {
            return Ok(v);
        }

        // If we can't find anything better, the fallback is to
        // normalize into an application of `Iterator::Item`.
        NormalizeApplication::new(solver, env_goal).solve()
    }
}
