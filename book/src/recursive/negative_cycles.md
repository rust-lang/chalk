# Negative cycles

Everything so far has been focused solely on *positive goals*. But we also
support *negative goals*, like `not { u32: X }`. Proving negative goals is
tricky for a number of reasons, but in this section we're going to focus just on
the aspects that have to do with cycle handling.

## Ordinary cycles are monotonic

In an [ordinary, positive cycle][cycle], we take advantage of the fact that the
result is "monotonic". Monotonic basically means "moving in one direction" -- in
particular, towards "more answers". The idea is that if you have a goal G1 that
depends on another goal G2 through a program clause like so:

```notrust
G1 :- ..., G2, ...
```

then if you get **more answers** for G2, you can only get **more answers** for
G1. You can't get **less**. For example, if before you assumed that G2 had zero
answers ("error"), then we would've assumed that this program clause was
unprovable (because it requires G2 to be true), and hence it contributed zero
answers to G1. But if we then find an answer for G2, now the program clause may
be provable, and we may have one more possible answer for G1.

This monotonic nature is what allows us to iterate until we reach a fixed point.
Each time through the cycle, we potentially add more answers. In our case, we
quickly reach a result of "ambiguity" ("non-unique answer") and hence the cycle
stops relatively quickly, but conceptually we could be tracking individual
answers.

[cycle]: inductive_cycles.md

## Negative goals are not monotonic

In contrast, a negative goal is not monotonic. Consider a program clause like
the following:

```notrust
G1 :- ..., not { G2 }, ...
```

Here, the relationship is inverted: this clause can only be proven if G2 has
**no answers**. So if we find **more answers** for G2, we may find **less
answers** for G1.

This complicates cycle handling. We can't just iterate in a loop and expect
things to converge on a final answer. The loop could go forever. Consider
a very simple example fo a negative cycle, like this one:

```notrust
G1 :- not { G1 }
```

If we use our normal technique, we start by assuming G1 is not provable. Then we
are able to prove G1 is true as a result. But when we iterate again, we find
that because G1 is true, the program clause is not provable, and hence we get an
error. This would repeat indefinitely.

## Negative cycles don't have a true/false answer

The previous example highlights another interesting facet of negative cycles.
They may not have a true/false answer. Consider the simple self-cycle:

```notrust
G1 :- not { G1 }
```

We can't use this rule to justify that G1 is *true*, since if G1 were true, then
`not { G1 }` is false. But if we say that G1 is *false*, then we are in a
contradiction, as we can prove that G1 is true using this rule (since `not {
G1}` is true). The solution to this problem is to use a "three-valued" notion of
truth, where a result can be *true*, *false*, or *ambiguous*. Ambiguous
results arise precisely from negative cycles like this one.

Note that chalk actually uses the ambiguous result for a few distinct things: we
use it for "multiple, distinct true results" as well as "neither true nor
false", as well as "unknowable" (e.g., for non-enumerable traits).

So when can we say that things are true/false/ambiguous *exactly*? The formal
definition is given in the [Well-founded Semantics][WFS]. The intuition though
is fairly simple: we can say that something is *true* if we can find some
inference rule that justifies that, where each clause in the rule is also
considered *true* (by some finite derivation). We say that something is *false*
if we can show that, for every applicable clause, there is at least one subgoal
in the clause that is *false*. (The base case here is that any goal with no
applicable clauses is false.) Everything else is ambiguous.

[WFS]: https://en.wikipedia.org/wiki/Well-founded_semantics

## Not all negative cycles are ambiguous

There are negative cycles where we can still conclude a result. Consider
this example:

```notrust
G1 :- not { G2 }, G3.
G2 :- not { G1 }.
```

There is a negative cycle here between `G1` and `G2`. Nonetheless, we can say
that `G1` is false and `G2` is true. Why is that?

* To start, `G3` is clearly false, as there is no rule that could make it true.
* Since `G3` is false, `G1` must be false.
* Since `G1` is false, `not { G1 }` is true, and hence `G2` is true.

You can see in this example some sense for how the [Well-Founded Semantics][WFS]
definition works: you iteratively build up an idea of which clauses are
true/false, starting from trivial negative cases like G3 (where there are no
applicable rules) or trivial positive cases where there is a rule with no
subgoals. Then you can iteratively add more goals, depending on whether you can
conclusively prove the goal true/false based on what you know so far. When
you're done, anything you haven't proven to be true/false is ambiguous. (To be
clear, this is a mathematical definition, which means that it builds up
potentially infinite sets of true/false answers. Our job as a logic solver is to
figure out whether any given goal would be in one of those infinite sets or
not.)

## Negative goals require "stratification"

Ultimately handling negative goals requires "stratification", which means
"forming layers". Put another way, we can only compute the result for something
like `not { G2 }` once we know the final result for `G2` (not the "intermediate"
results that we compute during iteration). Now, so long as there is no cycle,
stratification happens normally as a by-product of how the recursive solver (or
any solver, realy) works. But once a cycle arises, things get a bit trickier.

As we describe in the next section, what we do in the case of a cycle is
basically to treat the result as ambiguous and then see if we can prove/refute
the parent goal some other way. If we can, then the parent is actually in a
different strata from the child goal (in particular, the result of the child
goal depends on the parent goal, but not the inverse).

