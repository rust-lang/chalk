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

        // Extract the trait-ref that this impl implements and its where-clauses,
        // instantiating all the impl parameters with fresh variables.
        //
        // So, assuming `?1` is the next new variable in `self.infer`, if we had:
        //
        //     impl<T: Clone> Clone for Option<T>
        //
        // this would yield `Option<?1>: Clone` and `?1: Clone`.
        let (impl_trait_ref, where_clauses) =
            self.infer.instantiate(universe, &(&program.impl_data[&self.impl_id].trait_ref,
                                               &program.where_clauses[&self.impl_id]));

        // Unify the trait-ref we are looking for (`self.goal`) with the trait-ref that
        // the impl supplies (if we can).


        unimplemented!()
    }
}
