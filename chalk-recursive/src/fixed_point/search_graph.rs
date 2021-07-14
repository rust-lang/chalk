use super::stack::StackDepth;
use super::{Cache, Minimums};
use rustc_hash::FxHashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Add;
use std::ops::Index;
use std::ops::IndexMut;
use std::usize;
use tracing::{debug, instrument};

/// The "search graph" stores in-progress goals that are still
/// being solved.
pub(super) struct SearchGraph<K, V>
where
    K: Hash + Eq + Debug + Clone,
    V: Debug + Clone,
{
    indices: FxHashMap<K, DepthFirstNumber>,
    nodes: Vec<Node<K, V>>,
}

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub(super) struct DepthFirstNumber {
    index: usize,
}

pub(super) struct Node<K, V> {
    pub(crate) goal: K,

    pub(crate) solution: V,

    /// This is `Some(X)` if we are actively exploring this node, or
    /// `None` otherwise.
    pub(crate) stack_depth: Option<StackDepth>,

    /// While this node is on the stack, this field will be set to
    /// contain our own depth-first number. Once the node is popped
    /// from the stack, it contains the DFN of the minimal ancestor
    /// that the table reached (or MAX if no cycle was encountered).
    pub(crate) links: Minimums,
}

impl<K, V> SearchGraph<K, V>
where
    K: Hash + Eq + Debug + Clone,
    V: Debug + Clone,
{
    pub(crate) fn new() -> Self {
        SearchGraph {
            indices: FxHashMap::default(),
            nodes: vec![],
        }
    }

    pub(crate) fn lookup(&self, goal: &K) -> Option<DepthFirstNumber> {
        self.indices.get(goal).cloned()
    }

    /// Insert a new search node in the tree. The node will be in the initial
    /// state for a search node:
    ///
    /// - stack depth as given
    /// - links set to its own DFN
    /// - solution is initially an identity substitution for coinductive goals
    ///   or `NoSolution` for other goals
    pub(crate) fn insert(
        &mut self,
        goal: &K,
        stack_depth: StackDepth,
        solution: V,
    ) -> DepthFirstNumber {
        let dfn = DepthFirstNumber {
            index: self.nodes.len(),
        };
        let node = Node {
            goal: goal.clone(),
            solution,
            stack_depth: Some(stack_depth),
            links: Minimums { positive: dfn },
        };
        self.nodes.push(node);
        let previous_index = self.indices.insert(goal.clone(), dfn);
        assert!(previous_index.is_none());
        dfn
    }

    /// Clears all nodes with a depth-first number greater than or equal `dfn`.
    #[instrument(level = "debug", skip(self))]
    pub(crate) fn rollback_to(&mut self, dfn: DepthFirstNumber) {
        self.indices.retain(|_key, value| *value < dfn);
        self.nodes.truncate(dfn.index);
    }

    /// Removes all nodes with a depth-first-number greater than or
    /// equal to `dfn`, adding their final solutions into the cache.
    #[instrument(level = "debug", skip(self, cache))]
    pub(crate) fn move_to_cache(&mut self, dfn: DepthFirstNumber, cache: &Cache<K, V>) {
        self.indices.retain(|_key, value| *value < dfn);
        for node in self.nodes.drain(dfn.index..) {
            assert!(node.stack_depth.is_none());
            assert!(node.links.positive >= dfn);
            debug!("caching solution {:#?} for {:#?}", node.solution, node.goal);
            cache.insert(node.goal, node.solution);
        }
    }
}

impl<K, V> Index<DepthFirstNumber> for SearchGraph<K, V>
where
    K: Hash + Eq + Debug + Clone,
    V: Debug + Clone,
{
    type Output = Node<K, V>;

    fn index(&self, table_index: DepthFirstNumber) -> &Node<K, V> {
        &self.nodes[table_index.index]
    }
}

impl<K, V> IndexMut<DepthFirstNumber> for SearchGraph<K, V>
where
    K: Hash + Eq + Debug + Clone,
    V: Debug + Clone,
{
    fn index_mut(&mut self, table_index: DepthFirstNumber) -> &mut Node<K, V> {
        &mut self.nodes[table_index.index]
    }
}

impl DepthFirstNumber {
    pub(crate) const MAX: DepthFirstNumber = DepthFirstNumber { index: usize::MAX };
}

impl Add<usize> for DepthFirstNumber {
    type Output = DepthFirstNumber;

    fn add(self, v: usize) -> DepthFirstNumber {
        DepthFirstNumber {
            index: self.index + v,
        }
    }
}
