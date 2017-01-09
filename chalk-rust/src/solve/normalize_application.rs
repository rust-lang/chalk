use errors::*;
use ir::*;
use solve::Solution;
use solve::environment::{Environment, InEnvironment};
use solve::infer::{InferenceTable, UnificationResult};
use solve::solver::Solver;
use std::sync::Arc;

pub struct NormalizeApplication<'s> {
    solver: &'s mut Solver,
    infer: InferenceTable,
    environment: Arc<Environment>,
    goal: Normalize,
}

impl<'s> NormalizeApplication<'s> {
    pub fn new(solver: &'s mut Solver,
               q: Quantified<InEnvironment<Normalize>>)
               -> Self {
        let InEnvironment { environment, goal } = q.value;
        let infer = InferenceTable::new_with_vars(&q.binders);
        NormalizeApplication {
            solver: solver,
            environment: environment,
            infer: infer,
            goal: goal,
        }
    }

    pub fn solve(mut self) -> Result<Solution<InEnvironment<Normalize>>> {
        let environment = self.environment.clone();

        // Construct an application from the projection. So if we have
        // `<T as Iterator>::Item`, we would produce
        // `(Iterator::Item)<T>`.
        let apply_ty = {
            let name = TypeName::AssociatedType(AssociatedType {
                trait_id: self.goal.projection.trait_ref.trait_id,
                name: self.goal.projection.name,
            });
            let parameters = self.goal.projection.trait_ref.parameters.clone();
            Ty::Apply(ApplicationTy { name, parameters })
        };

        // Unify the result of normalization (`self.goal.ty`) with the
        // application type we just built (`apply_ty`).
        let UnificationResult { normalizations: normalize_to1 } =
            self.infer.unify(&self.goal.ty, &apply_ty)?;

        debug!("implemented_with::solve: normalize_to1={:?}", normalize_to1);

        // Now try to prove the where-clauses one by one. If all of
        // them can be successfully proved, then we know that this
        // unification succeeded.
        let env_where_clauses: Vec<_> =
            normalize_to1.into_iter().map(WhereClause::Normalize)
                         .map(|wc| InEnvironment::new(&environment, wc))
                         .collect();
        let successful = self.solver.solve_all(&mut self.infer, env_where_clauses)?;
        let refined_goal = self.infer.constrained(InEnvironment::new(&environment, &self.goal));
        let refined_goal = self.infer.quantify(&refined_goal);
        Ok(Solution {
            successful: successful,
            refined_goal: refined_goal,
        })
    }
}
