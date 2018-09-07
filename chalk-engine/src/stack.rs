use {DepthFirstNumber, TableIndex};
use std::ops::{Index, IndexMut, Range};

/// See `Forest`.
#[derive(Default)]
pub(crate) struct Stack {
    /// Stack: as described above, stores the in-progress goals.
    stack: Vec<StackEntry>,
}

/// The StackIndex identifies the position of a table's goal in the
/// stack of goals that are actively being processed. Note that once a
/// table is completely evaluated, it may be popped from the stack,
/// and hence no longer have a stack index.
index_struct! {
    pub(crate) struct StackIndex {
        value: usize,
    }
}

pub(crate) struct StackEntry {
    /// The goal G from the stack entry `A :- G` represented here.
    pub(super) table: TableIndex,

    /// The DFN of this computation.
    pub(super) dfn: DepthFirstNumber,
}

impl Stack {
    pub(super) fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Searches the stack to see if `table` is active. If so, returns
    /// its stack index.
    pub(super) fn is_active(&self, table: TableIndex) -> Option<StackIndex> {
        self.stack
            .iter()
            .enumerate()
            .filter_map(|(index, stack_entry)| {
                if stack_entry.table == table {
                    Some(StackIndex::from(index))
                } else {
                    None
                }
            })
            .next()
    }

    pub(super) fn top_of_stack_from(&self, depth: StackIndex) -> Range<StackIndex> {
        depth .. StackIndex::from(self.stack.len())
    }

    pub(super) fn push(&mut self, table: TableIndex, dfn: DepthFirstNumber) -> StackIndex {
        let old_len = self.stack.len();
        self.stack.push(StackEntry { table, dfn });
        StackIndex::from(old_len)
    }

    pub(super) fn pop(&mut self, table: TableIndex, depth: StackIndex) {
        assert_eq!(self.stack.len(), depth.value + 1);
        assert_eq!(self[depth].table, table);
        self.stack.pop();
    }
}

impl Index<StackIndex> for Stack {
    type Output = StackEntry;

    fn index(&self, index: StackIndex) -> &StackEntry {
        &self.stack[index.value]
    }
}

impl IndexMut<StackIndex> for Stack {
    fn index_mut(&mut self, index: StackIndex) -> &mut StackEntry {
        &mut self.stack[index.value]
    }
}
