use errors::*;
use ir::*;
use solve::Solution;
use solve::environment::{Environment, InEnvironment};
use solve::infer::InferenceTable;
use solve::solver::Solver;
use std::sync::Arc;

pub struct ImplementWithClause<'s> {
    solver: &'s mut Solver,
    infer: InferenceTable,
    environment: Arc<Environment>,
    goal: TraitRef,
    clause_index: usize,
}

impl<'s> ImplementWithClause<'s> {
    pub fn new(solver: &'s mut Solver,
               q: Quantified<InEnvironment<TraitRef>>,
               clause_index: usize)
               -> Self {
        let InEnvironment { environment, goal } = q.value;
        let infer = InferenceTable::new_with_vars(q.binders, environment.universe);
        assert!(clause_index < environment.clauses.len());
        ImplementWithClause { solver, infer, environment, goal, clause_index }
    }

    pub fn solve(&mut self) -> Result<Solution<Quantified<InEnvironment<TraitRef>>>> {
        let environment = self.environment.clone();
        let clause = &environment.clauses[self.clause_index];

        match *clause {
            WhereClause::Implemented(ref trait_ref) => {
                let normalize_to = self.infer.unify(&self.goal, &trait_ref)?;
                let env_where_clauses: Vec<_> =
                    normalize_to.into_iter()
                                .map(WhereClause::NormalizeTo)
                                .map(|wc| InEnvironment::new(&environment, wc))
                                .collect();
                let successful = self.solver.solve_all(&mut self.infer, env_where_clauses)?;
                let refined_goal = self.infer.quantify(&InEnvironment::new(&environment, &self.goal));
                Ok(Solution {
                    successful: successful,
                    refined_goal: refined_goal,
                })
            }

            _ => {
                bail!("clause `{:?}` not relevant to goal `{:?}`", clause, self.goal)
            }
        }
    }
}
