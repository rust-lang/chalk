use crate::context::Context;
use crate::table::Table;
use crate::TableIndex;
use rustc_hash::FxHashMap;
use std::ops::{Index, IndexMut};

use chalk_ir::interner::Interner;
use chalk_ir::{Goal, InEnvironment, UCanonical};

/// See `Forest`.
pub(crate) struct Tables<I: Interner, C: Context<I>> {
    /// Maps from a canonical goal to the index of its table.
    table_indices: FxHashMap<UCanonical<InEnvironment<Goal<I>>>, TableIndex>,

    /// Table: as described above, stores the key information for each
    /// tree in the forest.
    tables: Vec<Table<I, C>>,
}

impl<I: Interner, C: Context<I>> Tables<I, C> {
    pub(crate) fn new() -> Tables<I, C> {
        Tables {
            table_indices: FxHashMap::default(),
            tables: Vec::default(),
        }
    }

    /// The index that will be given to the next table to be inserted.
    pub(super) fn next_index(&self) -> TableIndex {
        TableIndex {
            value: self.tables.len(),
        }
    }

    pub(super) fn insert(&mut self, table: Table<I, C>) -> TableIndex {
        let goal = table.table_goal.clone();
        let index = self.next_index();
        self.tables.push(table);
        self.table_indices.insert(goal, index);
        index
    }

    pub(super) fn index_of(&self, literal: &UCanonical<InEnvironment<Goal<I>>>) -> Option<TableIndex> {
        self.table_indices.get(literal).cloned()
    }
}

impl<I: Interner, C: Context<I>> Index<TableIndex> for Tables<I, C> {
    type Output = Table<I, C>;

    fn index(&self, index: TableIndex) -> &Table<I, C> {
        &self.tables[index.value]
    }
}

impl<I: Interner, C: Context<I>> IndexMut<TableIndex> for Tables<I, C> {
    fn index_mut(&mut self, index: TableIndex) -> &mut Table<I, C> {
        &mut self.tables[index.value]
    }
}

impl<'a, I: Interner, C: Context<I>> IntoIterator for &'a mut Tables<I, C> {
    type IntoIter = <&'a mut Vec<Table<I, C>> as IntoIterator>::IntoIter;
    type Item = <&'a mut Vec<Table<I, C>> as IntoIterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter(&mut self.tables)
    }
}
