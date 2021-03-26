# Coinduction

This sub-chapter was originally prepared for wg-traits design meeting on 2019-11-08 (see the [Hackmd](https://hackmd.io/OJRi5OM6Twunw8ZmuLxfRA) doc). It briefly covers some tricky (and previously incorrectly handled) cases of coinduction, as well as two proposed solutions. The resulting and current solution ended up being something *pretty* close to Niko's solution. However, this is basically a copy-paste from the original document, and so shouldn't necessarily be taken as 100% truth as far as implementation.

## The problem

See [chalk#248] for details. The short version is that we fail to handle a case like this correctly, where `Ci` are all co-inductive goals:

[chalk#248]: https://github.com/rust-lang/chalk/issues/248

```notrust
C1 :- C2, C3.
C2 :- C1.
```

What happens is that we 

* start to prove C1
* start to prove C2
* see a recursive attempt to prove C1, assume it is successful
* consider C2 proved **and cache this**
* start to prove C3, fail
* consider C1 **unproven**

Now we incorrectly have a result that `C2` is true -- but that result was made on the assumption that `C1` was true, and it was not.

## Some other tricky cases to consider

### Unification failures

One thing to consider is that even when we have "coinduction obligations" to prove, we have to remember their substitutions too:

```notrust
C1(X) :- C2(Y), X = 22.
C2(X) :- C3(X), X = 44.
C3(X) :- C1(X), C2(X).
```

None of these predicates should be provable,  because `C1(X)` and `C2(X)` don't hold for the same `X`.

If we're not careful, we might:

* start to prove C1
* start to prove C2
* start to prove C3, see the recursive calls to C1 and C2
    * maybe we wait to consider it proven until C1 and C2 complete

In this case, it's not enough that C1 and C2 are provable at all, they have to be provable for the same X.

### Non-trivial self-cycles

```notrust
C1(A) :- C1(B), B = 22, C2(A).
C2(44).
```

This case is not provable, even though the only cycle is `C1(X) :- C1(Y)` -- but it turns out that `X` must not be 22. The catch is that while this might *appear* to be a trivial self-cycle, it is not! 

Actually I have to think about the best way to handle this case, as my proposed solution doesn't quite cut it. It wouldn't be *wrong* but it seems not ideal. -- Niko

### Delayed trivial cycles

```notrust
C1(A, B) :- C2(A, B), A = 22, B = 22.
C2(A, B) :- C1(B, A).
```

This should be provable, but the cycle from C2 to C1 is not immediately visible as a trivial cycle, at least if subgoals are solved in order.


### Delayed trivial cycles, variant 2

```notrust
C1(A, B) :- C2(A, B), A = 22.
C2(A, B) :- C1(B, A).
```

As above, here the only complete answer is `C1(22, 22)`. This is because the `C1`, `C2` cycle effectively guarantees equality.

### Delayed trivial cycles, variant 3

```notrust
C1(A, B) :- C1(B, A).
```

This is true for all `A, B`

### Other cases?

## Approach in existing PR

### High-level idea

* When we encounter a co-inductive subgoal, we delay them in the current `Strand`
* When all subgoals have been tested, and there are remaining delayed co-inductive subgoals, this is propagated up, marking the current `Strand` as co-inductive
* When the co-inductive `Strand`s reach the root table, we only then pursue an answer

## Niko's proposed solution

### High-level idea

* We only consider a co-induction subgoal proven for *trivial* recursion -- i.e., self-recursion where you have `C1 :- C1`.
* For non-trivial recursion, we propagate the co-inductive subgoal to the parent. This continues until it becomes trivial.

### Implementation steps

**Extend `Answer` in two ways.**

Currently `Answer` has a "constrained substitution" that includes values for the table's substitution + region constraints:

```notrust
struct Answer {
    constrained_subst: Canonical<ConstrainedSubst>,
    is_ambiguous: bool
}

struct ConstrainedSubst {
    substitution: Substitution,
    region_constraints: Vec<RegionConstraint>,
}
```

we would first extend `ConstrainedSubst` to include as yet unproven co-inductive subgoals (this might actually be better done as a new type): 

```rust,ignore
struct ConstrainedSubst {
    substitution: Substitution,
    delayed_subgoals: Vec<Literal>,
    region_constraints: Vec<RegionConstraint>,
}
```

then we would extend `Answer` slightly as well so it can be "ok" or ambiguous, as today, but also an *error* case

```rust,ignore
enum AnswerMode {
    OK,
    Ambiguous,
    Error,
}

struct Answer {
    constrained_subst: Canonical<ConstrainedSubst>,
    mode: AnswerMode
}
```

We won't need this error case till later, so let's ignore it for now. (And in a way, we never need it.)

**Deferring coinductive subgoals**

When we encounter a co-inductive subgoal, we check if it is **trivial cycle** or not. A trivial cycle is a case like `C1 :- C1`. We can simply consider such cycles to be true (but note the distinction between a *trivial* cycle and a *self-cycle* -- see the "non-trivial self-cycle" example above).

For non-trivial cycles, we will want to store the cycle to be validated later. To accommodate that, we extend `ExClause` to include a `delayed_subgoals` list as well. We can write this the same way SLG does, so `Goal :- DelayedSubgoals | Subgoals`

In our example, proving `C2 :- C1` would result in adding `C1` to the list of delayed subgoals.

When we reach the end of the list of subgoals, we can create an answer that contains the delayed subgoals. We don't have to add all the goals -- we can check for those that are trivial self-cycles again and remove them (in some cases, something which was not trivial to start may have become trivial through later unifications, see Delayed Trivial Self-Cycle case). Note that we *do* have to add all non-trivial cycles, including non-trivial self-cycles -- see the walkthrough of Non-trivial self-cycle variant 3.

So the answer to `C2` would be

```notrust
substitution: [] // no variables
delayed_subgoals: ["C1"]
region_constraints: []
```

We can denote this as `C2 :- C1 |`, to use SLG notation.

**Incorporating an answer with deferred subgoals.**

When a table gets back an answer that has deferred sub-goals, they get added to the current list of subgoals. 

So e.g. in our case, we had a `ExClause` like:

```notrust
C1 :- | C2, C3
```

and we get the answer `C2 :- C1 |`, so we would convert it to

```notrust
C1 :- | C3, C1
```

i.e., we have added `C1` to the list of goals to prove. When we go to prove `C3`, of course, we will fail -- but it had succeeded, we would go on to prove `C1` but encounter a trivial cycle and hence succeed.

**Extending root answer**

So we failed to prove C1, but we do have a (conditional) answer to C2 -- `C2 :- C1 |`, even though `C2` is unprovable. What happens if `ensure_root_answer` is invoked on `C2`?

What we have here is a *conditional* answer. We know that `C1` must have ultimately resolved itself somehow (although it might not yet be proven). What we can do is create a strand in C2 to evaluate C1 again -- if this strand succeeds, it can actually overwrite the `C2 :- C1 |` answer in place with `C2 :-` (i.e., an unconditional answer). This is just a refinement of what we had. If the strand fails, though, we'll want to remember the error.

I think when we get a new answer, we want it to *overwrite* the old answer in place, rather than create a new answer. This is valid because it's not a new answer, it's just a more refined form of the old answer (although note that it might have different substitutions and other details, see the "delayed trivial cycle" case).

In particular, it could be that the table already has a "complete" set of answers -- i.e., somebody invoked `ensure_answer(N)` and got back `None`. We don't want to be adding new answers which would change the result of that call. It *is* a bit strange that we are changing the result of `ensure_answer(i)` for the current `i`, but then the result is the same answer, just a bit more elaborated.

The idea then would be to create a strand *associated with this answer somehow* (it doesn't, I don't think, live in the normal strand table; we probably have a separate "refinement strand" table). This strand has as its subgoals the delayed subgoals. It pursues them. This either results in an answer (which replaces the existing answer) or an error (in which case the existing answer is marked as *error*). This may require extending strand with an optional answer index that it should overwrite, or perhaps we thread it down as an argument to `pursue_strand` (optional because, in the normal mode, we are just appending a new answer).

(Question: What distinguishes root answer? Nothing -- we could actually do this process for any answer, so long as the delayed subgoals are not to tables actively on the stack. This just happens to be trivially true for root answers. The key part though is that the answer must be registered in the table first before the refinement strand is created, see Delayed Self-Cycle Variant 3.)

This is complex, so let's walk through an example or two.

**The original problem.** When we finish solving `C1`, we leave `C2` with a single answer `C2 :- C1 |`. If someone  invokes `ensure_root_answer(C2, 0)`, we would see the delayed literal and create a refinement strand for the answer: `C2 :- | C1`. We would pursue `C1` and get back the successful answer. So the refinement strand would terminate and we can overwrite with the answer `C2 :- |`.

**Delayed trivial self-cycle.** Similar to above, but the answer is `C2(?A, ?B) :- C1(?B, ?A) |`. In other words, in the canonical answer, we have a (identity) substitution of `[^0, ^1]` and a delayed goal of `C1(^1, ^0)`. The strand we create will find only one answer to `C1`, `C1(22, 22)`, so we wind up with an answer `C2(22, 22)`.

**Handling error answers**

We introduced the idea of an "error answer"...how do we handle that? It's fairly simple. If a strand encounters an error answer, it simply fails. Done. The *outer* search however needs to treat an error answer as basically a no-op -- so e.g. the answer iterator should simply increment the error counter and move to the next answer.

### Walk through: delayed trivial self cycle, variant 2

```notrust
C1(A, B) :- C2(A, B), A = 22.
C2(A, B) :- C1(B, A).
```

* `ensure_root_answer(C1(?A, ?B))` is invoked
    * We start solving `C1(?A, ?B)` with the ex-clause `C1(?A, ?B) :- | C2(?A, ?B), ?A = 22`
        * That starts solving `C2(?A, ?B)`
            * This gets an answer `C2(?A, ?B) :- C1(?B, ?A) |`
            * When answer is incorporated, we get `C1(?A, ?B) :- | C1(?B, ?A), ?A = 22`
        * `C1(?B, ?A)` is a non-trivial cycle, and so we get 
            * `C1(?A, ?B) :- C1(?B, ?A) | ?A = 22`
        * Unification completes, leaving us with
            * `C1(22, ?B) :- C1(?B, 22) |`
        * This is a complete answer
    * ensure root answer attempts to refine this answer, creating a strand for `C1(22, ?B) :- | C1(?B, 22)`
        * This creates a table for `C1(?B, 22)` with ex-clause `C1(?B, 22) :- | C2(?B, 22), ?B = 22`
            * We start solving `C2(?B, 22)`, which has ex-clause `C2(?B, 22) :- C1(22, ?B)`
                * This creates a table for `C1(22, ?B)`, with ex-clause `C1(22, ?B) :- C2(22, ?B), 22 = 22`
                    * This starts solving `C2(22, ?B)`, which is a fresh table with ex-clause `C2(22, ?B) :- C1(?B, 22)`
                        * This is a co-inductive cycle
                        * So our answer is `C2(22, ?B) :- C1(?B, 22) |`
                    * Incorporating this answer yields `C1(22, ?B) :- 22 = 22, C1(?B, 22)`
                    * The unification constraint succeeds, leaving `C1(22, ?B) :- C1(?B, 22)`
                    * Co-inductive cycle detected, so answer is
                        * `C1(22, ?B) :- C1(?B, 22) |`
            * This answer is incorporated into `C2`, yielding the ex-clause
                * `C2(?B, 22) :- C1(?B, 22)`
            * Pursuing that sub-goal gives a co-inductive cycle, so our final answer is
                * `C2(?B, 22) :- C1(?B, 22) |`
        * This answer is incorporated, yielding ex-clause `C1(?B, 22) :- | ?B = 22, C1(?B, 22)`
        * Unification yields `C1(22, 22) :- C1(22, 22)`
        * Trivial self-cycle detected, so final answer is
            * `C1(22, 22)`
    * the answer for `C1(?A, ?B)` is thus updated to `C1(22, 22)`

### Walk through: delayed trivial self cycle, variant 3

```notrust
C1(A, B) :- C1(B, A).
```

This example is interesting because it shows that we have to incorporate non-trivial self cycles into an answer so they can recursively build on one another.

* we get an initial answer of
    * `C1(?A, ?B) :- C1(?B, ?A) |`
* if we attempt to refine this, we will get a strand `C1(?X, ?Y) :- C1(?Y, ?X)`
    * pursuing the first subgoal `C1(?Y, ?X)` leads us to our own table, but at answer 0
        * (the very answer we are refining)
        * the answer is `C1(?Y, ?X) :- C1(?X, ?Y) |`
    * this strand incorporates its own answer, yielding
        * `C1(?X, ?Y) :- C1(?X, ?Y)`
    * next subgoal is a trivial self-cycle, discard, yielding
        * `C1(?X, ?Y) :-`
* result: true



### Walk through: non-trivial self cycle

Let's walk through one more case, the non-trivial self cycle.

```notrust
C1(A) :- C1(B), B = 22, C2(A).
C2(44).
```

What happens here is that we get an initial answer from `C1` that looks like:

```notrust
C1(44) :- C1(22) |
```

Ensure root answer will thus try to refine by trying to solve `C1(22)`. Interestingly, this is going to go to a distinct table, because the canonical form is not the same, but that table will just fail.
