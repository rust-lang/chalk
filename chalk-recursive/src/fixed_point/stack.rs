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

/// The data we actively keep for each goal on the stack.
pub(super) struct StackEntry {
    /// Was this a coinductive goal?
    coinductive_goal: bool,

    /// Initially false, set to true when some subgoal depends on us.
    cycle: bool,
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

    pub(super) fn push(&mut self, coinductive_goal: bool) -> StackDepth {
        let depth = StackDepth {
            depth: self.entries.len(),
        };

        if depth.depth >= self.overflow_depth {
            // This should perhaps be a result or something, though
            // really I'd prefer to move to subgoal abstraction for
            // guaranteeing termination. -nmatsakis
            panic!("overflow depth reached")
        }

        self.entries.push(StackEntry {
            coinductive_goal,
            cycle: false,
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

    /// True iff there exist at least one coinductive goal
    /// and one inductive goal each from the top of the stack
    /// down to (and including) the given depth.
    pub(super) fn mixed_inductive_coinductive_cycle_from(&self, depth: StackDepth) -> bool {
        let coinductive_count = self.entries[depth.depth..]
            .iter()
            .filter(|entry| entry.coinductive_goal)
            .count();
        let total_count = self.entries.len() - depth.depth;
        let any_coinductive = coinductive_count != 0;
        let any_inductive = coinductive_count != total_count;
        any_coinductive && any_inductive
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
