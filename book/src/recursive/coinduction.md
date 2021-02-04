# Coinduction

This sub-chapter is meant to describe the current handling of coinductive goals in the recursive solver rather than providing an extensive overview over the theoretical backgrounds and ideas.
It follows the description in [this GitHub comment](https://github.com/rust-lang/chalk/issues/399#issuecomment-643420016) and the Zulip topic linked there.

## General Idea
The general idea for the handling of coinductive cycles in the recursive solver is to start by assuming the goal is provable and then try to find evidence that it is not.
This search for a disproof is done by the standard recursive solving process described in the sub-chapters before.

Albeit this approach would allow for the handling of mixed inductive/co-inductive cycles, these are actually handled as errors to prevent propagation of the assumed provability outside of the coinductive cycle.
This propagation of the assumed solution might also happen in pure coinductive cycles and can potentially lead to invalid results.

## Prevention of Invalid Results
The problem of invalid results propagated outside of the coinductive cycle is also described in the [Coinduction chapter](../engine/logic/coinduction.md) for the SLG solver alongside the rather complex handling used with it.

### The Problem
The problem arises if a solution that is purely based on the positive starting value for the coinductive cycle is cached and as such propagated to other goals that are possibly reliant on this. An example may look like this (cf. the test case `coinduction::coinductive_unsound`):

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
The above example should make it evident that the caching of found solutions in coinductive cycles can lead to invalid results and should therefore be prevented.
This can be achieved by delaying the caching of all results inside the coinductive cycle until it is clear whether the start of the cycle (i.e. `C1` in the example above) is provable.
If the start of the cycle can be proven by the results of the cycle and related subgoals then the assumption about it was correct and thus all results for goals inside the cycle are also valid.
If, however, the start of the cycle can not be proved, i.e. the initial assumption was false, then a subset of the found solutions for the coinductive cycle may be invalid (i.e. the solution for `C2` in the example).

To remove such invalid results, the cycle is restarted with a negative result for the cycle start.
With this approach, it is possible to remove all invalid result that would otherwise depend on the disproved cycle assumption.
To allow for the cycle to be restarted correctly, all nodes in the search graph after the cycle start are deleted.

With this procedure, the example is handled as follows:
* Start proving `C` with `C1`:
    * For `C1` prove `C2` and `C3`:
        * For `C2` prove `C1`:
            * This is a coinductive cycle. Assume that `C1` holds.
        * Thus `C2` also holds. Delay the caching of the result about `C2`.
        * There is no way to prove `C3`. Cache this result and lift the failure up.
    * Due to the failure of `C3` there is also no solution for `C1`. Set `C1` to a negative result and restart the cycle.
        * For `C2` prove `C1`:
            * `C1` has now a negative result.
        * Thus, `C2` also has a negative result which is not yet cached.
        * The result for `C3` is already cached.
    * Nothing changed regarding `C1` (this would indicate a negative cycle which is currently  not allowed) and the negative result for `C1` and `C2` are cached. Lift negative result for `C1`.
* Start proving `C` with `C2`:
    * Find negative cached result for `C2`. Lift the result back up.
* Neither `C1` nor `C2` have a positive result. Stop with the valid disproof of `C`.
