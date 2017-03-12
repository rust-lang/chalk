use cast::Cast;
use errors::*;
use fold::Fold;
use ir::*;
use solve::Solution;
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
               q: &'s Query<InEnvironment<G>>,
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

    pub fn solve(self) -> Result<Solution<InEnvironment<G>>> {
        let MatchProgramClause { mut fulfill, environment, goal, program_clause_data } = self;
        debug_heading!("MatchProgramClause::solve(program_clause_data={:?})",
                       &program_clause_data);
        let environment = environment.clone();
        fulfill.unify(&environment,
                      &goal.clone().cast(),
                      &program_clause_data.consequence)?;
        for condition in program_clause_data.conditions {
            fulfill.push_goal(condition, &environment);
        }
        debug!("unification succeeded, attempting to solve all sub-goals");
        let successful = fulfill.solve_all()?;
        let refined_goal = fulfill.refine_goal(InEnvironment::new(&environment, &goal));
        debug!("success, refined goal = {:?}", refined_goal);
        Ok(Solution {
            successful: successful,
            refined_goal: refined_goal,
        })
    }
}
