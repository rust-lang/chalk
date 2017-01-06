use cast::Cast;
use errors::*;
use fold::Fold;
use ir::*;
use solve::Solution;
use solve::environment::{Environment, InEnvironment};
use solve::infer::InferenceTable;
use solve::solver::Solver;
use std::sync::Arc;

pub struct MatchClause<'s, G: 's> {
    solver: &'s mut Solver,
    infer: InferenceTable,
    environment: Arc<Environment>,
    goal: &'s G,
    clause_index: usize,
}

impl<'s, G> MatchClause<'s, G>
    where G: Clone + Cast<WhereClause> + Fold<Result = G>
{
    pub fn new(solver: &'s mut Solver,
               q: &'s Quantified<InEnvironment<G>>,
               clause_index: usize)
               -> Self {
        let InEnvironment { ref environment, ref goal } = q.value;
        let infer = InferenceTable::new_with_vars(&q.binders);
        assert!(clause_index < environment.clauses.len());
        let environment = environment.clone();
        MatchClause { solver, infer, environment, goal, clause_index }
    }

    pub fn solve(mut self) -> Result<Solution<Quantified<InEnvironment<G>>>> {
        let environment = self.environment.clone();
        let clause = &environment.clauses[self.clause_index];
        let normalize_to = self.infer.unify(&self.goal.clone().cast(), &clause)?;
        let env_where_clauses: Vec<_> =
            normalize_to.into_iter()
                        .map(WhereClause::Normalize)
                        .map(|wc| InEnvironment::new(&environment, wc))
                        .collect();
        let successful = self.solver.solve_all(&mut self.infer, env_where_clauses)?;
        let refined_goal = self.infer.quantify(&InEnvironment::new(&environment, self.goal));
        Ok(Solution {
            successful: successful,
            refined_goal: refined_goal,
        })
    }
}
