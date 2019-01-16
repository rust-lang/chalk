use petgraph::prelude::*;

use crate::errors::Result;
use crate::rust_ir::Program;
use chalk_ir::{self, ItemId};
use chalk_solve::solve::SolverChoice;
use std::sync::Arc;

mod orphan;
mod solve;
mod test;

impl Program {
    crate fn record_specialization_priorities(
        &mut self,
        solver_choice: SolverChoice,
    ) -> Result<()> {
        chalk_ir::tls::set_current_program(&Arc::new(self.clone()), || {
            let forest = self.build_specialization_forest(solver_choice)?;

            // Visit every root in the forest & set specialization
            // priority for the tree that is the root of.
            for root_idx in forest.externals(Direction::Incoming) {
                self.set_priorities(root_idx, &forest, 0);
            }

            Ok(())
        })
    }

    // Build the forest of specialization relationships.
    fn build_specialization_forest(
        &self,
        solver_choice: SolverChoice,
    ) -> Result<Graph<ItemId, ()>> {
        // The forest is returned as a graph but built as a GraphMap; this is
        // so that we never add multiple nodes with the same ItemId.
        let mut forest = DiGraphMap::new();

        // Find all specializations (implemented in coherence/solve)
        // Record them in the forest by adding an edge from the less special
        // to the more special.
        self.visit_specializations(solver_choice, |less_special, more_special| {
            forest.add_edge(less_special, more_special, ());
        })?;

        Ok(forest.into_graph())
    }

    // Recursively set priorities for those node and all of its children.
    fn set_priorities(&mut self, idx: NodeIndex, forest: &Graph<ItemId, ()>, p: usize) {
        // Get the impl datum recorded at this node and reset its priority
        {
            let impl_id = forest
                .node_weight(idx)
                .expect("index should be a valid index into graph");
            let impl_datum = self
                .impl_data
                .get_mut(impl_id)
                .expect("node should be valid impl id");
            impl_datum.binders.value.specialization_priority = p;
        }

        // Visit all children of this node, setting their priority to this + 1
        for child_idx in forest.neighbors(idx) {
            self.set_priorities(child_idx, forest, p + 1)
        }
    }
}
