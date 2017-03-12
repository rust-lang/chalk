use cast::Cast;
use errors::*;
use fold::Fold;
use ir::*;
use solve::match_clause::MatchClause;
use solve::match_program_clause::MatchProgramClause;
use solve::solver::Solver;
use solve::Solution;
use std::hash::Hash;

pub struct MatchAny<'s, G: 's> {
    solver: &'s mut Solver,
    env_goal: &'s Query<InEnvironment<G>>,
}

enum Technique<'t> {
    WithClause(WhereClause),
    WithProgramClause(&'t ProgramClause),
}

impl<'s, G> MatchAny<'s, G>
    where G: Cast<WhereClause> + Cast<WhereClauseGoal> + Clone + Hash + Eq + Fold<Result = G>
{
    pub fn new(solver: &'s mut Solver, env_goal: &'s Query<InEnvironment<G>>) -> Self {
        MatchAny {
            solver: solver,
            env_goal: env_goal,
        }
    }

    pub fn solve(self) -> Result<Solution<InEnvironment<G>>> {
        let MatchAny { solver, env_goal } = self;
        let program = solver.program.clone();

        let environment = &env_goal.value.environment;
        let techniques =
            environment.elaborated_clauses(&program)
                       .map(Technique::WithClause)
                       .chain(program.program_clauses.iter().map(Technique::WithProgramClause));

        let result = solver.solve_any(techniques, &env_goal, |solver, technique| {
            match technique {
                Technique::WithClause(clause) => {
                    MatchClause::new(solver, &env_goal, &clause).solve()
                }
                Technique::WithProgramClause(program_clause) => {
                    MatchProgramClause::new(solver, &env_goal, program_clause).solve()
                }
            }
        });

        result
    }
}
