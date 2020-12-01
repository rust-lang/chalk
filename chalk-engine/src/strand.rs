use crate::table::AnswerIndex;
use crate::{ExClause, TableIndex, TimeStamp};
use std::fmt::{self, Debug};

use chalk_derive::HasInterner;
use chalk_ir::fold::{Fold, Folder};
use chalk_ir::interner::Interner;
use chalk_ir::{Canonical, DebruijnIndex, Fallible, UniverseMap};

use chalk_solve::infer::InferenceTable;

#[derive(Clone, Debug, HasInterner)]
pub(crate) struct InnerStrand<I: Interner> {
    pub(super) ex_clause: ExClause<I>,

    /// Index into `ex_clause.subgoals`.
    pub(crate) selected_subgoal: Option<SelectedSubgoal>,

    pub(crate) last_pursued_time: TimeStamp,
}

pub(crate) type CanonicalStrand<I> = Canonical<InnerStrand<I>>;

#[derive(HasInterner)]
pub(crate) struct InferringStrand<I: Interner> {
    pub(crate) strand: InnerStrand<I>,
    pub(crate) infer: InferenceTable<I>,
}

#[derive(Clone, Debug)]
pub(crate) struct SelectedSubgoal {
    /// The index of the subgoal in `ex_clause.subgoals`
    pub(crate) subgoal_index: usize,

    /// The index of the table that we created or found for this subgoal
    pub(super) subgoal_table: TableIndex,

    /// Index of the answer we should request next from the table
    pub(crate) answer_index: AnswerIndex,

    /// Maps the universes of the subgoal to the canonical universes
    /// used in the table
    pub(crate) universe_map: UniverseMap,
}

impl<I: Interner> Debug for InferringStrand<I> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("InferringStrand")
            .field("strand", &self.strand)
            .finish()
    }
}

impl<I: Interner> Fold<I> for InnerStrand<I> {
    type Result = InnerStrand<I>;
    fn fold_with<'i>(
        self,
        folder: &mut dyn Folder<'i, I>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Self::Result>
    where
        I: 'i,
    {
        Ok(InnerStrand {
            ex_clause: self.ex_clause.fold_with(folder, outer_binder)?,
            last_pursued_time: self.last_pursued_time,
            selected_subgoal: self.selected_subgoal.clone(),
        })
    }
}
