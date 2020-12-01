use crate::index_struct;
use crate::strand::CanonicalStrand;
use crate::tables::Tables;
use crate::{Minimums, TableIndex, TimeStamp};
use std::fmt;
use std::ops::{Index, IndexMut, Range};

use chalk_ir::interner::Interner;

/// See `Forest`.
#[derive(Debug)]
pub(crate) struct Stack<I: Interner> {
    /// Stack: as described above, stores the in-progress goals.
    stack: Vec<StackEntry<I>>,
}

impl<I: Interner> Stack<I> {
    // This isn't actually used, but it can be helpful when debugging stack issues
    #[allow(dead_code)]
    pub(crate) fn debug_with<'a>(&'a self, tables: &'a Tables<I>) -> StackDebug<'_, I> {
        StackDebug {
            stack: self,
            tables,
        }
    }
}

pub(crate) struct StackDebug<'a, I: Interner> {
    stack: &'a Stack<I>,
    tables: &'a Tables<I>,
}

impl<I: Interner> fmt::Debug for StackDebug<'_, I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "---- Stack ----")?;
        for entry in self.stack.stack.iter() {
            writeln!(f, "  --- StackEntry ---")?;
            writeln!(
                f,
                "  Table {:?} with goal {:?}",
                entry.table, self.tables[entry.table].table_goal
            )?;
            writeln!(f, "  Active strand: {:#?}", entry.active_strand)?;
            writeln!(
                f,
                "  Additional strands: {:#?}",
                self.tables[entry.table].strands().collect::<Vec<_>>()
            )?;
        }
        write!(f, "---- End Stack ----")?;
        Ok(())
    }
}

impl<I: Interner> Default for Stack<I> {
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
pub(crate) struct StackEntry<I: Interner> {
    /// The goal G from the stack entry `A :- G` represented here.
    pub(super) table: TableIndex,

    /// The clock TimeStamp of this stack entry.
    pub(super) clock: TimeStamp,

    pub(super) cyclic_minimums: Minimums,

    // FIXME: should store this as an index.
    // This would mean that if we unwind,
    // we don't need to worry about losing a strand
    pub(super) active_strand: Option<CanonicalStrand<I>>,
}

impl<I: Interner> Stack<I> {
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
    pub(super) fn pop_and_take_caller_strand(&mut self) -> Option<CanonicalStrand<I>> {
        if self.pop_and_adjust_depth() {
            Some(self.top().active_strand.take().unwrap())
        } else {
            None
        }
    }

    /// Pops the top-most entry from the stack, which should have the depth `*depth`:
    /// * If the stack is now empty, returns None.
    /// * Otherwise, borrows the active strand (mutably) from the new top and returns it.
    pub(super) fn pop_and_borrow_caller_strand(&mut self) -> Option<&mut CanonicalStrand<I>> {
        if self.pop_and_adjust_depth() {
            Some(self.top().active_strand.as_mut().unwrap())
        } else {
            None
        }
    }

    pub(super) fn top(&mut self) -> &mut StackEntry<I> {
        self.stack.last_mut().unwrap()
    }
}

impl<I: Interner> Index<StackIndex> for Stack<I> {
    type Output = StackEntry<I>;

    fn index(&self, index: StackIndex) -> &StackEntry<I> {
        &self.stack[index.value]
    }
}

impl<I: Interner> IndexMut<StackIndex> for Stack<I> {
    fn index_mut(&mut self, index: StackIndex) -> &mut StackEntry<I> {
        &mut self.stack[index.value]
    }
}
