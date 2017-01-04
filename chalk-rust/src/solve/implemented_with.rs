use errors::*;
use solve::environment::Environment;
use solve::infer::InferenceTable;
use solve::quantify::Quantified;
use solve::Solution;
use ir::*;

pub struct ImplementedWith {
    infer: InferenceTable,
    environment: Environment,
    goal: TraitRef,
    impl_id: ItemId,
}

impl ImplementedWith {
    pub fn new(&self, q: Quantified<(Environment, TraitRef)>, impl_id: ItemId) -> Self {
        let (environment, goal) = q.value;
        let infer = InferenceTable::new_with_vars(q.binders, environment.universe);
        ImplementedWith {
            environment: environment,
            infer: infer,
            goal: goal,
            impl_id: impl_id,
        }
    }

    pub fn solve(&mut self) -> Result<Solution<TraitRef>> {
        let universe = self.environment.universe;
        let program = self.environment.program.clone();
        let (impl_data, where_clauses) =
            self.infer.instantiate(universe, &(&program.impl_data[&self.impl_id],
                                               &program.where_clauses[&self.impl_id]));

        unimplemented!()
    }
}
