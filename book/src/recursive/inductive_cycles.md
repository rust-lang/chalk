# Inductive cycles

Recursive solving without cycles is easy. Solving with cycles is rather more
complicated. Before we get into the details of the implementation,
let's talk a bit about what behavior we actually *expect* in the face
of possible cycles.

## Inductive cycles

By default, Rust trait solving is **inductive**. What that means is that, roughly
speaking, you have to prove something is true without any cycles (i.e., you
can't say "it's true because it's true"!).

For our purpose, a "cycle" means that, in the course of proving some canonical
goal G, we had to prove that same goal G again.

Consider this Rust program:

```rust
trait A { }
impl<T: A> A for Vec<T> { }
impl A for u32 { }
```

Whether or not we hit a cycle will depend on the goal we are trying
to solve. If for example we are trying to prove `Implemented(Vec<u32>: A)`,
then we don't hit any cycle:

* `Implemented(Vec<u32>: A) :- Implemented(u32: A)` // from the first impl
    * `Implemented(u32: A)` // from the second impl

But what if we are trying to prove `Implemented(?X: A)`? This is a bit
more interesting. Because we don't know what `?X` is, both impls are
actually potentially applicable, so we wind up with two ways to
prove our goal. We will try them out one after the other.

One possible execution might be:

* Prove `Implemented(?X: A)`
    * we find the program clause `forall<T> { Implemented(Vec<T>: A) :- Implemented(T: A) }` from the first impl
        * we create the variable `?Y` to represent `T` and unify `?X = Vec<?Y>`.
        * after unification, we have the subgoal `Implemented(?Y: A)`
            * when we go to recursively prove this impl, however, we find that it is already on the stack
            * this is because the [canonical form] of `Implemented(?X: A)` and `Implemented(?Y: A)` is the same

[canonical form]: ../canonical_queries.md

## What happens if we treat inductive cycles as errors?

So, what do we do when we hit an inductive cycle? Given that we told you that an
inductive proof cannot contain cycles, you might imagine that we can just treat
such a cycle as an error. But this won't give us the correct result.

Consider our previous example. If we just treat that cycle as an error, then we
will conclude that the impl for `Vec<T>` doesn't apply to `?X: A`, and we'll
proceed to try the impl for `u32`. This will let us reason that `?X: A` is
provable if `?X = u32`. This is, in fact, correct: `?X = u32` *is* a possible
answer. The problem is, it's not the only one!

In fact, `Implemented(?X: A)` has an **infinite** number of answers. It is true
for `?X = u32`. It is true for `?X = Vec<u32>`. It is also true for
`Vec<Vec<u32>>` and `Vec<Vec<Vec<u32>>>` and so on.

Given this, the correct result for our query is actually "ambiguous" -- in
particular, there is no unique substitution that we can give that would make the
query provable.

## How we solve cycles: loop and try again

The way we actually handle cycles is by iterating until we reach a fixed point
(or ambiguity). We start out by assuming that all cycles are errors and we try
to find some solution S. If we succeed, then we can do a loop and iterate again
-- this time, for each cycle, we assume the result is S. This may yield some new
solution, S1. The key point here is that we now have **two possible solutions**
to the same goal, S and S1. This implies two possibilities:

* If S == S1, then in fact there is a unique solution, so we can return `Provable(S)`.
* If S != S1, then we know there are two solutions, which means that there is
  not one unique solution, and hence the correct result is **ambiguous**,
  and in fact we can just stop and return right now.

This technique is very similar to the traditional Prolog technique of handling
cycles, which is called **tabling**. The difference between our approach and
tabling is that we are always looking for a unique solution, whereas Prolog
(like the [SLG solver]) tries to enumerate all solutions (i.e., in Prolog,
solving a goal is not a function but an iterator that yields solutions, and
hence it would yield up S first, and then S1, and then any further answers we
might get).

[SLG solver]: ../engine.md

Intuitively, what is happening here is that we're building bigger and bigger
"proof trees" (i.e., trees of impl applications). In the first iteration, where
we assumed that all recursive calls were errors, we would find exactly one
solution, `u32: A` -- this is the root tree. In the next iteration, we can use
this result to build a tree for `Vec<u32>: A` and so forth.

## Inductive cycles with no base case

It is interesting to look at what happens without the base case. Consider this
program:

```rust
trait B { }
impl<T: B> B for Vec<T> { }
```

In this case, there is no base case -- this means that in fact there are no
solutions at all to the query `?X: B`. The reason is that the only type that
could match would be a type of infinite size like `Vec<Vec<Vec<...>>>: B`, where
the chain of `Vec` never terminates.

In our solver, this will work out just fine. We will wind up recursing
and encountering a cycle. This will be treated as an error in the first
iteration -- and then, at the end, we'll still have an error. This means
that we've reached a fixed point, and we can stop.


## Inductive cycles: when do we ever terminate

You might be wondering whether there are any examples of inductive cycles that
actually terminate successfully and without ambiguity. In fact, there are very
few, but you can construct an example like this:

```rust
trait C { }
impl<T: C + D> C for Vec<T> { }
impl C for u32 { }

trait D { }
```

In this case, the only valid result of `Implemented(?X: C)` is `?X = u32`. It can't
be `Vec<u32>` because `Implemented(u32: D)` is not true.

How does this work out with the recursive solver? In the first iteration,
we wind up with `?X = u32`, but we do encounter a cycle:

* proving `Implemented(?X: C)` has two possibilities...
    * `?X = Vec<?Y>` and `Implemented(?Y: C)`, which is a cycle (error, at least in this iteration)
    * `?X = u32`, succeeds

So then we try the next iteration:

* proving `Implemented(?X: C)` has two possibilities...
    * `?X = Vec<?Y>` and `Implemented(?Y: C)`, which is a cycle, so we use our previous result of `?Y = u32`
        * we then have to prove `Implemented(u32: D)`, which fails
    * `?X = u32`, succeeds
