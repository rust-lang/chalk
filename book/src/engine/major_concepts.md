# Major concepts

This section goes over a few different concepts that are crucial to
understanding how `chalk-engine` works, without going over the exact solving
logic.

## Goals

A "goal" in Chalk can be thought of as "something we want to prove". The engine
itself understands `GoalData`s. `GoalData`s consist of the most basic logic,
such as introducing Binders (`Forall` or `Exists`) or combining goals (`All`).
On the other hand, `DomainGoal` represents an opaque goal generated
externally. As such, it may contain any extra information or may be interned.
When solving a logic predicate, Chalk will lazily convert `DomainGoal`s
into `GoalData`s.

There are three types of completely opaque `GoalData`s that Chalk can solve:
`Unify`, `DomainGoal`, and `CannotProve`. Unlike the other types of goals,
these three cannot be simplified any further. `Unify` is the goal of unifying
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
solve for a given root [`Goal`], we return a single [`Solution`]. The
[`AntiUnifier`](https://rust-lang.github.io/chalk/chalk_engine/slg/aggregate/struct.AntiUnifier.html)
struct from `chalk-solve` then finds that solution, by finding a minimal
generalization of answers which don't
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
[`InferenceTable`]: https://rust-lang.github.io/chalk/chalk_solve/infer/struct.InferenceTable.html
[`Solution`]: https://rust-lang.github.io/chalk/chalk_solve/solve/enum.Solution.html
[`ExClause`]: https://rust-lang.github.io/chalk/chalk_engine/struct.ExClause.html
[`Strand`]: https://rust-lang.github.io/chalk/chalk_engine/strand/struct.Strand.html
[`Table`]: https://rust-lang.github.io/chalk/chalk_engine/table/struct.Table.html
[`Forest`]: https://rust-lang.github.io/chalk/chalk_engine/forest/struct.Forest.html
[`Goal`]: https://rust-lang.github.io/chalk/chalk_ir/struct.Goal.html
[`UnificationOps`]: https://rust-lang.github.io/chalk/chalk_engine/context/trait.UnificationOps.html
[`TruncateOps`]: https://rust-lang.github.io/chalk/chalk_engine/context/trait.TruncateOps.html
[`ResolventOps`]: https://rust-lang.github.io/chalk/chalk_engine/context/trait.ResolventOps.html
[`ProgramClause`]: https://rust-lang.github.io/chalk/chalk_ir/struct.ProgramClause.html
[`Answer`]: https://rust-lang.github.io/chalk/chalk_engine/struct.Answer.html
