# Application types

An [`ApplicationTy`] is kind of a "normal Rust type", like
`Vec<u32>` or `f32`. Such types are only "equal" to themselves (modulo
aliases, see below), and they may take type arguments.  Note that we
group together *both* user-defined structs/enums/unions (like `Vec`)
as well as built-in types like `f32`, which effectively behave the
same.

[`ApplicationTy`]: http://rust-lang.github.io/chalk/chalk_ir/struct.ApplicationTy.html

An [`ApplicationTy`] contains two fields:

* a "type name" (of type [`TypeName`]); and,
* a list of generic arguments (of type [`Substitution`]).

The [`TypeName`] itself is largely opaque to chalk. We discuss it in
more detail elsewhere. The point is that it represents, semantically,
either the name of some user-defined type (like `Vec`) or builtin-types
like `i32`. It may also represent types like "tuple of arity 2" (`(_,
_)`) or "fixed-length array" `[_; _]`. Note that the precise set of
these built-in types is defined by the `Interner` and is unknown to
chalk-ir.

## [`TypeName`] variants

### Generator

A `Generator` represents a Rust generator. There are three major components
to a generator:

* Upvars - similar to closure upvars, they reference values outside of the generator,
  and are stored across al yield points.
* Resume/yield/return types - the types produced/consumed by various generator methods.
  These are not stored in the generator across yield points - they are only
  used when the generator is running.
* Generator witness - see the `Generator Witness` section below.

Of these types, only upvars and resume/yield/return are stored directly in
`TypeName::Generator`. The generator witness is implicitly associated with the generator
by virtue of sharing the same `GeneratorId`. It is only used when determining auto trait
impls, where it is considered a 'constituent type'.

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

Witnesses are very similar to an `Apply` type, but it has a binder for
the erased lifetime(s), which must be handled specifically in equating
and so forth. In many ways, witnesses are also quite similar to `Fn`
types, and it is not out of the question that these two could be
unified; however, they are quite distinct semantically and so that
would be an annoying mismatch in other parts of the system.
Witnesses are also similar to a `Dyn` type, in that they represent an
existential type, but in contrast to `Dyn`, what we know here is
not a *predicate* but rather some upper bound on the set of types
contained within.


[`TypeName`]: http://rust-lang.github.io/chalk/chalk_ir/enum.TypeName.html
[`Substitution`]: http://rust-lang.github.io/chalk/chalk_ir/struct.Substitution.html
