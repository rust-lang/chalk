use ir::*;
use solve::infer::Quantified;
use solve::environment::Environment;
use solve::solver::Solver;
use std::sync::Arc;

pub struct Implemented<'s> {
    solver: &'s mut Solver,
    env_goal: Quantified<(Arc<Environment>, TraitRef)>,
}

impl<'s> Implemented<'s> {
    pub fn new(&self,
               solver: &'s mut Solver,
               env_goal: Quantified<(Arc<Environment>, TraitRef)>)
               -> Self {
        Implemented {
            solver: solver,
            env_goal: env_goal,
        }
    }

    pub fn solve(&mut self) {
        let program = self.solver.program.clone();

        // Find the impls for this trait.
        let goal_trait_id = self.env_goal.value.1.trait_id;
        let impls: Vec<_> = program.impl_data
            .iter()
            .filter(|&(_, impl_data)| impl_data.trait_ref.trait_id == goal_trait_id)
            .collect();

        // For each impl, recursively apply it.
        
    }
}