## Intuition for how the recursive solver handles negative goals

Let's return to our previous example and talk through how the recursive
solver would approach the goal `G1`:

```notrust
G1 :- not { G2 }, G3.
G2 :- not { G1 }.
```

In trying to prove `G1`, we will first consider the subgoal `not { G2 }`. This
requires trying to prove `G2`, which in turn leads to the subgoal `not { G1 }`.
This is a negative cycle, so we will flag `G1` as being a cyclic goal, and
return an ambiguous result. Thus the goal `G2` is considered to be "ambiguous"
for now. Since the goal `G2` was involved in a cycle, though, that result is not
cached, but it does remain in the search graph.

At this point, we return to trying to prove `G1`. We tried to prove `not { G2 }`
and got back an ambiguous result, so now we try to prove `G3`. That will
actually give an error, because there are no applicable program clauses. Since we know
`G3` is in error, we can return an error as the overall result (this result will also
be cached, because it was not a participant in any cycle).

Now, `G1` was the head of a cycle, so we will actually iterate again (though in
truth it's not necessary), but we will just wind up with the same result. At
this point, `G1` has reached a fixed-point result of "error". We return that
result. At the same time, we also move all results in the search graph to the
cache -- *except* for those results were "negatively dependent" on some parent
goal. In particular, the result for `G2` (ambiguous) is not cached, but the
result for `G1` (error) is.

The fact that `G2` is not cached is crucial. If it were cached, and we were
later to try to prove `G2`, we would encounter this cached result and give back
ambiguous, but that is not the correct result -- now that we know that `G1` is
in error, the correct result is that `G2` is true. And indeed, since `G2` is not
yet cached, if we do try to solve it, things will go differently this time.

If, after we try to prove `G1` and get back error, we later try to prove `G2`,
we need to prove `not { G1 }`. Thus we recursively try to prove `G1`, which
encounters a cached result (error). Because we hit a cached result, there is no
negative cycle encountered. This `G1` is not provable, we conclude that `not {G1
}` is true, and hence `G2` is true.

You can see that the "cache" here plays the role of the "sets of true/false
facts" that we talked about in the mathematical WFS definition. We also use the
cache to define the "strata" between goals. Once something is cached, it means
that we were able to compute a final result that was dependent purely on things
already cached (and hence on a lower strata) or which is independent of any
negative cycle (because negative cycles always give ambiguous results). Anything
that we can then prove based on that cached reuslt must be in some other strata.

## An example of a negative cycle that is ambiguous

It's interesting to note that the previous example was only provable
because the goal `G3` had no applicable clauses. Consider this variation:

```notrust
G1 :- not { G2 }, G3.
G2 :- not { G1 }.
G3.
```

Here, the goal `G3` can be proven -- this means that both `G1` and `G2` are
ambiguous and can never be proven true or false. How would that play out
in our solver?

* To start, proving G1 would find that `not { G2 }` is ambiguous (as before),
  and `G3` is true. That means the overall result is ambiguous, which would be
  cached. As before, the result for `G2` would not be cached, as it is a
  participant in a negative cycle.
* If we go to recompute `G2`, this time we would find that `G1` is ambiguous,
  and hence `not { G1 }` is also ambiguous.

It's somewhat interesting to see why ambiguity arises here. The problem is
that there are multiple consistent sets of answers in this scenario, and there
is no good way to choose between them.

* We could say that `G2` is *false*, in which case `G1` would be true.
* We could say that `G1` is *false*, in which case `G2` would be true.

These results would be "consistent", but we have no basis on which to say
that `G1` or `G2` is false, and no real way to choose which of those two
answers is "correct". Hence the result is considered ambiguous. 

(There are alternative formulations, such as [Answer Set Programming][ASP], that
can accommodate situations like this and give back the full set of
possibilities.)

[ASP]: https://en.wikipedia.org/wiki/Answer_set_programming

## An example of a negative cycle that is provable

The only way to "positively prove" something involving a negative cycle is for
it to have multiple clauses, such that one clause may be ambiguous due to a
cycle, but other clauses are fully provable, as in this example:

```notrust
G1 :- not { G2 }, G3.
G1.
G2 :- not { G1 }.
```

Here we have a negative cycle for `G1`, but only through one of the two
available clauses. The other clause will give back a positive result. Therefore,
we can compute `G1` as "true", and in turn conclude that `G2` must be "false".

## Detailed notes on how the recursive solver handles negative goals

To handle negative goals we:

* Track the minimum depth-first number of any goal that we depended on
  negatively; if we try to refute a goal `G1`, and the result had a minimum
  result of `(P1, N1)`, where `P1` is the minimum positive depth and `N1` the
  minimum negative depth, then we would update our own negative minimum `N0` to
  be the minimum of `(N0, P1, N1)`.
* When solving a negative subgoal `not { G1 }`, we try to recursively prove `G1`.
  If `G1` has a dependency on something up the stack, return ambiguous always.
* When caching results at the end of iteration, forego caching any result for
  some goal with DFS number `D` that was negatively dependent on some other goal
  `D1` where `D1 < D` (i.e., negatively dependent on something further up the
  stack). This goal `D` must be computed in a second strata.
  * We could eagerly recompute `D`, but why.
