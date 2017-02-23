use errors::*;
use ir::*;
use solve::Solution;
use solve::environment::{Environment, InEnvironment};
use solve::fulfill::Fulfill;
use solve::infer::InferenceTable;
use solve::solver::Solver;
use std::sync::Arc;

pub struct NormalizeApplication<'s> {
    fulfill: Fulfill<'s>,
    environment: Arc<Environment>,
    goal: Normalize,
}

impl<'s> NormalizeApplication<'s> {
    pub fn new(solver: &'s mut Solver,
               q: Query<InEnvironment<Normalize>>)
               -> Self {
        let InEnvironment { environment, goal } = q.value;
        let infer = InferenceTable::new_with_vars(&q.binders);
        let fulfill = Fulfill::new(solver, infer);
        NormalizeApplication { fulfill, environment, goal }
    }

    pub fn solve(mut self) -> Result<Solution<InEnvironment<Normalize>>> {
        let environment = self.environment.clone();

        // Construct an application from the projection. So if we have
        // `<T as Iterator>::Item`, we would produce
        // `(Iterator::Item)<T>`.
        let apply_ty = {
            let ProjectionTy { associated_ty_id, ref parameters } = self.goal.projection;
            Ty::Apply(ApplicationTy {
                name: TypeName::AssociatedType(associated_ty_id),
                parameters: parameters.clone()
            })
        };

        // Unify the result of normalization (`self.goal.ty`) with the
        // application type we just built (`apply_ty`).
        self.fulfill.unify(&environment, &self.goal.ty, &apply_ty)?;

        // Now try to prove any resulting where-clauses one by one. If
        // all of them can be successfully proved, then we know that
        // this unification succeeded.
        let successful = self.fulfill.solve_all()?;
        let refined_goal = self.fulfill.refine_goal(InEnvironment::new(&environment, &self.goal));
        Ok(Solution {
            successful: successful,
            refined_goal: refined_goal,
        })
    }
}
