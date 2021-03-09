use ::std::cmp::{max, min};
use chalk_ir::{interner::Interner, Canonical, ConstrainedSubst, Constraints, Fallible};
use chalk_solve::Solution;
use rustc_hash::FxHashMap;
use std::collections::BinaryHeap;
use tracing::debug;

use crate::{
    search_graph::{DepthFirstNumber, SearchGraph},
    Minimums, UCanonicalGoal,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct CoinductiveCycleDependencyBoundaries {
    pub(crate) lower: DepthFirstNumber,
    pub(crate) upper: DepthFirstNumber,
}

impl CoinductiveCycleDependencyBoundaries {
    pub(crate) fn update_from(&mut self, other_boundaries: CoinductiveCycleDependencyBoundaries) {
        self.lower = min(self.lower, other_boundaries.lower);
        self.upper = max(self.upper, other_boundaries.upper);
    }

    pub(crate) fn eq_singular(&self, singular: DepthFirstNumber) -> bool {
        self.lower == singular && self.upper == singular
    }

    pub(crate) fn new(singular: DepthFirstNumber) -> Self {
        CoinductiveCycleDependencyBoundaries {
            lower: singular,
            upper: singular,
        }
    }
}

#[derive(Debug)]
pub(crate) struct PrematureResult<I: Interner> {
    result: Fallible<Solution<I>>,

    /// The outermost and innermost cycles on which this result depends.
    dependency: CoinductiveCycleDependencyBoundaries,
}

pub(crate) struct CoinductionHandler<I: Interner> {
    /// Heap of start DFNs for nested cycles.
    /// The innermost cycle start is the first element in the heap.
    cycle_start_dfns: BinaryHeap<DepthFirstNumber>,

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
        self.cycle_start_dfns.peek().copied()
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
            if self.cycle_start_dfns.iter().any(|start| start == &dfn) {
                minimums.update_coinductive_cycle_boundaries(
                    CoinductiveCycleDependencyBoundaries::new(dfn),
                );
                return Some(Ok(Self::generate_assumption(goal, interner)));
            }
        }

        self.temp_cache
            .get(goal)
            .map(|PrematureResult { result, dependency }| {
                minimums.update_coinductive_cycle_boundaries(*dependency);
                result.clone()
            })
    }

    /// Handles results inside coinductive cycles.
    /// If the result is mature, it is moved to the standard cache.
    /// Else if the result belongs to a coinductive cycle start
    /// the cycle is finished. If neither of these apply,
    /// the result is stored in the temporary cache.
    pub fn handle_coinductive_result(
        &mut self,
        dfn: DepthFirstNumber,
        standard_cache: &mut FxHashMap<UCanonicalGoal<I>, Fallible<Solution<I>>>,
        search_graph: &mut SearchGraph<I>,
        minimums: &mut Minimums,
    ) {
        if minimums.coinductive_cycle_boundaries.is_none() {
            // If the result is mature, it can be cached directly in the standard cache.
            search_graph.move_to_cache(dfn, standard_cache, move |result| result);
        } else if let Some(start_dfn) = self.get_current_cycle_start() {
            if dfn == start_dfn {
                // If the handled result belongs to the current innermost cycle start
                // this cycle can be finished.
                self.finish_cycle(standard_cache, search_graph, minimums);
            } else {
                search_graph.move_to_cache(dfn, &mut self.temp_cache, move |result| {
                    PrematureResult {
                        result,
                        dependency: minimums.coinductive_cycle_boundaries.unwrap(),
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
        standard_cache: &mut FxHashMap<UCanonicalGoal<I>, Fallible<Solution<I>>>,
        search_graph: &mut SearchGraph<I>,
        minimums: &Minimums,
    ) {
        if let Some(start_dfn) = self.cycle_start_dfns.pop() {
            if let Some(minimum_boundaries) = minimums.coinductive_cycle_boundaries {
                if minimum_boundaries.eq_singular(start_dfn) {
                    // The temporarily cached results from inside the current coinductive cycle
                    // can be moved to the standard cache iff the assumption at the cycle start
                    // holds and is itself not dependent on any assumption.
                    if search_graph[start_dfn].solution.is_ok() {
                        for (goal, PrematureResult { result, dependency }) in
                            self.temp_cache.iter_mut()
                        {
                            if dependency.eq_singular(start_dfn) {
                                standard_cache.insert(goal.clone(), result.clone());
                            }
                        }
                    }

                    search_graph.move_to_cache(start_dfn, standard_cache, |result| result);
                } else {
                    // If the cycle start is itself dependent on another coinductive cycle assumption,
                    // its result is also premature and can't be decided yet.
                    // All results from inside the current cycle are invalidated as the precise
                    // dependency on the outer cycle is not tracked.
                    search_graph.rollback_to(start_dfn);
                }

                // Remove all moved or invalidated results (i.e. still dependent on the finished cycle).
                self.temp_cache.retain(
                    |_k,
                     PrematureResult {
                         result: _,
                         dependency,
                     }| {
                        dependency.lower != start_dfn && dependency.upper != start_dfn
                    },
                );

                debug!(
                    "Coinductive cycle finished. Remaining temporary cache: {:?}",
                    self.temp_cache
                );
            }
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
            cycle_start_dfns: BinaryHeap::new(),
            temp_cache: FxHashMap::default(),
        }
    }
}
