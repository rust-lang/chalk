use solve::slg::{TableIndex, UCanonicalGoal};
use solve::slg::table::Table;
use std::collections::HashMap;
use std::ops::{Index, IndexMut};

/// See `Forest`.
#[derive(Default)]
crate struct Tables {
    /// Maps from a canonical goal to the index of its table.
    table_indices: HashMap<UCanonicalGoal, TableIndex>,

    /// Table: as described above, stores the key information for each
    /// tree in the forest.
    tables: Vec<Table>,
}

impl Tables {
    /// The index that will be given to the next table to be inserted.
    pub(super) fn next_index(&self) -> TableIndex {
        TableIndex {
            value: self.tables.len(),
        }
    }

    pub(super) fn insert(&mut self, goal: UCanonicalGoal, coinductive_goal: bool) -> TableIndex {
        let index = self.next_index();
        self.tables.push(Table::new(goal.clone(), coinductive_goal));
        self.table_indices.insert(goal, index);
        index
    }

    pub(super) fn index_of(&self, literal: &UCanonicalGoal) -> Option<TableIndex> {
        self.table_indices.get(literal).cloned()
    }
}

impl Index<TableIndex> for Tables {
    type Output = Table;

    fn index(&self, index: TableIndex) -> &Table {
        &self.tables[index.value]
    }
}

impl IndexMut<TableIndex> for Tables {
    fn index_mut(&mut self, index: TableIndex) -> &mut Table {
        &mut self.tables[index.value]
    }
}

impl<'a> IntoIterator for &'a mut Tables {
    type IntoIter = <&'a mut Vec<Table> as IntoIterator>::IntoIter;
    type Item = <&'a mut Vec<Table> as IntoIterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter(&mut self.tables)
    }
}

