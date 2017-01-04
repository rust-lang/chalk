use solve::environment::Environment;
use solve::infer::InferenceTable;
use solve::quantify::Quantified;
use ir::*;

pub struct Implemented {
    infer: InferenceTable,
    environment: Environment,
    goal: TraitRef,
}

impl Implemented {
    pub fn new(&self, q: Quantified<(Environment, TraitRef)>) -> Self {
        let (environment, goal) = q.value;
        let infer = InferenceTable::new_with_vars(q.binders, environment.universe);
        Implemented {
            infer: infer,
            environment: environment,
            goal: goal,
        }
    }

    pub fn solve(&mut self) {
        let program = self.environment.program.clone();

        // Find the impls for this trait.
        let impls: Vec<_> = program.impl_data
            .iter()
            .filter(|&(&impl_id, impl_data)| impl_data.trait_ref.trait_id == self.goal.trait_id)
            .collect();

        // For each impl, recursively apply it.

    }
}
