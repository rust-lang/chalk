use cast::Cast;
use errors::*;
use fold::Fold;
use ir::*;
use solve::Solution;
use solve::environment::{Environment, InEnvironment};
use solve::fulfill::Fulfill;
use solve::infer::InferenceTable;
use solve::solver::Solver;
use std::sync::Arc;

pub struct MatchProgramClause<'s, G: 's> {
    fulfill: Fulfill<'s>,
    environment: Arc<Environment>,
    goal: &'s G,
    program_clause_data: ProgramClauseImplication,
}

impl<'s, G> MatchProgramClause<'s, G>
    where G: Clone + Cast<WhereClauseGoal> + Fold<Result = G>
{
    pub fn new(solver: &'s mut Solver,
               q: &'s Quantified<InEnvironment<G>>,
               program_clause: &'s ProgramClause)
               -> Self {
        let InEnvironment { ref environment, ref goal } = q.value;
        let infer = InferenceTable::new_with_vars(&q.binders);
        let environment = environment.clone();
        let mut fulfill = Fulfill::new(solver, infer);

        let program_clause_data =
            fulfill.instantiate_in(environment.universe,
                                   program_clause.implication.binders.iter().cloned(),
                                   &program_clause.implication.value);

        MatchProgramClause { fulfill, environment, goal, program_clause_data }
    }

    pub fn solve(mut self) -> Result<Solution<InEnvironment<G>>> {
        let environment = self.environment.clone();
        self.fulfill.unify(&environment,
                           &self.goal.clone().cast(),
                           &self.program_clause_data.consequence)?;
        for condition in self.program_clause_data.conditions {
            self.fulfill.push_goal(condition, &environment);
        }
        let successful = self.fulfill.solve_all()?;
        let refined_goal = self.fulfill.refine_goal(InEnvironment::new(&environment, &self.goal));
        Ok(Solution {
            successful: successful,
            refined_goal: refined_goal,
        })
    }
}
