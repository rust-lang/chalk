use std::fmt::{Debug, Error, Formatter};
use crate::{ExClause, TableIndex};
use crate::context::{Context, InferenceTable};
use crate::table::AnswerIndex;

#[derive(Debug)]
crate struct CanonicalStrand<C: Context> {
    pub(super) canonical_ex_clause: C::CanonicalExClause,

    /// Index into `ex_clause.subgoals`.
    crate selected_subgoal: Option<SelectedSubgoal<C>>,
}

crate struct Strand<'table, C: Context + 'table, I: Context + 'table> {
    crate infer: &'table mut dyn InferenceTable<C, I>,

    pub(super) ex_clause: ExClause<I>,

    /// Index into `ex_clause.subgoals`.
    crate selected_subgoal: Option<SelectedSubgoal<C>>,
}

#[derive(Clone, Debug)]
crate struct SelectedSubgoal<C: Context> {
    /// The index of the subgoal in `ex_clause.subgoals`
    crate subgoal_index: usize,

    /// The index of the table that we created or found for this subgoal
    pub(super) subgoal_table: TableIndex,

    /// Index of the answer we should request next from the table
    crate answer_index: AnswerIndex,

    /// Maps the universes of the subgoal to the canonical universes
    /// used in the table
    crate universe_map: C::UniverseMap,
}

impl<'table, C: Context, I: Context> Debug for Strand<'table, C, I> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        fmt.debug_struct("Strand")
            .field("ex_clause", &self.ex_clause)
            .field("selected_subgoal", &self.selected_subgoal)
            .finish()
    }
}
