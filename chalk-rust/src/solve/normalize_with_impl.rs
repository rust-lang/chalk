use cast::Cast;
use errors::*;
use fold::Shifted;
use ir::*;
use solve::Solution;
use solve::environment::{Environment, InEnvironment};
use solve::fulfill::Fulfill;
use solve::infer::InferenceTable;
use solve::solver::Solver;
use std::sync::Arc;

pub struct NormalizeWithImpl<'s> {
    fulfill: Fulfill<'s>,
    environment: Arc<Environment>,
    goal: Normalize,
    impl_id: ItemId,
}

impl<'s> NormalizeWithImpl<'s> {
    pub fn new(solver: &'s mut Solver,
               q: Quantified<InEnvironment<Normalize>>,
               impl_id: ItemId)
               -> Self {
        let InEnvironment { environment, goal } = q.value;
        let infer = InferenceTable::new_with_vars(&q.binders);
        NormalizeWithImpl {
            fulfill: Fulfill::new(solver, infer),
            environment: environment,
            goal: goal,
            impl_id: impl_id,
        }
    }

    pub fn solve(mut self) -> Result<Solution<InEnvironment<Normalize>>> {
        let environment = self.environment.clone();
        let program = self.fulfill.program();
        let goal_projection = &self.goal.projection;
        let (associated_ty_data, goal_trait_params, goal_other_params) =
            program.split_projection(goal_projection);

        assert_eq!(goal_trait_params.len() + goal_other_params.len(),
                   associated_ty_data.parameter_kinds.len());

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
        let ((impl_trait_ref, where_clauses), assoc_ty_value) = {
            let impl_data = &program.impl_data[&self.impl_id];

            // if we are looking for (e.g.) `Iterator::Item`, must be an impl of `Iterator`
            if impl_data.trait_ref.trait_id != associated_ty_data.trait_id {
                bail!("impl trait `{:?}` does not match projection trait `{:?}`",
                      impl_data.trait_ref.trait_id,
                      associated_ty_data.trait_id)
            }

            debug!("impl_data = {:?}", impl_data.trait_ref);

            // find the definition for `Item` (must be present or something is wrong with
            // the program)
            let assoc_ty_value = impl_data.assoc_ty_values
                                          .iter()
                                          .find(|v| v.name == associated_ty_data.name)
                                          .map(|v| &v.value)
                                          .unwrap_or_else(|| {
                                              panic!("impl `{:?}` has no definition for `{}`",
                                                     self.impl_id, associated_ty_data.name)
                                          });

            // the associated item value is defined with additional binders compared
            // to the trait-ref and where-clauses:
            //
            // ```
            // impl<A> Iterable for Vec<A> {
            //     ^^^ binders for trait-ref and where-clauses
            //     type Iterator<'a> = vec::Iter<'a, A>;
            //                  ^^^^ add'l binders for associated ty value
            // }
            // ```
            //
            // the new binders are introduced at depth 0. So in this
            // case the full set of binders would be `['a,
            // A]`. Therefore, we want to up-shift references in the
            // trait-ref/where-clauses by 1 (when they say index 0,
            // they mean `A`, which is now at index 1).
            let num_addl_binders = goal_other_params.len();
            debug!("associated_ty_data.parameter_kinds = {:?}", associated_ty_data.parameter_kinds);
            debug!("impl_data.parameter_kinds = {:?}", impl_data.parameter_kinds);
            let parameter_kinds =
                associated_ty_data.parameter_kinds
                                  .iter()
                                  .take(num_addl_binders)
                                  .chain(impl_data.parameter_kinds.iter())
                                  .map(|k| k.as_ref().map(|_| environment.universe));
            let value_to_fold = {
                let impl_values = (&impl_data.trait_ref, &impl_data.where_clauses);
                (Shifted::new(num_addl_binders, impl_values), assoc_ty_value)
            };

            // instantiate the trait-ref, where-clause, and assoc-ty-value all together,
            // since they are defined in terms of a common set of variables
            self.fulfill.instantiate(parameter_kinds, &value_to_fold)
        };

        // Unify the trait-ref we are looking for (`self.goal`) with
        // the trait-ref that the impl supplies (if we can).
        self.fulfill.unify(&environment, goal_trait_params, &impl_trait_ref.parameters[..])?;

        // Unify the result of normalization (`self.goal.ty`) with the
        // value that this impl provides (`assoc_ty_value`).
        self.fulfill.unify(&environment, &self.goal.ty, &assoc_ty_value)?;

        // Add the where-clauses from the impl to list of things to solve.
        self.fulfill.extend(
            where_clauses.into_iter()
                         .map(|wc| InEnvironment::new(&environment, wc.cast())));

        // Now try to prove the where-clauses one by one. If all of
        // them can be successfully proved, then we know that this
        // impl applies. If any of them error out, this impl does not.
        let successful = self.fulfill.solve_all()?;
        let refined_goal = self.fulfill.refine_goal(InEnvironment::new(&environment, &self.goal));
        Ok(Solution {
            successful: successful,
            refined_goal: refined_goal,
        })
    }
}
