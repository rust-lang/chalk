use super::lib::Minimums;
use std::mem;
use std::ops::Index;
use std::ops::IndexMut;
use std::usize;

pub(super) struct Stack {
    // program: Arc<ProgramEnvironment>,
    entries: Vec<StackEntry>,
    overflow_depth: usize,
}

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub(super) struct StackDepth {
    depth: usize,
}

impl StackDepth {
    /// True if this represents the "root goal" on the stack (i.e., there is nothing
    /// below it on the stack).
    pub(super) fn is_root(&self) -> bool {
        self.depth == 0
    }
}

/// The data we actively keep for each goal on the stack.
pub(super) struct StackEntry {
    /// Was this a coinductive goal?
    coinductive_goal: bool,

    /// Initially false, set to true when some subgoal depends on us.
    cycle: bool,

    pub(super) minimums: Minimums,
}

impl Stack {
    pub(super) fn new(
        // program: &Arc<ProgramEnvironment>,
        overflow_depth: usize,
    ) -> Self {
        Stack {
            // program: program.clone(),
            entries: vec![],
            overflow_depth,
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns a shared reference to the top of the stack.
    ///
    /// Panics if the stack is empty.
    pub(super) fn top(&self) -> &StackEntry {
        self.entries.last().unwrap()
    }

    /// Returns a mutable reference to the top of the stack.
    ///
    /// Panics if the stack is empty.
    pub(super) fn top_mut(&mut self) -> &mut StackEntry {
        self.entries.last_mut().unwrap()
    }

    /// Returns the depth of the top of the stack.
    ///
    /// Panics if the stack is empty.
    pub(super) fn top_depth(&self) -> StackDepth {
        assert!(!self.is_empty());
        StackDepth {
            depth: self.entries.len() - 1,
        }
    }

    pub(super) fn push(&mut self, coinductive_goal: bool) -> StackDepth {
        let depth = StackDepth {
            depth: self.entries.len(),
        };

        if depth.depth >= self.overflow_depth {
            // This shoudl perhaps be a result or something, though
            // really I'd prefer to move to subgoal abstraction for
            // guaranteeing termination. -nmatsakis
            panic!("overflow depth reached")
        }

        self.entries.push(StackEntry {
            coinductive_goal,
            cycle: false,
            minimums: Minimums::new(),
        });
        depth
    }

    pub(super) fn pop(&mut self, depth: StackDepth) {
        assert_eq!(
            depth.depth + 1,
            self.entries.len(),
            "mismatched stack push/pop"
        );
        self.entries.pop();
    }

    /// True if all the goals from the top of the stack down to (and
    /// including) the given depth are coinductive.
    pub(super) fn coinductive_cycle_from(&self, depth: StackDepth) -> bool {
        self.entries[depth.depth..]
            .iter()
            .all(|entry| entry.coinductive_goal)
    }
}

impl StackEntry {
    pub(super) fn flag_cycle(&mut self) {
        self.cycle = true;
    }

    pub(super) fn read_and_reset_cycle_flag(&mut self) -> bool {
        mem::replace(&mut self.cycle, false)
    }
}

impl Index<StackDepth> for Stack {
    type Output = StackEntry;

    fn index(&self, depth: StackDepth) -> &StackEntry {
        &self.entries[depth.depth]
    }
}

impl IndexMut<StackDepth> for Stack {
    fn index_mut(&mut self, depth: StackDepth) -> &mut StackEntry {
        &mut self.entries[depth.depth]
    }
}
