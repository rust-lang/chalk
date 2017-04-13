use errors::*;
use ir::*;
use solve::fulfill::Fulfill;
use solve::solver::Solver;
use solve::Solution;

pub struct Prove<'s> {
    fulfill: Fulfill<'s>,
    goals: Vec<InEnvironment<WhereClauseGoal>>,
}

impl<'s> Prove<'s> {
    pub fn new(solver: &'s mut Solver, goal: Box<Goal>) -> Self {
        let mut prove = Prove {
            fulfill: Fulfill::new(solver),
            goals: vec![],
        };
        let environment = &Environment::new();
        prove.fulfill.push_goal(*goal, environment);
        prove.goals.extend(prove.fulfill.pending_obligations().iter().cloned());
        prove
    }

    pub fn solve(mut self) -> Result<Solution<Vec<WhereClauseGoal>>> {
        let successful = self.fulfill.solve_all()?;
        let goals: Vec<_> = self.goals.into_iter().map(|g| g.goal).collect();
        let refined_goal = self.fulfill.refine_goal(goals);
        Ok(Solution {
            successful: successful,
            refined_goal: refined_goal,
        })
    }
}
