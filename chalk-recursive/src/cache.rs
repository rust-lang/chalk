use crate::UCanonicalGoal;
use chalk_ir::interner::Interner;
use chalk_ir::Fallible;
use chalk_solve::Solution;
use rustc_hash::FxHashMap;
use std::sync::{Arc, Mutex};
use tracing::debug;
use tracing::instrument;
/// The "cache" stores results for goals that we have completely solved.
/// Things are added to the cache when we have completely processed their
/// result, and it can be shared amongst many solvers.
pub struct Cache<I: Interner> {
    data: Arc<Mutex<CacheData<I>>>,
}
struct CacheData<I: Interner> {
    cache: FxHashMap<UCanonicalGoal<I>, Fallible<Solution<I>>>,
}

impl<I: Interner> Cache<I> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a cache result.
    #[instrument(skip(self))]
    pub fn insert(&self, goal: UCanonicalGoal<I>, result: Fallible<Solution<I>>) {
        let mut data = self.data.lock().unwrap();
        data.cache.insert(goal, result);
    }

    /// Record a cache result.
    pub fn get(&self, goal: &UCanonicalGoal<I>) -> Option<Fallible<Solution<I>>> {
        let data = self.data.lock().unwrap();
        if let Some(result) = data.cache.get(&goal) {
            debug!(?goal, ?result, "Cache hit");
            Some(result.clone())
        } else {
            debug!(?goal, "Cache miss");
            None
        }
    }
}

impl<I: Interner> Clone for Cache<I> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<I: Interner> Default for Cache<I> {
    fn default() -> Self {
        Self {
            data: Default::default(),
        }
    }
}

impl<I: Interner> Default for CacheData<I> {
    fn default() -> Self {
        Self {
            cache: Default::default(),
        }
    }
}
