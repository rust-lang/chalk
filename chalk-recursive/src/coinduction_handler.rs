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
        if let Some(dfn) = dfn {
            if self.cycle_start_dfns.contains(&dfn) {
                minimums.add_cycle_start(dfn);
                return Some(Ok(Self::generate_assumption(goal, interner)));
            }
        }

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

    /// Handles results inside coinductive cycles.
    /// If the result is mature, it is moved to the standard cache.
    /// Else if the result belongs to a coinductive cycle start
    /// the cycle is finished. If neither of these apply,
    /// the result is stored in the temporary cache.
    pub fn handle_coinductive_result(
        &mut self,
        dfn: DepthFirstNumber,
        cache: &mut FxHashMap<UCanonicalGoal<I>, Fallible<Solution<I>>>,
        search_graph: &mut SearchGraph<I>,
        minimums: &mut Minimums,
    ) {
        if minimums.is_mature() {
            // If the result is mature, it can be cached directly in the standard cache.
            search_graph.move_to_cache(dfn, cache, move |result| result);
        } else if let Some(start_dfn) = self.get_current_cycle_start() {
            if dfn == start_dfn {
                // If the handled result belongs to the current innermost cycle start
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

    /// Finish a cycle by either moving all dependent premature results to
    /// the standard cache or dropping them. Caches also the result for the start
    /// goal of the cycle.
    fn finish_cycle(
        &mut self,
        cache: &mut FxHashMap<UCanonicalGoal<I>, Fallible<Solution<I>>>,
        search_graph: &mut SearchGraph<I>,
        minimums: &Minimums,
    ) {
        if let Some(start_dfn) = self.cycle_start_dfns.pop() {
            if minimums.is_mature() {
                // The temporarily cached results from inside the current coinductive cycle
                // can be moved to the standard cache iff the assumption at the cycle start
                // holds and is itself not dependent on any assumption.
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

                search_graph.move_to_cache(start_dfn, cache, |result| result);
            } else {
                // If the cycle start is itself dependent on another coinductive cycle assumption
                // its result is also premature and needs to be put in the temporary cache.
                // All results from inside th current cycle are invalidated as the concrete
                // dependency on the outer cycle is not tracked.
                search_graph.move_to_cache(start_dfn, &mut self.temp_cache, move |result| {
                    PrematureResult {
                        result,
                        dependencies: minimums.coinductive_cycle_starts.clone(),
                    }
                });
            }

            // Remove all moved or invalidated results (i.e. still dependent on the finished cycle).
            self.temp_cache.retain(
                |_k,
                 PrematureResult {
                     result: _,
                     dependencies,
                 }| !(dependencies.is_empty() || dependencies.contains(&start_dfn)),
            );

            debug!(
                "Coinductive cycle finished. Remaining temporary cache: {:?}",
                self.temp_cache
            );
        }
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