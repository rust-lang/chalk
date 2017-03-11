use errors::*;
use fold::Fold;
use ir::*;
use solve::infer::InferenceTable;
use solve::fulfill::Fulfill;
use solve::solver::Solver;
use solve::Solution;
use std::fmt::Debug;
use std::sync::Arc;
use zip::Zip;

pub struct SolveUnify<'s, T>
    where T: Zip + Debug + Fold<Result = T>
{
    fulfill: Fulfill<'s>,
    environment: Arc<Environment>,
    goal: Unify<T>,
}

impl<'s, T> SolveUnify<'s, T>
    where T: Zip + Debug + Fold<Result = T>
{
    pub fn new(solver: &'s mut Solver, env_goal: Query<InEnvironment<Unify<T>>>) -> Self {
        let Query { binders, value: InEnvironment { environment, goal } } = env_goal;
        let infer = InferenceTable::new_with_vars(&binders);
        let fulfill = Fulfill::new(solver, infer);
        SolveUnify { fulfill, environment, goal }
    }

    pub fn solve(mut self) -> Result<Solution<InEnvironment<Unify<T>>>> {
        let environment = self.environment.clone();
        self.fulfill.unify(&environment, &self.goal.a, &self.goal.b)?;
        let successful = self.fulfill.solve_all()?;
        let refined_goal = self.fulfill.refine_goal(InEnvironment::new(&environment, &self.goal));
        Ok(Solution {
            successful: successful,
            refined_goal: refined_goal,
        })
    }
}
