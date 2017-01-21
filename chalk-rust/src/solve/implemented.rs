use errors::*;
use ir::*;
use solve::environment::InEnvironment;
use solve::implemented_with_impl::ImplementedWithImpl;
use solve::match_clause::MatchClause;
use solve::solver::Solver;
use solve::Solution;

pub struct Implemented<'s> {
    solver: &'s mut Solver,
    env_goal: Quantified<InEnvironment<TraitRef>>,
}

enum Technique {
    WithClause(WhereClause),
    WithImpl(ItemId),
}

impl<'s> Implemented<'s> {
    pub fn new(solver: &'s mut Solver, env_goal: Quantified<InEnvironment<TraitRef>>) -> Self {
        Implemented {
            solver: solver,
            env_goal: env_goal,
        }
    }

    pub fn solve(self) -> Result<Solution<InEnvironment<TraitRef>>> {
        let Implemented { solver, env_goal } = self;
        let program = solver.program.clone();

        let environment = &env_goal.value.environment;
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
                    ImplementedWithImpl::new(solver, env_goal.clone(), impl_id).solve()
                }
            }
        });

        result.chain_err(|| {
            format!("`{:?}{:?}` is not implemented for `{:?}` in environment `{:?}`",
                    env_goal.value.goal.trait_id,
                    debug::Angle(&env_goal.value.goal.parameters[1..]),
                    &env_goal.value.goal.parameters[0],
                    env_goal.value.environment)
        })
    }
}
