# The stack

The first "layer" of the recursive solver is the [`Stack`]. It is really just
what it sounds like: a stack that stores each thing that the recursive solver is
solving. Initially, it contains only one item, the root goal that was given by
the user.

[`Stack`]: https://rust-lang.github.io/chalk/chalk_recursive/fixed_point/stack/struct.Stack.html

Each frame on the stack has an associated [`StackDepth`], which is basically an
index that increases (so 0 is the top of the stack, 1 is the next thing pushed,
etc).

[`StackDepth`]: https://rust-lang.github.io/chalk/chalk_recursive/fixed_point/stack/struct.StackDepth.html

## How the recursive solver works at the highest level

At the highest level, the recursive solver works like so.

* Push the initial goal `G0` onto the stack.
* Find all the program clauses `G1 :- G2...Gn` that could apply to the goal `G0`.
* For each program clause, unify `G1` and `G0`. If that succeeds, then recursively try to prove each goal `Gi` in the list `G2..Gn`:
    * If proving `Gi` yields an error, return an error.
    * If proving `Gi` yields an ambiguity, keep going, but remember that we got an ambiguous result.
    * If proving `Gi` succeeded, apply the resulting answer to our inference variables and keep going.
* At the end, if any result proved ambiguous, return ambiguous, otherwise construct the final answer and return success.

## Example

```rust
trait A { }
trait B { }

impl<T: B> A for Vec<T> { }

impl B for u32 { }
```

Imagine we are trying to prove `Implemented(Vec<?X>: A)`. There is one unbound
inference variable here, `?X`. We will ultimately get the result `Provable(?X =
u32)`. But how do we find it?

* Initially we are solving `Implemented(Vec<?X>: A)`
    * we find one applicable program clause, `forall<T> { Implemented(Vec<T>: A) :- Implemented(T: B) }`.
    * after unification, the list of subgoals is `[Implemented(?X: B)]`.
    * we recursively try to solve `Implemented(?X: B)`
        * we find one applicable program clause, `Implemented(u32: B)`.
        * after unification, `?X = u32`, but there are no more subgoals.
        * we return the answer `Provable(?X = u32)`.
    * we apply the substitution `?X = u32`, and find there are no more subgoals.
    * we return the answer `Provable(?X = u32)`.

## Why do we need the stack?

You may have noticed that the description above never seemed to use the [`Stack`],
it only relied on the program stack. That's because I left out any discussion
of cycles. In fact, the [`Stack`] data structure does mirror the program stack,
it just adds some extra information we use in resolving cycles. We'll discuss
cycles in the next chapter, when we discuss the [search graph].

## Figuring out if something is on the stack

The stack itself never stores the goal associated with a particular entry. That
information is found in the [search graph], which will be covered in detail in
the next section. For now it suffices to say that the search graph maps from
"some goal that we are currently solving" to "information about that goal", and
one of the bits of information is the [`StackDepth`] of its entry on the stack
(if any).

Therefore, when we are about to start solving some (canonical) goal G, we can
detect a cycle by checking in the [search graph] to see whether G has an associated
[`StackDepth`]. If so, it must be on the stack already (and we can set the
[`cycle`] field to true...but I get ahead of myself, read the next chapters
to learn more about that).

[search graph]: ./search_graph.md
[`cycle`]: https://rust-lang.github.io/chalk/chalk_recursive/fixed_point/stack/struct.StackEntry.html#structfield.cycle