use errors::*;
use ir::*;
use solve::Solution;
use solve::environment::{Environment, InEnvironment};
use solve::infer::InferenceTable;
use solve::solver::Solver;
use std::sync::Arc;

pub struct NormalizeWithImpl<'s> {
    solver: &'s mut Solver,
    infer: InferenceTable,
    environment: Arc<Environment>,
    goal: NormalizeTo,
    impl_id: ItemId,
}

impl<'s> NormalizeWithImpl<'s> {
    pub fn new(solver: &'s mut Solver,
               q: Quantified<InEnvironment<NormalizeTo>>,
               impl_id: ItemId)
               -> Self {
        let InEnvironment { environment, goal } = q.value;
        let infer = InferenceTable::new_with_vars(&q.binders);
        NormalizeWithImpl {
            solver: solver,
            environment: environment,
            infer: infer,
            goal: goal,
            impl_id: impl_id,
        }
    }

    pub fn solve(&mut self) -> Result<Solution<Quantified<InEnvironment<NormalizeTo>>>> {
        let environment = self.environment.clone();
        let program = self.solver.program.clone();

        // Extract the trait-ref that this impl implements, its
        // where-clauses, and the value that it provides for the
        // desired associated type, instantiating all the impl
        // parameters with fresh variables.
        //
        // So, assuming `?1` is the next new variable in `self.infer`, if we had:
        //
        //     impl<T: Clone> Clone for Option<T>
        //
        // this would yield `Option<?1>: Clone` and `?1: Clone`.
        let (impl_trait_ref, (where_clauses, assoc_ty_value)) = {
            let impl_data = &program.impl_data[&self.impl_id];
            let goal_projection = &self.goal.projection;

            // if we are looking for (e.g.) `Iterator::Item`, must be an impl of `Iterator`
            if impl_data.trait_ref.trait_id != goal_projection.trait_ref.trait_id {
                bail!("impl trait `{:?}` does not match projection trait `{:?}`",
                      impl_data.trait_ref.trait_id,
                      goal_projection.trait_ref.trait_id);
            }

            // find the definition for `Item` (must be present or something is wrong with
            // the program)
            let assoc_ty_value = impl_data.assoc_ty_values
                                          .iter()
                                          .find(|v| v.name == goal_projection.name)
                                          .map(|v| &v.value)
                                          .unwrap_or_else(|| {
                                              panic!("impl `{:?}` has no definition for `{}`",
                                                     self.impl_id, goal_projection.name)
                                          });

            // instantiate the trait-ref, where-clause, and assoc-ty-value all together,
            // since they are defined in terms of a common set of variables
            self.infer.instantiate(environment.universe,
                                   &(&impl_data.trait_ref,
                                     (&impl_data.where_clauses,
                                      assoc_ty_value)))
        };

        // Unify the trait-ref we are looking for (`self.goal`) with
        // the trait-ref that the impl supplies (if we can). This will
        // result in some auxiliary normalization clauses we must
        // prove.
        let normalize_to1 = self.infer.unify(&self.goal.projection.trait_ref, &impl_trait_ref)?;
        debug!("implemented_with::solve: normalize_to1={:?}", normalize_to1);

        // Unify the result of normalization (`self.goal.ty`) with the
        // value that this impl provides (`assoc_ty_value`).
        let normalize_to2 = self.infer.unify(&self.goal.ty, &assoc_ty_value)?;
        debug!("implemented_with::solve: normalize_to2={:?}", normalize_to2);

        // Combine the where-clauses from the impl with the results
        // from unification into one master list of things to solve,
        // pairing each with the environment.
        let env_where_clauses: Vec<_> =
            where_clauses.into_iter()
                         .chain(normalize_to1.into_iter().map(WhereClause::NormalizeTo))
                         .chain(normalize_to2.into_iter().map(WhereClause::NormalizeTo))
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
