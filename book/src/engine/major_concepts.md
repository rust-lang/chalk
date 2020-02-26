# Major concepts

This section goes over a few different concepts that are crucial to
understanding how `chalk-engine` works, without going over the exact solving
logic.

## `Context`, `ContextOps`, and `InferenceTable`

### `Context`

The [`Context`] trait is the primary bridge between Chalk internal logic and
external types. In addition actually *defining* the types (via associated
types), it also contains associated functions to convert or extract
information from those types. Overall, this allows the types to be basically
opaque to the engine internals. Functions in the trait are agnostic to specific
program or environment details, since they lack a `&self` argument.

To give an example, there is an associated [`Goal`] type. However, Chalk doesn't
know how to solve this. Instead, it has to be converted an [`HhGoal`] via the
`Context::into_hh_goal` function. This will be coverted more in the `Goals`
section.

### `ContextOps`

The [`ContextOps`] trait contains functions that may specifically require
information a specific program or environment. For example, the
`program_clauses` function gives potential ways to prove a `Goal`, but obviously
 it requires knowing the program (for example, what types, traits, and impls
 there are). Functions in this trait all take a `&self` argument.

### `InferenceTable`

The [`InferenceTable`] is a super trait to the [`UnificationOps`], [`TruncateOps`],
and [`ResolventOps`]. Each of these contains functions that track the state of
specific parts of the program. Importantly, these operations can dynamically
change the state of the logic itself.

## Goals

A "goal" in Chalk can be thought of as "something we want to prove". The engine
itself understands [`HhGoal`]s. `HHGoal`s consist of the most basic logic,
such as introducing Binders (`Forall` or `Exists`) or combining goals (`All`).
On the other hand, `Context::Goal` represents an opaque goal generated
externally. As such, it may contain any extra information or may be interned.
When solving a logic predicate, Chalk will lazily convert `Context::Goal`s
into `HHGoal`s.

There are three types of completely opaque `HhGoal`s that Chalk can solve:
`Unify`, `DomainGoal`, and `CannotProve`. Unlike the other types of goals,
these three cannot be simiplified any further. `Unify` is the goal of unifying
any two types. `DomainGoal` is any goal that can solve by applying a
[`ProgramClause`]. To solve this, more `Goal`s may generated. Finally,
`CannotProve` is a special goal that *cannot* be proven true or false.

## Answers and Solutions

For every `Goal`, there are zero or more `Answer`s. Each [`Answer`] contains
values for the inference variables in the goal.

For example, given the following program:
```notrust
trait Clone {}
struct A {}
struct B {}
impl Clone for A {}
impl Clone for B {}
```
With the following goal: `exists<T> { T: Clone }`
The following solutions would be given:
```notrust
T = A
T = B
```
In other words, either `A` or `B` can substituted for `T` and the goal will
hold true. Moreover, either answer could be used when further solving other
goals that depend on this goal.

However, oftentimes, this is not what external crates want when solving for a
goal. Instead, the may want a *unique* solution to this goal. Indeed, when we
solve for a given root [`Goal`], we return a since [`Solution`]. It is up to the
implementation of [`Context`] to decide how a `Solution` is made, given a possibly
infinite set of answers. One of example of this is the
[`AntiUnifier`](https://rust-lang.github.io/chalk/chalk_solve/solve/slg/aggregate/struct.AntiUnifier.html)
from `chalk-solve`, which finds a minimal generalization of answers which don't
unify. (For the example above, it would return only `Ambiguous`, since `A` and
`B` can't unify.)

## ExClauses and Strands

An [`ExClause`] is described in literature as `A :- D | G` or
`A holds given that G holds with D delayed goals`. In `chalk-engine`, an
`ExClause` stores the current state of proving a goal, including existing
substitutions already found, subgoals yet to be proven, or delayed subgoals. A
[`Strand`] wraps both an [`ExClause`] and an [`InferenceTable`] together. 

## Tables and Forests

A [`Strand`] represents a *single* direction to find an [`Answer`] - for example, an
implementation of a trait with a set of where clauses. However, in a program,
there may be *multiple* possible implementations that match a goal - e.g.
multiple impls with different where clauses. Every [`Table`] has a goal, and
stores existing `Answers`, as well as all `Strand`s that may result in more
answers.

A [`Forest`] holds all the `Table`s that program generates, and is what most of
the logic is implemented on. It also stores the current state of solving (the
stack).



[`Context`]: https://rust-lang.github.io/chalk/chalk_engine/context/trait.Context.html
[`ContextOps`]: https://rust-lang.github.io/chalk/chalk_engine/context/trait.ContextOps.html
[`InferenceTable`]: https://rust-lang.github.io/chalk/chalk_engine/context/trait.InferenceTable.html
[`HhGoal`]: https://rust-lang.github.io/chalk/chalk_engine/hh/enum.HhGoal.html
[`Solution`]: https://rust-lang.github.io/chalk/chalk_engine/context/trait.Context.html#associatedtype.Solution
[`ExClause`]: https://rust-lang.github.io/chalk/chalk_engine/struct.ExClause.html
[`Strand`]: https://rust-lang.github.io/chalk/chalk_engine/strand/struct.Strand.html
[`Table`]: https://rust-lang.github.io/chalk/chalk_engine/table/struct.Table.html
[`Forest`]: https://rust-lang.github.io/chalk/chalk_engine/forest/struct.Forest.html
[`Goal`]: https://rust-lang.github.io/chalk/chalk_engine/context/trait.Context.html#associatedtype.Goal
[`UnificationOps`]: https://rust-lang.github.io/chalk/chalk_engine/context/trait.UnificationOps.html
[`TruncateOps`]: https://rust-lang.github.io/chalk/chalk_engine/context/trait.TruncateOps.html
[`ResolventOps`]: https://rust-lang.github.io/chalk/chalk_engine/context/trait.ResolventOps.html
[`ProgramClause`]: https://rust-lang.github.io/chalk/chalk_engine/context/trait.Context.html#associatedtype.ProgramClause
[`Answer`]: https://rust-lang.github.io/chalk/chalk_engine/struct.Answer.html