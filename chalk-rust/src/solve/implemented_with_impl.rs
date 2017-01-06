use errors::*;
use ir::*;
use solve::Solution;
use solve::environment::{Environment, InEnvironment};
use solve::infer::InferenceTable;
use solve::solver::Solver;
use std::sync::Arc;

pub struct ImplementedWithImpl<'s> {
    solver: &'s mut Solver,
    infer: InferenceTable,
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
        ImplementedWithImpl {
            solver: solver,
            environment: environment,
            infer: infer,
            goal: goal,
            impl_id: impl_id,
        }
    }

    pub fn solve(&mut self) -> Result<Solution<Quantified<InEnvironment<TraitRef>>>> {
        let environment = self.environment.clone();
        let program = self.solver.program.clone();

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

            self.infer.instantiate(environment.universe,
                                   &(&impl_data.trait_ref, &impl_data.where_clauses))
        };

        // Unify the trait-ref we are looking for (`self.goal`) with
        // the trait-ref that the impl supplies (if we can). This will
        // result in some auxiliary normalization clauses we must
        // prove.
        let normalize_to = self.infer.unify(&self.goal, &impl_trait_ref)?;
        debug!("implemented_with::solve: normalize_to={:?}", normalize_to);

        // Combine the where-clauses from the impl with the results
        // from unification into one master list of things to solve,
        // pairing each with the environment.
        let env_where_clauses: Vec<_> =
            where_clauses.into_iter()
                         .chain(normalize_to.into_iter().map(WhereClause::Normalize))
                         .map(|wc| InEnvironment::new(&environment, wc))
                         .collect();

        // Now try to prove the where-clauses one by one. If all of
        // them can be successfully proved, then we know that this
        // impl applies. If any of them error out, this impl does not.
        let successful = self.solver.solve_all(&mut self.infer, env_where_clauses)?;
        let refined_goal = self.infer.quantify(&InEnvironment::new(&environment, &self.goal));
        Ok(Solution {
            successful: successful,
            refined_goal: refined_goal,
        })
    }
}
