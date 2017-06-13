use petgraph::prelude::*;

use errors::Result;
use ir::{Program, ItemId};

mod solve;

type Forest = Graph<ItemId, ()>;

impl Program {
    pub fn record_specialization_priorities(&mut self) -> Result<()> {
        let forest = self.build_specialization_forest()?;

        for root_idx in forest.externals(Direction::Incoming) {
            self.set_priorities(root_idx, &forest, 0);
        }

        Ok(())
    }

    fn build_specialization_forest(&self) -> Result<Forest> {
        let mut forest = DiGraphMap::new();

        self.find_specializations(|less_special, more_special| {
            forest.add_edge(less_special, more_special, ());
        })?;

        Ok(forest.into_graph())
    }

    fn set_priorities(&mut self, idx: NodeIndex, forest: &Forest, p: usize) {
        let impl_id = forest.node_weight(idx).expect("received valid node index");

        {
            let impl_datum = self.impl_data.get_mut(impl_id).expect("node is valid impl id");
            impl_datum.binders.value.specialization_priority = p;
        }

        for child_idx in forest.neighbors(idx) {
            self.set_priorities(child_idx, forest, p + 1)
        }
    }
}
