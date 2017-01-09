use errors::*;
use ir::*;
use solve::Solution;
use solve::environment::{Environment, InEnvironment};
use solve::fulfill::Fulfill;
use solve::infer::InferenceTable;
use solve::solver::Solver;
use std::sync::Arc;

pub struct ImplementedWithImpl<'s> {
    fulfill: Fulfill<'s>,
    environment: Arc<Environment>,
    goal: TraitRef,
    impl_id: ItemId,
}

impl<'s> ImplementedWithImpl<'s> {
    pub fn new(solver: &'s mut Solver,
               q: Quantified<InEnvironment<TraitRef>>,
               impl_id: ItemId)
               -> Self {
        let InEnvironment { environment, goal } = q.value;
        let infer = InferenceTable::new_with_vars(&q.binders);
        let fulfill = Fulfill::new(solver, infer);
        ImplementedWithImpl { fulfill, environment, goal, impl_id }
    }

    pub fn solve(mut self) -> Result<Solution<InEnvironment<TraitRef>>> {
        let environment = self.environment.clone();
        let program = self.fulfill.program();

        // Extract the trait-ref that this impl implements and its where-clauses,
        // instantiating all the impl parameters with fresh variables.
        //
        // So, assuming `?1` is the next new variable in `self.infer`, if we had:
        //
        //     impl<T: Clone> Clone for Option<T>
        //
        // this would yield `Option<?1>: Clone` and `?1: Clone`.
        let (impl_trait_ref, where_clauses) = {
            let impl_data = &program.impl_data[&self.impl_id];

            // screen out impls that are for the totally wrong trait
            // early, just to keep debug logging under control. This
            // would otherwise fail when we unify the trait-ref below.
            if impl_data.trait_ref.trait_id != self.goal.trait_id {
                bail!("impl for wrong trait");
            }

            self.fulfill.instantiate(
                impl_data.parameter_kinds.iter().map(|k| k.as_ref().map(|_| environment.universe)),
                &(&impl_data.trait_ref, &impl_data.where_clauses))
        };

        // Unify the trait-ref we are looking for (`self.goal`) with
        // the trait-ref that the impl supplies (if we can).
        self.fulfill.unify(&environment, &self.goal, &impl_trait_ref)?;

        // Add the where-clauses from the impl into the fulfillment list.
        self.fulfill.extend(
            where_clauses.into_iter()
                         .map(|wc| InEnvironment::new(&environment, wc)));

        // Now try to prove the where-clauses one by one. If all of
        // them can be successfully proved, then we know that this
        // impl applies. If any of them error out, this impl does not.
        let successful = self.fulfill.solve_all()?;
        let refined_goal = self.fulfill.constrained(InEnvironment::new(&environment, &self.goal));
        let refined_goal = self.fulfill.quantify(&refined_goal);
        Ok(Solution {
            successful: successful,
            refined_goal: refined_goal,
        })
    }
}
