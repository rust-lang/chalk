use errors::*;
use ir::*;
use solve::environment::InEnvironment;
use solve::implemented_with::ImplementedWith;
use solve::solver::Solver;
use solve::{Solution, Successful};
use std::collections::HashSet;

pub struct Implemented<'s> {
    solver: &'s mut Solver,
    env_goal: Quantified<InEnvironment<TraitRef>>,
}

impl<'s> Implemented<'s> {
    pub fn new(solver: &'s mut Solver,
               env_goal: Quantified<InEnvironment<TraitRef>>)
               -> Self {
        Implemented {
            solver: solver,
            env_goal: env_goal,
        }
    }

    pub fn solve(&mut self) -> Result<Solution<Quantified<InEnvironment<TraitRef>>>> {
        let program = self.solver.program.clone();

        // For each impl, recursively apply it. Note that all we need
        // to verify is that `T: Foo` **is implemented**. We don't
        // actually need to know *which impl* implified with.
        let mut candidates = HashSet::new();
        for (&impl_id, impl_data) in &program.impl_data {
            // screen out the things that are for the totally wrong
            // trait, just to keep debug logging under control
            let goal_trait_id = self.env_goal.value.goal.trait_id;
            if impl_data.trait_ref.trait_id != goal_trait_id {
                continue;
            }

            let result = ImplementedWith::new(self.solver, self.env_goal.clone(), impl_id).solve();
            if let Ok(solution) = result {
                // If we found an impl which definitively applies
                // **without unifying anything in the goal**, then we
                // know that the type is indeed implemented (though
                // there may be other impls which also apply, because
                // of specialization).
                //
                // If the impl **does unify things in the goal**, then
                // it only applies **conditionally**, and we have to
                // see what other impls apply. If this is indeed the
                // only applicable one, then we can opt to use it (and
                // this implies that those variables can be unified on
                // the other side, since its the only way to ensure
                // that the trait is implemented). But if there are
                // multiple impls, perhaps with distinct unifications,
                // then we have to return an ambiguous result.
                if let Successful::Yes = solution.successful {
                    if solution.refined_goal == self.env_goal {
                        return Ok(solution);
                    }
                }

                candidates.insert(solution);
            }
        }

        if candidates.len() == 0 {
            bail!("`{:?}` is not implemented in environment `{:?}`",
                  self.env_goal.value.goal,
                  self.env_goal.value.environment);
        }

        if candidates.len() == 1 {
            return Ok(candidates.into_iter().next().unwrap());
        }

        // There are multiple candidates and they don't agree about
        // what we can infer thus far. Return an ambiguous
        // result. This actually isn't as precise as it could be, in
        // two ways:
        //
        // a. It might be that while there are multiple distinct
        //    candidates, they all agree about *some things*. To be
        //    maximally precise, we would compute the intersection of
        //    what they agree on. It's not clear though that this is
        //    actually what we want Rust's inference to do, and it's
        //    certainly not what it does today.
        // b. There might also be an ambiguous candidate and a successful
        //    candidate, both with the same refined-goal. In that case,
        //    we could probably claim success, since if the conditions of the
        //    ambiguous candidate were met, we now the success would apply.
        //    Example: `?0: Clone` yields ambiguous candidate `Option<?0>: Clone`
        //    and successful candidate `Option<?0>: Clone`.
        //
        // But you get the idea.
        return Ok(Solution {
            successful: Successful::Maybe,
            refined_goal: self.env_goal.clone()
        });
    }
}
