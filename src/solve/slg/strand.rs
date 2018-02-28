use solve::infer::ucanonicalize::UniverseMap;
use std::fmt::{Debug, Error, Formatter};
use solve::slg::{ExClause, TableIndex};
use solve::slg::context::Context;
use solve::slg::table::AnswerIndex;

crate struct Strand<C: Context> {
    crate infer: C::InferenceTable,

    pub(super) ex_clause: ExClause,

    /// Index into `ex_clause.subgoals`.
    crate selected_subgoal: Option<SelectedSubgoal>,
}

#[derive(Clone, Debug)]
crate struct SelectedSubgoal {
    /// The index of the subgoal in `ex_clause.subgoals`
    crate subgoal_index: usize,

    /// The index of the table that we created or found for this subgoal
    pub(super) subgoal_table: TableIndex,

    /// Index of the answer we should request next from the table
    crate answer_index: AnswerIndex,

    /// Maps the universes of the subgoal to the canonical universes
    /// used in the table
    crate universe_map: UniverseMap,
}

impl<C: Context> Debug for Strand<C> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        fmt.debug_struct("Strand")
            .field("ex_clause", &self.ex_clause)
            .field("selected_subgoal", &self.selected_subgoal)
            .finish()
    }
}
