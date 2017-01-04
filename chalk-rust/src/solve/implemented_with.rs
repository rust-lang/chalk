use errors::*;
use solve::infer::InferenceTable;
use solve::Solution;
use ir::*;

pub struct ImplementedWith {
    infer: InferenceTable,
    environment: Environment,
    goal: TraitRef,
    impl_id: ItemId,
}

impl ImplementedWith {
    pub fn solve(&mut self) -> Result<Solution<TraitRef>> {
        let program = self.environment.program.clone();
        let impl_data = self.impl_data[&self.impl_id];
    }
}
