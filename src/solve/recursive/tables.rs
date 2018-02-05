use fallible::*;
use std::ops::Index;
use std::ops::IndexMut;

use super::stack::StackDepth;

#[derive(Default)]
crate struct Tables {
    indices: HashMap<CanonicalLeafGoal, TableIndex>,
    tables: Vec<Table>,
}

crate struct TableIndex {
    index: usize
}

crate struct Table {
    crate solution: Fallible<Solution>,
    crate stack_depth: Option<StackDepth>,
}

impl Tables {
    crate fn lookup(&self, goal: &CanonicalLeafGoal) -> Option<TableIndex> {
        self.indices.get(goal)
    }

    crate fn insert(&mut self,
                  goal: &CanonicalLeafGoal,
                  solution: Fallible<Solution>,
                  stack_depth: Option<StackDepth>)
                  -> TableIndex {
        let table = Table { solution, stack_depth };
        let index = self.tables.len();
        self.tables.push(table);
        let previous_index = self.indices.insert(goal.clone(), index);
        assert!(previous_index.is_none());
        TableIndex { index }
    }
}

impl Index<TableIndex> for Tables {
    type Output = Table;

    fn index(&self, table_index: TableIndex) -> &Table {
        &self.tables[table_index.index]
    }
}

impl IndexMut<TableIndex> for Tables {
    fn index(&self, table_index: TableIndex) -> &mut Table {
        &self.tables[table_index.index]
    }
}
