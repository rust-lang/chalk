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

### Generator

A `Generator` represents a Rust generator. There are three major components
to a generator:

* Upvars - similar to closure upvars, they reference values outside of the generator,
  and are stored across al yield points.
* Resume/yield/return types - the types produced/consumed by various generator methods.
  These are not stored in the generator across yield points - they are only
  used when the generator is running.
* Generator witness - see the `Generator Witness` section below.

Of these types, only upvars and resume/yield/return are stored directly in `GeneratorDatum`
(which is acessed via `RustIrDatabase`). The generator witness is implicitly associated with
the generator by virtue of sharing the same `GeneratorId`. It is only used when determining
auto trait impls, where it is considered a 'constituent type'.

### Generator witness types

The `GeneratorWitness` variant represents the generator witness of
the generator with id `GeneratorId`. 

The generator witness contains multiple witness types,
which represent the types that may be part of a generator
state - that is, the types of all variables that may be live across
a `yield` point.

Unlike other types, witnesses include bound, existential
lifetimes, which refer to lifetimes within the suspended stack frame.
You can think of it as a type like `exists<'a> { (T...) }`.

Witnesses have a binder for the erased lifetime(s), which must be
handled specifically in equating and so forth. In many ways,
witnesses are also quite similar to `Function` types, and it is not
out of the question that these two could be unified; however, they
are quite distinct semantically and so that would be an annoying
mismatch in other parts of the system. Witnesses are also similar 
to a `Dyn` type, in that they represent an existential type, but
in contrast to `Dyn`, what we know here is not a *predicate* but
rather some upper bound on the set of types contained within.
