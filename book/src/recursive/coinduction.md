# Coinduction

This sub-chapter is meant to describe the current handling of coinductive goals in the recursive solver rather than providing an extensive overview over the theoretical backgrounds and ideas.
It follows the description in [this GitHub comment](https://github.com/rust-lang/chalk/issues/399#issuecomment-643420016) and the Zulip topic linked there.

## General Idea
The general idea for the handling of coinductive cycles in the recursive solver is to start by assuming the goal is provable and then try to find evidence that it is not.
This search for a disproof is done by the standard recursive solving process described in the sub-chapters before.

Albeit this approach would allow for the handling of mixed inductive/co-inductive cycles, these are actually handled as errors to prevent propagation of the assumed provability outside of the coinductive cycle.
This propagation of the assumed solution might also happen in pure coinductive cycles and can potentially lead to false positives.

## Prevention of False Positives
The problem of false positives propagated outside of the coinductive cycle is also described in the [Coinduction chapter](../engine/logic/coinduction.md) for the SLG solver alongside the rather complex handling used with it.

### The Problem
The problem arises if a solution that is purely based on the positive starting value for the coinductive cycle is cached and as such propagated to other goals that are possibly reliant on this. An example may look like this (cf. the test case `coinduction::coinductive_unsound1`):

```notrust
C :- C1.
C :- C2
C1 :- C2, C3.
C2 :- C1.
```

Here `C` may be proved by either showing `C1` or `C2`.
Assuming the solver starts evaluating the branch with `C1` first, it then recursively tries to prove `C2` and `C3`.
For proving `C2` it needs to show `C1` again, the coinductive cycle becomes evident.
Therefore, `C1` is assumed to be provable and the solver proves `C2` with this information.
Assuming, the solve does not handle this case specifically, the solution for `C2` is cached. 
Now it tries solving `C3` but fails due to the lack of information about it.
As such, `C1` can also not be proven for this program.
The recursive solver will now attempt to prove the initial goal `C` by solving `C2`.
Unfortunately, it finds the invalidly cached solution and returns it as proof for `C`.

By visualizing this path of computation, it becomes evident, where the problem lies:
* Start proving `C` with `C1`:
    * For `C1` prove `C2` and `C3`:
        * For `C2` prove `C1`:
            * This is a coinductive cycle. Assume that `C1` holds.
        * Thus `C2` also holds. Store this result about `C2` in the cache.
        * There is no way to prove `C3`. Lift this failure up.
    * Due to the failure of `C3` there is also no solution for `C1`.
* Try proving `C` with `C2`:
    * Find the cached result that `C2` has a solution and return it as the solution for `C`.
* Stop with the invalid result for `C`.

### The Solution
The above example should make it evident that the caching of found solutions in coinductive cycles can lead to false positives and should therefore be prevented.
This can be achieved by delaying the caching of all results inside the coinductive cycle until it is clear whether the start of the cycle (i.e. `C1` in the example above) is provable.
If the start of the cycle can be proven by the results of the cycle and related subgoals then the assumption about it was correct and thus all results for goals inside the cycle are also valid.
If, however, the start of the cycle can not be proved, i.e. the initial assumption was false, then a subset of the found solutions for the coinductive cycle may be invalid (i.e. the solution for `C2` in the example).
This subset can (probably) only consist of positive results, i.e. derived solutions that could reference the assumption.

Negative results should not be influenced by the positive assumption as refutability can not be followed from provability.
As a result, it is sound to over-approximate and remove all positive solutions if the start of the cycle is disproved.
Therefore, the recursive solver delays caching by relating all solutions in the cycle too its start (i.e. by adjusting the minimum in this subtree) and removes all positive results inside the coinductive cycle if and only if the start is disproven.
The only downside of this approach is the late caching which might lead to increased work for shared subgoals in the same proof sub-tree as the coinductive cycle.

With this procedure, the example is handled as follows:
* Start proving `C` with `C1`:
    * For `C1` prove `C2` and `C3`:
        * For `C2` prove `C1`:
            * This is a coinductive cycle. Assume that `C1` holds.
        * Thus `C2` also holds. Delay the caching of the result about `C2`.
        * There is no way to prove `C3`. Lift this failure up.
    * Due to the failure of `C3` there is also no solution for `C1`. Cache this result as well as the failure of `C3` but delete everything about `C2`.
* Start proving `C` with `C2`:
    * For `C2` prove `C1`:
        * Find the cached result that `C1` is disproved.
    * Follow that `C2` is also disproved and cache this result.
* Stop with the valid disproof of `C`.

### Other Concerns
Another possible concern is related to the [search graph](./search_graph.md).
As the solutions of the coinductive cycle are only moved to the cache after a certain delay, they are still part of the search graph and might influence other computations as such.
Fortunately, as long as the subtree with the coinductive cycle is not finished all solutions within the graph can be assumed to be valid. 
This follows directly from the initial assumption and the fact that every goal that is investigated by the solver can still contribute to the whole cycle's validity (errors are propagated upwards directly).
This is especially important as reoccurring nodes in the search graph could also denote inductive cycles.
Though, as all nodes from inside the coinductive cycle are directly related to its start, they are not treated as part of any other cycle and the computed solutions are still valid in this context. 
