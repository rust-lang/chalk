use crate::table::Table;
use crate::TableIndex;
use rustc_hash::FxHashMap;
use std::ops::{Index, IndexMut};

use chalk_ir::interner::Interner;
use chalk_ir::{Goal, InEnvironment, UCanonical};

/// See `Forest`.
#[derive(Debug)]
pub(crate) struct Tables<I: Interner> {
    /// Maps from a canonical goal to the index of its table.
    table_indices: FxHashMap<UCanonical<InEnvironment<Goal<I>>>, TableIndex>,

    /// Table: as described above, stores the key information for each
    /// tree in the forest.
    tables: Vec<Table<I>>,
}

impl<I: Interner> Tables<I> {
    pub(crate) fn new() -> Tables<I> {
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

    pub(super) fn insert(&mut self, table: Table<I>) -> TableIndex {
        let goal = table.table_goal.clone();
        let index = self.next_index();
        self.tables.push(table);
        self.table_indices.insert(goal, index);
        index
    }

    pub(super) fn index_of(
        &self,
        literal: &UCanonical<InEnvironment<Goal<I>>>,
    ) -> Option<TableIndex> {
        self.table_indices.get(literal).cloned()
    }
}

impl<I: Interner> Index<TableIndex> for Tables<I> {
    type Output = Table<I>;

    fn index(&self, index: TableIndex) -> &Table<I> {
        &self.tables[index.value]
    }
}

impl<I: Interner> IndexMut<TableIndex> for Tables<I> {
    fn index_mut(&mut self, index: TableIndex) -> &mut Table<I> {
        &mut self.tables[index.value]
    }
}

impl<'a, I: Interner> IntoIterator for &'a mut Tables<I> {
    type IntoIter = <&'a mut Vec<Table<I>> as IntoIterator>::IntoIter;
    type Item = <&'a mut Vec<Table<I>> as IntoIterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter(&mut self.tables)
    }
}
