use errors::*;
use ir::*;
use solve::{Solution, Successful};
use solve::environment::{Environment, InEnvironment};
use solve::infer::{InferenceTable, Quantified};
use solve::solver::Solver;
use std::sync::Arc;

pub struct ImplementedWith<'s> {
    solver: &'s mut Solver,
    infer: InferenceTable,
    environment: Arc<Environment>,
    goal: TraitRef,
    impl_id: ItemId,
}

impl<'s> ImplementedWith<'s> {
    pub fn new(solver: &'s mut Solver,
               q: Quantified<InEnvironment<TraitRef>>,
               impl_id: ItemId)
               -> Self {
        let InEnvironment { environment, goal } = q.value;
        let infer = InferenceTable::new_with_vars(q.binders, environment.universe);
        ImplementedWith {
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
        let (impl_trait_ref, mut where_clauses) = self.infer
            .instantiate(environment.universe,
                         &(&program.impl_data[&self.impl_id].trait_ref,
                           &program.where_clauses[&self.impl_id]));

        // Unify the trait-ref we are looking for (`self.goal`) with
        // the trait-ref that the impl supplies (if we can). This will
        // result in some auxiliary normalization clauses we must
        // prove.
        let normalize_to = self.infer.unify(&self.goal, &impl_trait_ref)?;
        where_clauses.extend(normalize_to.into_iter().map(WhereClause::NormalizeTo));

        // Now try to prove the where-clauses one by one. If all of
        // them can be successfully proved, then we know that this
        // impl applies. If any of them error out, this impl does not.
        let successful = self.solve_all(where_clauses)?;
        let refined_goal = self.infer.quantify(&InEnvironment::new(&environment, &self.goal));
        Ok(Solution {
            successful: successful,
            refined_goal: refined_goal,
        })
    }

    fn solve_all(&mut self, mut where_clauses: Vec<WhereClause>) -> Result<Successful> {
        // Try to solve all the where-clauses. We do this via a
        // fixed-point iteration. We try to solve each where-clause in
        // turn. Anything which is successful, we drop; anything
        // ambiguous, we retain in the `where_clauses` array. This
        // process is repeated so long as we are learning new things
        // about our inference state.
        let mut retained = Vec::with_capacity(where_clauses.len());
        let mut progress = true;
        while progress {
            progress = false;

            for wc in where_clauses.drain(..) {
                match self.solve_wc(&wc, &mut progress)? {
                    Successful::Yes => (),
                    Successful::Maybe => retained.push(wc),
                }
            }

            where_clauses.extend(retained.drain(..));
        }

        // If we still have ambiguous where-clauses, then we have an
        // ambiguous overall result.
        if where_clauses.is_empty() {
            Ok(Successful::Yes)
        } else {
            Ok(Successful::Maybe)
        }
    }

    fn solve_wc(&mut self, wc: &WhereClause, inference_progress: &mut bool) -> Result<Successful> {
        let quantified_goal = self.infer.quantify(&InEnvironment::new(&self.environment, wc));
        let solution = self.solver.solve(quantified_goal.clone())?;

        // Regardless of whether the result is ambiguous or not,
        // solving the where-clause may have yielded a refined
        // goal. For example, if the original where-clause was
        // something like `Foo<?4>: Borrow<?3>`, we would have
        // "quantified" that to yield `exists ?0, ?1. Foo<?0>: Borrow<?1>`.
        // We may now have gotten back a refined goal like `exists ?0. Foo<?0>:
        // Borrow<Foo<?0>>`. In that case, we can unify `?3` with `Foo<?4>`.
        //
        // To make that unification happen, we first instantiate all
        // the variables on the goal we got back with new inference
        // variables. So we would thus convert `exists ?0. Foo<?0>:
        // Borrow<Foo<?0>>` into `Foo<?5>: Borrow<Foo<?5>>`.  We would
        // then unify this with our original goal (`Foo<?4>:
        // Borrow<?3>`). This will result in the equations `?4 = ?5`
        // and `?3 = Foo<?5>`.
        //
        // As a potential efficiency improvement, one could imagine a
        // more algorithm written just for this case instead of
        // instantiating with variables and applying the standard
        // unification algorithm. But this is good enough for now.
        if solution.refined_goal != quantified_goal {
            let refined_goal = self.infer
                .instantiate(self.environment.universe, &solution.refined_goal.value);
            self.infer.unify(&self.environment, &refined_goal.environment)?;
            self.infer.unify(wc, &refined_goal.goal)?;
            *inference_progress = true;
        }

        Ok(solution.successful)
    }
}
