# Application types

[`TyKind`] variants that consist of some type-specific info ("type name")
and a substitution are usually referred to as application types. 
These include most of the "normal Rust types", such as `Vec` and `(f32, u32)`.
Such types are only "equal" to themselves (modulo aliases, see below). 
Scalar types (and some others) also fall into this category, despite having no
substitutions: we treat them as having zero-length substitutions.
Note that we group together *both* user-defined structs/enums/unions (like `Vec`)
as well as built-in types like `f32`, which effectively behave the
same.

We used to have application types in chalk as a separate notion in the codebase,
but have since moved away from that; nevertheless, the term is still useful in discussions.

[`TyKind`]: https://rust-lang.github.io/chalk/chalk_ir/enum.TyKind.html

## Notable application types

### Coroutine

A `Coroutine` represents a Rust coroutine. There are three major components
to a coroutine:

* Upvars - similar to closure upvars, they reference values outside of the coroutine,
  and are stored across all yield points.
* Resume/yield/return types - the types produced/consumed by various coroutine methods.
  These are not stored in the coroutine across yield points - they are only
  used when the coroutine is running.
* Coroutine witness - see the `Coroutine Witness` section below.

Of these types, only upvars and resume/yield/return are stored directly in `CoroutineDatum`
(which is accessed via `RustIrDatabase`). The coroutine witness is implicitly associated with
the coroutine by virtue of sharing the same `CoroutineId`. It is only used when determining
auto trait impls, where it is considered a 'constituent type'.

For example:

```rust,ignore
// This is not "real" syntax at the moment.
fn gen() -> Bar {
  let a = yield 0usize;
  use(a)
}

fn use(_: usize) -> Bar {}
```

The type of yield would be `usize`, the resume type would be the type of `a` and the return type
would be `Bar`.

### Coroutine witness types

The `CoroutineWitness` variant represents the coroutine witness of
the coroutine with id `CoroutineId`. 

The coroutine witness contains multiple witness types,
which represent the types that may be part of a coroutine
state - that is, the types of all variables that may be live across
a `yield` point.

Unlike other types, witnesses include bound, existential
lifetimes, which refer to lifetimes within the suspended stack frame.
You can think of it as a type like `exists<'a> { (T...) }`.
As an example, imagine that a type that isn't `Send` lives across a `yield`, then the coroutine
itself can't be `Send`.

Witnesses have a binder for the erased lifetime(s), which must be
handled specifically in equating and so forth. In many ways,
witnesses are also quite similar to `Function` types, and it is not
out of the question that these two could be unified; however, they
are quite distinct semantically and so that would be an annoying
mismatch in other parts of the system. Witnesses are also similar 
to a `Dyn` type, in that they represent an existential type, but
in contrast to `Dyn`, what we know here is not a *predicate* but
rather some upper bound on the set of types contained within.
