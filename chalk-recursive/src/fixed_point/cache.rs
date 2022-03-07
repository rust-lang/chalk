use rustc_hash::FxHashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use tracing::debug;
use tracing::instrument;
/// The "cache" stores results for goals that we have completely solved.
/// Things are added to the cache when we have completely processed their
/// result, and it can be shared amongst many solvers.
pub struct Cache<K, V>
where
    K: Hash + Eq + Debug,
    V: Debug + Clone,
{
    data: Arc<Mutex<CacheData<K, V>>>,
}
struct CacheData<K, V>
where
    K: Hash + Eq + Debug,
    V: Debug + Clone,
{
    cache: FxHashMap<K, V>,
}

impl<K, V> Cache<K, V>
where
    K: Hash + Eq + Debug,
    V: Debug + Clone,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a cache result.
    #[instrument(skip(self))]
    pub fn insert(&self, goal: K, result: V) {
        let mut data = self.data.lock().unwrap();
        data.cache.insert(goal, result);
    }

    /// Record a cache result.
    pub fn get(&self, goal: &K) -> Option<V> {
        let data = self.data.lock().unwrap();
        if let Some(result) = data.cache.get(goal) {
            debug!(?goal, ?result, "Cache hit");
            Some(result.clone())
        } else {
            debug!(?goal, "Cache miss");
            None
        }
    }
}

impl<K, V> Clone for Cache<K, V>
where
    K: Hash + Eq + Debug,
    V: Debug + Clone,
{
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<K, V> Default for Cache<K, V>
where
    K: Hash + Eq + Debug,
    V: Debug + Clone,
{
    fn default() -> Self {
        Self {
            data: Default::default(),
        }
    }
}

impl<K, V> Default for CacheData<K, V>
where
    K: Hash + Eq + Debug,
    V: Debug + Clone,
{
    fn default() -> Self {
        Self {
            cache: Default::default(),
        }
    }
}
