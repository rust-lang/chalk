use crate::context::Context;
use crate::strand::Strand;
use crate::{Minimums, TableIndex, TimeStamp};
use std::ops::{Index, IndexMut, Range};

/// See `Forest`.
#[derive(Debug)]
pub(crate) struct Stack<C: Context> {
    /// Stack: as described above, stores the in-progress goals.
    stack: Vec<StackEntry<C>>,
}

impl<C: Context> Default for Stack<C> {
    fn default() -> Self {
        Stack { stack: vec![] }
    }
}

index_struct! {
    /// The StackIndex identifies the position of a table's goal in the
    /// stack of goals that are actively being processed. Note that once a
    /// table is completely evaluated, it may be popped from the stack,
    /// and hence no longer have a stack index.
    pub(crate) struct StackIndex {
        value: usize,
    }
}

#[derive(Debug)]
pub(crate) struct StackEntry<C: Context> {
    /// The goal G from the stack entry `A :- G` represented here.
    pub(super) table: TableIndex,

    /// The clock TimeStamp of this stack entry.
    pub(super) clock: TimeStamp,

    pub(super) cyclic_minimums: Minimums,

    // FIXME: should store this as an index.
    // This would mean that if we unwind,
    // we don't need to worry about losing a strand
    pub(super) active_strand: Option<Strand<C>>,
}

impl<C: Context> Stack<C> {
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
        depth..StackIndex::from(self.stack.len())
    }

    pub(super) fn push(
        &mut self,
        table: TableIndex,
        clock: TimeStamp,
        cyclic_minimums: Minimums,
    ) -> StackIndex {
        let old_len = self.stack.len();
        self.stack.push(StackEntry {
            table,
            clock,
            cyclic_minimums,
            active_strand: None,
        });
        StackIndex::from(old_len)
    }

    pub(super) fn pop(&mut self, depth: StackIndex) -> Option<StackIndex> {
        assert_eq!(self.stack.len(), depth.value + 1);
        self.stack.pop();
        if !self.stack.is_empty() {
            Some(StackIndex::from(self.stack.len() - 1))
        } else {
            None
        }
    }
}

impl<C: Context> Index<StackIndex> for Stack<C> {
    type Output = StackEntry<C>;

    fn index(&self, index: StackIndex) -> &StackEntry<C> {
        &self.stack[index.value]
    }
}

impl<C: Context> IndexMut<StackIndex> for Stack<C> {
    fn index_mut(&mut self, index: StackIndex) -> &mut StackEntry<C> {
        &mut self.stack[index.value]
    }
}
