use crate::context::Context;
use crate::table::AnswerIndex;
use crate::{ExClause, TableIndex, TimeStamp};
use std::fmt::{Debug, Error, Formatter};

#[derive(Debug)]
pub(crate) struct CanonicalStrand<C: Context> {
    pub(super) canonical_ex_clause: C::CanonicalExClause,

    /// Index into `ex_clause.subgoals`.
    pub(crate) selected_subgoal: Option<SelectedSubgoal<C>>,

    pub(crate) last_pursued_time: TimeStamp,
}

pub(crate) struct Strand<C: Context> {
    pub(super) infer: C::InferenceTable,

    pub(super) ex_clause: ExClause<C>,

    /// Index into `ex_clause.subgoals`.
    pub(crate) selected_subgoal: Option<SelectedSubgoal<C>>,

    pub(crate) last_pursued_time: TimeStamp,
}

#[derive(Clone, Debug)]
pub(crate) struct SelectedSubgoal<C: Context> {
    /// The index of the subgoal in `ex_clause.subgoals`
    pub(crate) subgoal_index: usize,

    /// The index of the table that we created or found for this subgoal
    pub(super) subgoal_table: TableIndex,

    /// Index of the answer we should request next from the table
    pub(crate) answer_index: AnswerIndex,

    /// Maps the universes of the subgoal to the canonical universes
    /// used in the table
    pub(crate) universe_map: C::UniverseMap,
}

impl<C: Context> Debug for Strand<C> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        fmt.debug_struct("Strand")
            .field("ex_clause", &self.ex_clause)
            .field("selected_subgoal", &self.selected_subgoal)
            .finish()
    }
}
