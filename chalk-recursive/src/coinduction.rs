use chalk_ir::{interner::Interner, Canonical, ConstrainedSubst, Constraints, Fallible};
use chalk_solve::Solution;
use rustc_hash::{FxHashMap, FxHashSet};
use tracing::debug;

use crate::{
    search_graph::{DepthFirstNumber, SearchGraph},
    Minimums, UCanonicalGoal,
};

#[derive(Debug)]
pub(crate) struct PrematureResult<I: Interner> {
    result: Fallible<Solution<I>>,

    /// All coinductive cycle assumptions this result depends on.
    /// Must not be empty.
    dependencies: FxHashSet<DepthFirstNumber>,
}

pub(crate) struct CoinductionHandler<I: Interner> {
    /// Stack of cycle start DFNs for nested cycles.
    cycle_start_dfns: Vec<DepthFirstNumber>,

    /// Temporary cache for premature results with
    /// corresponding cycle start DFNs.
    temp_cache: FxHashMap<UCanonicalGoal<I>, PrematureResult<I>>,
}

impl<I: Interner> CoinductionHandler<I> {
    pub fn start_cycle(&mut self, start_dfn: DepthFirstNumber) {
        self.cycle_start_dfns.push(start_dfn);
    }

    pub fn in_coinductive_cycle(&self) -> bool {
        !self.cycle_start_dfns.is_empty()
    }

    pub fn get_current_cycle_start(&self) -> Option<DepthFirstNumber> {
        self.cycle_start_dfns.last().copied()
    }

    /// Get a cached result from the temporary cache
    /// or an assumption if the requested goal corresponds
    /// to the start of a coinductive cycle.
    pub fn get_assumption_or_cached(
        &mut self,
        goal: &UCanonicalGoal<I>,
        dfn: Option<DepthFirstNumber>,
        minimums: &mut Minimums,
        interner: &I,
    ) -> Option<Fallible<Solution<I>>> {
        if dfn.is_some() && self.cycle_start_dfns.contains(&dfn.unwrap()) {
            minimums.add_cycle_start(dfn.unwrap());
            Some(Ok(Self::generate_assumption(goal, interner)))
        } else {
            self.temp_cache.get(goal).map(
                |PrematureResult {
                     result,
                     dependencies,
                 }| {
                    minimums.add_cycle_starts(dependencies);
                    result.clone()
                },
            )
        }
    }

    pub fn handle_coinductive_result(
        &mut self,
        dfn: DepthFirstNumber,
        cache: &mut FxHashMap<UCanonicalGoal<I>, Fallible<Solution<I>>>,
        search_graph: &mut SearchGraph<I>,
        minimums: &mut Minimums,
    ) {
        if minimums.is_mature() {
            // If the result is mature, it can be directly cached in the standard cache.
            search_graph.move_to_cache(dfn, cache, move |result| result);
        } else if let Some(start_dfn) = self.get_current_cycle_start() {
            if dfn == start_dfn {
                // If the handled result belongs to the current innermost cycle
                // this cycle can be finished.
                minimums.coinductive_cycle_starts.remove(&dfn);
                self.finish_cycle(cache, search_graph, minimums);
            } else {
                search_graph.move_to_cache(dfn, &mut self.temp_cache, move |result| {
                    PrematureResult {
                        result,
                        dependencies: minimums.coinductive_cycle_starts.clone(),
                    }
                });
            }
        }
    }

    fn finish_cycle(
        &mut self,
        cache: &mut FxHashMap<UCanonicalGoal<I>, Fallible<Solution<I>>>,
        search_graph: &mut SearchGraph<I>,
        minimums: &Minimums,
    ) {
        if let Some(start_dfn) = self.cycle_start_dfns.pop() {
            if search_graph[start_dfn].solution.is_ok() {
                for (
                    goal,
                    PrematureResult {
                        result,
                        dependencies,
                    },
                ) in self.temp_cache.iter_mut()
                {
                    dependencies.remove(&start_dfn);
                    if dependencies.is_empty() {
                        // The result has no pending dependencies anymore and can be moved to the standard cache.
                        cache.insert(goal.clone(), result.clone());
                    }
                }
            }
            if minimums.is_mature() {
                search_graph.move_to_cache(start_dfn, cache, |result| result);
            } else {
                search_graph.move_to_cache(start_dfn, &mut self.temp_cache, move |result| {
                    PrematureResult {
                        result,
                        dependencies: minimums.coinductive_cycle_starts.clone(),
                    }
                });
            }

            self.temp_cache.retain(
                |_k,
                 PrematureResult {
                     result: _,
                     dependencies,
                 }| !(dependencies.is_empty() || dependencies.contains(&start_dfn)), // Remove all moved or invalidated results.
            );
        }
        debug!(
            "Coinductive cycle finished. Caches: {:?} {:?}",
            cache, self.temp_cache
        );
    }

    fn generate_assumption(goal: &UCanonicalGoal<I>, interner: &I) -> Solution<I> {
        Solution::Unique(Canonical {
            value: ConstrainedSubst {
                subst: goal.trivial_substitution(interner),
                constraints: Constraints::empty(interner),
            },
            binders: goal.canonical.binders.clone(),
        })
    }
}

impl<I: Interner> Default for CoinductionHandler<I> {
    fn default() -> Self {
        CoinductionHandler {
            cycle_start_dfns: Vec::new(),
            temp_cache: FxHashMap::default(),
        }
    }
}
