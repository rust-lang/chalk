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
        cyclic_minimums: Minimums,
        clock: TimeStamp,
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

    /// Pops the top-most entry from the stack:
    /// * If the stack is now empty, returns false.
    /// * Otherwise, returns true.
    fn pop_and_adjust_depth(&mut self) -> bool {
        self.stack.pop();
        !self.stack.is_empty()
    }

    /// Pops the top-most entry from the stack, which should have the depth `*depth`:
    /// * If the stack is now empty, returns None.
    /// * Otherwise, `take`s the active strand from the new top and returns it.
    pub(super) fn pop_and_take_caller_strand(&mut self) -> Option<Strand<C>> {
        if self.pop_and_adjust_depth() {
            Some(self.top().active_strand.take().unwrap())
        } else {
            None
        }
    }

    /// Pops the top-most entry from the stack, which should have the depth `*depth`:
    /// * If the stack is now empty, returns None.
    /// * Otherwise, borrows the active strand (mutably) from the new top and returns it.
    pub(super) fn pop_and_borrow_caller_strand(&mut self) -> Option<&mut Strand<C>> {
        if self.pop_and_adjust_depth() {
            Some(self.top().active_strand.as_mut().unwrap())
        } else {
            None
        }
    }

    pub(super) fn top(&mut self) -> &mut StackEntry<C> {
        self.stack.last_mut().unwrap()
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
