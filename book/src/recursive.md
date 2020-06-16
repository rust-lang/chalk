# Chalk recursive solver

The recursive solver, as its name suggests, is a logic solver that works
"recursively". In particular, its basic structure is a function like:

```rust,ignore
fn(Goal) -> Solution
```

where the Goal is some [canonical goal](./canonical_queries.md) and
the Solution is a result like:

* Provable(S): meaning the goal is provable and it is provably exactly (and
  only) for the substitution S. S is a set of values for the inference variables
  that appear in the goal. So if we had a goal like `Vec<?X>: Foo`, and we
  returned `Provable(?X = u32)`, it would mean that only `Vec<u32>: Foo` and not
  any other sort of vector (e.g., `Vec<u64>: Foo` does not hold).
* Ambiguous(S): meaning that we can't prove whether or not the goal is true.
  This can sometimes come with a substitution S, which offers suggested values
  for the inference variables that might make it provable.
* Error: the goal cannot be proven.

## Recursion: pros and cons

The recursive solver is so-called because, in the process of solving one goal,
it will "recurse" to solve another. Consider an example like this:

```rust,ignore
trait A { }
impl<T: A> A for Vec<T> { }
impl A for u32 { }
impl A for i32 { }
```

which results in program clauses like:

```notrust
forall<T> { Implemented(Vec<T>: A) :- Implemented(T: A) }
Implemented(u32: A)
Implemented(i32: A)
```

First, suppose that we have a goal like `Implemented(Vec<u64>: A)`. This would
proceed like so:

* `Solve(Implemented(Vec<u64>: A))`
    * `Solve(Implemented(u64: A))`
        * returns `Error`
    * returns `Error`

In other words, the recursive solver would start by applying the first rule,
which would cause us recursively try to solve `Implemented(u64: A)`. This would
yield an Error result, because there are no applicable rules, and that error
would propagate back up, causing the entire attempt at proving things to fail.

Next, consider `Implemented(Vec<u32>: A)`. This would proceed like so:

* `Solve(Implemented(Vec<u32>: A))`
    * `Solve(Implemented(u32: A))`
        * returns `Provable` with no substitution (no variables)
    * returns `Provable`

Finally, consider `Implemented(Vec<?X>: A)`. This is more interesting because it
has a variable:

* `Solve(Implemented(Vec<?X>: A))`
    * `Solve(Implemented(?X: A))`
        * finds two viable solutions, returns `Ambiguous`
    * returns `Ambiguous`

## Recursion and completeness

One side-effect of the recursive solver's structure is that it
cannot solve find solutions in some cases where a traditional
Prolog solver would be successful. Consider this example:

```rust
trait A { }
trait B { }

impl<T: A + B> A for Vec<T> { }

impl A for u32 { }
impl B for u32 { }

impl A for i32 { }
impl B for i8 { }
```

In the recursive solver, with a goal of `Implemented(Vec<?X>: A)`, we
recursively try to prove `Implemented(?X: A)` and `Implemented(?X: B)`, which
are both ambiguous, and we get stuck there.

The [SLG solver] in contrast starts by exploring `?X = u32` and finds
that it works, and then later tries to explore `?X = i32` and finds that it
fails (because `i32: B` is not true).

[SLG solver]: ./engine.md
