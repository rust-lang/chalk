# Coinduction

This sub-chapter is meant to describe the current handling of coinductive goals in the recursive solver rather than providing an extensive overview over the theoretical backgrounds and ideas.
It follows the description in [this GitHub comment](https://github.com/rust-lang/chalk/issues/399#issuecomment-643420016) and the Zulip topic linked there.
In general, coinductive cycles can arise for well-formedness checking and autotraits.
Therefore, correctly handling coinductive cycles is necessary to model the Rust trait system in its entirety.

## General Idea
Coinductive cycles can be handled the same way as inductive cycles described [before](./inductive_cycles.md).
The only difference is the start value for coinductive goals.
Whereas inductive goals start with a negative result and are iterated until a least fixed-point is found, coinductive goals start with a positive result (i.e. a unique solution with identity substitution).
This negative result is then iterated until a greatest fixed-point is reached.

## Mixed co-inductive and inductive Cycles
As described above, the handling of inductive and coindutive cycles differs only in the start value from which the computation begins.
Thus, it might seem reasonable to have mixed inductive and coinductive cycles as all goals inside these cycles would be handled the same way anyway.
Unfortunately, this is not possible for the kind of logic that Chalk is based on (i.e. essentially an extension of co-LP for Hereditary Harrop clauses, cf. [this paper][co-LP]).

There is fundamental difference between results for inductive cycles and results for coinductive cycles of goals.
An inductive goal is provable if and only if there exists a proof for it consisting of a finite chain of derivations from axioms that are members of the least-fixed point of the underlying logic program.
On the other hand, coinductive goals are provable if there exists an at most infinite derivation starting from the axioms that proves it (this includes in particular all finite derivations).
This infinite derivation is then part of the greatest fixed-point of the logic program.
As infinite derivations are not feasible to compute, it is enough to show that such a derivation contains no contradiction.

A simple example `X :- X.` (with `X` a free variable) is thus not provable by inductive reasoning (the least solution/lfp for this is the empty solution, a failure) but it is provable by coinductive reasoning (the greatest solution/gfp is the universe, i.e. all values).

This difference between inductive and coinductive results becomes a problem when combined in a single cycle.
Consider a coinductive goal `CG` and an inductive goal `IG`. Now consider the simplest possible mixed cycle:
```notrust
CG :- IG
IG :- CG
```
It is apparent, that there can not exist a solution for `IG` as the cyclic dependency prevents a finite proof derivation.
In contrast to that, `CG` could potentially be provable as the derivation *`CG` if `IG` if `CG` if `IG` ...* is infinite and based only on the two axioms.
As a result, `CG` would hold whereas `IG` would not hold, creating a contradiction.

The simplest solution to this problem, proposed by Simon et al. in [their paper about co-LP][co-LP], is to disallow mixed inductive and coinductive cycles.
This approach is also used by Chalk.

## Prevention of Invalid Results
The problem of invalid results propagated outside of the coinductive cycle is also described in the [Coinduction chapter](../engine/logic/coinduction.md) for the SLG solver alongside the rather complex handling used with it.
Whereas the SLG solver introduces [special constructs](../engine/logic/coinduction.html#nikos-proposed-solution) to handle coinduction, it is sufficient for the recursive solver to use the same logic for inductive and coinductive cycles.
The following is a description of how this works in more detail.

### The Problem
The problem arises if a solution that is purely based on the positive starting value for the coinductive cycle is cached (or tabled in logic programming terms) and as such propagated to other goals that are possibly reliant on this. An example where all clause goals are assumedly coinductive may look like this (cf. the test case `coinduction::coinductive_unsound1`):

```notrust
C :- C1.
C :- C2.
C1 :- C2, C3.
C2 :- C1.
```
The following is a computation to find out whether there exists a type that implements `C`.
Here the implementation of `C` may be proved by either showing that the type implements `C1` or `C2`.
* Start proving `C` by trying to prove `C1`:
    * For `C1` try to prove `C2` and `C3`:
        * Start with `C2`. For `C2` we need to prove `C1`:
            * This is a (coinductive) cycle. Assume that `C1` holds, i.e. use the positive start value.
        * Based on this `C2` also holds. If this case is not handled specifically, the solution for `C2` is cached without a reference to the solution for `C1` on which it depends.
        * Now try to prove `C3`:
            * Find that there is no way do so from the given axioms.
        * Thus, there exists no solution for `C3` and the computation fails. This valid result is cached and lifted back up.
    * Due to the failure of `C3` there is also no solution for `C1`. This failure is also cached correctly and lifted back up. The cached solution for `C2` has now become invalid as it depends on a positive result for `C1`.
* As a result of the failure for `C1`, `C` can not be proved from `C1`. Try proving `C` from `C2` instead:
    * Find the cached result that `C2` has a solution and lift it back up.
* Due to the solution for `C2`, `C` is also proved with the same solution.
* Stop with this positive but invalid result for `C`.

### The Solution
The above example should make it evident that the caching of found solutions in coinductive cycles can lead to invalid results and should therefore be prevented.
This can be achieved by delaying the caching of all results inside the coinductive cycle until it is clear whether the start of the cycle (i.e. `C1` in the example above) is provable (cf. the handling of inductive cycles [before](./inductive_cycles.md)).
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
        * Find the already cached negative result for `C3`.
    * Nothing changed regarding `C1` (this would indicate a negative cycle which is currently  not allowed) and the negative result for `C1` and `C2` are cached. Lift the negative result for `C1` back up.
* Start proving `C` with `C2`:
    * Find negative cached result for `C2`. Lift the result back up.
* Neither `C1` nor `C2` have a positive result. Stop with the valid disproof of `C`.


[co-LP]: https://link.springer.com/chapter/10.1007%2F978-3-540-73420-8_42