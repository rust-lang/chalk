use fallible::*;
use solve::Solution;
use std::collections::HashMap;
use std::ops::Index;
use std::ops::IndexMut;
use std::ops::Add;
use std::usize;

use super::{UCanonicalGoal, Minimums};
use super::stack::StackDepth;

pub(super) struct SearchGraph {
    indices: HashMap<UCanonicalGoal, DepthFirstNumber>,
    nodes: Vec<Node>,
}

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub(super) struct DepthFirstNumber {
    index: usize,
}

pub(super) struct Node {
    pub goal: UCanonicalGoal,

    pub solution: Fallible<Solution>,

    /// This is `Some(X)` if we are actively exploring this node, or
    /// `None` otherwise.
    pub stack_depth: Option<StackDepth>,

    /// While this node is on the stack, this field will be set to
    /// contain our own depth-first number. Once the node is popped
    /// from the stack, it contains the DFN of the minimal ancestor
    /// that the table reached (or MAX if no cycle was encountered).
    pub links: Minimums,
}

impl SearchGraph {
    pub fn new() -> Self {
        SearchGraph {
            indices: HashMap::new(),
            nodes: vec![],
        }
    }

    pub fn lookup(&self, goal: &UCanonicalGoal) -> Option<DepthFirstNumber> {
        self.indices.get(goal).cloned()
    }

    /// Insert a new search node in the tree. The node will be in the initial
    /// state for a search node:
    ///
    /// - stack depth as given
    /// - links set to its own DFN
    /// - solution is initially `NoSolution`
    pub fn insert(&mut self, goal: &UCanonicalGoal, stack_depth: StackDepth) -> DepthFirstNumber {
        let dfn = DepthFirstNumber {
            index: self.nodes.len(),
        };
        let node = Node {
            goal: goal.clone(),
            solution: Err(NoSolution),
            stack_depth: Some(stack_depth),
            links: Minimums { positive: dfn },
        };
        self.nodes.push(node);
        let previous_index = self.indices.insert(goal.clone(), dfn);
        assert!(previous_index.is_none());
        dfn
    }

    /// Clears all nodes with a depth-first number greater than or equal `dfn`.
    pub fn rollback_to(&mut self, dfn: DepthFirstNumber) {
        debug!("rollback_to(dfn={:?})", dfn);
        self.indices.retain(|_key, value| *value < dfn);
        self.nodes.truncate(dfn.index);
    }

    /// Removes all nodes with a depth-first-number greater than or
    /// equal to `dfn`, adding their final solutions into the cache.
    pub fn move_to_cache(
        &mut self,
        dfn: DepthFirstNumber,
        cache: &mut HashMap<UCanonicalGoal, Fallible<Solution>>,
    ) {
        debug!("move_to_cache(dfn={:?})", dfn);
        self.indices.retain(|_key, value| *value < dfn);
        for node in self.nodes.drain(dfn.index..) {
            assert!(node.stack_depth.is_none());
            assert!(node.links.positive >= dfn);
            debug!("caching solution {:?} for {:?}", node.solution, node.goal);
            cache.insert(node.goal, node.solution);
        }
    }
}

impl Index<DepthFirstNumber> for SearchGraph {
    type Output = Node;

    fn index(&self, table_index: DepthFirstNumber) -> &Node {
        &self.nodes[table_index.index]
    }
}

impl IndexMut<DepthFirstNumber> for SearchGraph {
    fn index_mut(&mut self, table_index: DepthFirstNumber) -> &mut Node {
        &mut self.nodes[table_index.index]
    }
}

impl DepthFirstNumber {
    pub const MAX: DepthFirstNumber = DepthFirstNumber { index: usize::MAX };
}

impl Add<usize> for DepthFirstNumber {
    type Output = DepthFirstNumber;

    fn add(self, v: usize) -> DepthFirstNumber {
        DepthFirstNumber {
            index: self.index + v,
        }
    }
}
