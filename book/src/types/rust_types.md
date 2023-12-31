# Rust types

Rust types are represented by the [`Ty`] and [`TyKind`] types.
You use [`Ty`] to represent "some Rust type". But to actually inspect
what sort of type you have, you invoke the [`kind`] method, which
returns a [`TyKind`]. As described earlier, the actual in-memory
representation of types is controlled by the [`Interner`] trait.

[`Interner`]: https://rust-lang.github.io/chalk/chalk_ir/interner/trait.Interner.html
[`Ty`]: https://rust-lang.github.io/chalk/chalk_ir/struct.Ty.html
[`TyKind`]: https://rust-lang.github.io/chalk/chalk_ir/enum.TyKind.html
[`kind`]: https://rust-lang.github.io/chalk/chalk_ir/struct.Ty.html#method.kind

## The `TyKind` variants and how they map to Rust syntax

This section covers the variants we use to categorize types. We have
endeavored to create a breakdown that simplifies the Rust "surface
syntax" of types to their "essence". In particular, the goal is to
group together types that are largely treated identically by the
system and to separate types when there are important semantic
differences in how they are handled.

| Chalk variant | Example Rust types |
| ------------- | ------------------ |
| `Placeholder` | how we represent `T` when type checking `fn foo<T>() { .. }` |
| `Dyn` | `dyn Trait` |
| `Fn` | `fn(&u8)` |
| `Alias` | `<T as Iterator>::Item`, or the `Foo` in `type Foo = impl Trait` and `type Foo = u32` |
| `BoundVariable` | an uninstantiated generic parameter like the `T` in `struct Foo<T>` |
| `Adt` | `struct Foo<T>` |
| ... | ... |

## Justification for each variant

Each variant of `TyKind` generally wraps a single struct, which
represents a type known to be of that particular variant. This section
goes through the variants in a bit more detail, and in particular
describes why each variant exists.

### Application types

Most of "normal rust types" like `Vec<u32>` or `(f32, Vec<isize>)` are represented with 
`TyKind` variants containing some type-specific info ("type name") and a substitution
that is "applied" to that type. In this case, type names are `Vec` and "tuple of arity 2",
and substitutions are `[u32]` and `[f32, Vec<isize>]`.

They are equal to other types (modulo aliases, see below) iff they
have the same "type name" and the generic arguments are
recursively equal

### Placeholders

The `Placeholder` variant contains a `PlaceholderIndex` type. It
represents a generic type that is being treated abstractly or -- more
generally -- the result of a "type function" that cannot be
evaluated. For example, when typing the body of a generic function
like `fn foo<T: Iterator>`, the type `T` would be represented with a
placeholder. Similarly, in that same function, the associated type
`T::Item` might be represented with a placeholder.

Like application types, placeholder *types* are only known to be
equal.

When proving negative goals, e.g., `not { Implemented(T:
Trait) }`, placeholders are treated quite differently from application
types, since they do not (in fact) represent a known type. When
solving negative goals, placeholders are replaced with inference
variables -- the idea is that this goal is only true if there is *no
type* `T` that implements `Trait`. Therefore, if we can find no
answers for `exists<T> { Implemented(T: Trait) }`, then we know that
the negation is true. (Note that this means that e.g. `forall<X> { X =
i32 }` is false but so is `forall<X> { not { X = i32 } }`.)

### Inference variables

The `InferenceVar` variant wraps an `InferenceVar` type.  This
represents a type whose value is being inferred. The value of an
inference variables may be "known" or "not known", but that state is
stored externally, in the inference context (see the section on
inference below).

When equating, inference variables are treated specially in that they
become bound (or, if they have already been bound, they are replaced
with their value).

Inference variables are also integral to canonicalization and
other types.

### Dyn types

The `Dyn` variant wraps a `DynTy` and represents a `dyn Trait`
type. In chalk, these are represented as an existential type where we
store the predicates that are known to be true. So a type like `dyn
Write` would be represented as, effectively, an `exists<T> { T: Write
}` type.

When equating, two `dyn P` and `dyn Q` types are equal if `P = Q` --
i.e., they have the same bounds. Note that -- for this purpose --
ordering of bounds is significant. That means that if you create a
`dyn Foo + Send` and a `dyn Send + Foo`, chalk would consider them
distinct types. The assumption is that bounds are ordered in some
canonical fashion somewhere else. This may want to change.

There are "automatic" rules for proving that `dyn P: P` and so forth, but
that is outside the scope of the chalk-ir crate.

### Function pointer types

The `Function` variant wraps a `FnPointer` struct and represents a `fn()` type
(in other words, a function pointer). In some ways, fn types are like
application types, but with one crucial difference: they also contain
a `forall` binder that for lifetimes whose value is determined when
the function is called. Consider e.g. a type like `fn(&u32)` or --
more explicitly -- `for<'a> fn(&'a u32)`.

Two `Function` types `A, B` are equal `A = B` if `A <: B` and `B <: A`

Two `Function` types `A, B` are subtypes `A <: B` if

* After instantiating the lifetime parameters on `B` universally...
    * You can instantiate the lifetime parameters on `A` existentially...
        * And then you find that `P_B <: P_A` for every parameter type `P` on `A` and `B` and
          `R_A <: R_B` for the return type `R` of `A` and `B`.

We currently handle type inference with a bit of a hack (same as
rustc); when relating a `Fn` type `F` to an unbounded type
variable `V`, we instantiate `V` with `F`.  But in practice
because of the above subtyping rules there are actually a range of
values that `V` could have and still be equal with `F`. This may
or may not be something to consider revisiting.


### Alias types

The `Alias` variant wraps an `AliasTy` and is used to represent some form of *type
alias*. They are used to represent a number of related Rust concepts, include
actual type aliases, associated types, and opaque types -- you can read about
them in the [aliases chapter](./rust_types/alias.md).

### Bound variables

The `BoundVar` variant represents some variable that is bound in
an outer term. For example, given a term like `forall<X> {
Implemented(X: Trait) }`, the `X` is bound. Bound variables in chalk
(like rustc) use De Bruijn indices (See below).

Bound variables are never directly equated, as any bound variables would have
been instantiated with either inference variables or placeholders.

They do appear in canonical forms and other terms that contain binders.

### Error types

The `Error` variant represents a type that resulted from some
erroneous expression. Error types generally propagate eagerly in an
attempt to suppress nonsense errors that are derived by interactions
with buggy code.

`Error` should be its own variant because most bits of code will want
to handle it somewhat specially -- e.g., maybe it can "unify" with any
other type without any effect, and so forth.

## Mapping to rustc types

The rustc [`TyKind`][Rustc-TyKind] enum is almost equivalent to chalk's. This
section describes how the rustc types can be mapped to chalk
types. The intention is that, at least when transitioning, rustc would
implement the `Interner` trait and would map from the [`TyKind`][Rustc-TyKind]
enum to chalk's [`TyKind`] on the fly, when `data()` is invoked.

[Rustc-TyKind]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_type_ir/ty_kind/enum.TyKind.html

| rustc type | chalk variant (and some notes) |
| ------------- | ------------------ |
| `Bool` | `Scalar` |
| `Char` | `Scalar` |
| `Int` | `Scalar` |
| `Uint` | `Scalar` |
| `Float` | `Scalar` |
| `Adt` | `Adt` |
| `Foreign` | `Foreign` |
| `Str` | `Str` |
| `Array` | `Array` |
| `Slice` | `Slice` |
| `RawPtr` | `Raw` |
| `Ref` | `Ref` |
| `FnDef` | `FnDef` |
| `FnPtr` | `Function` |
| `Dynamic` | `Dyn` |
| `Closure` | `Closure` |
| `Coroutine` | `Coroutine` |
| `CoroutineWitness` | `CoroutineWitness` |
| `Never` | `Never` |
| `Tuple` | `Tuple` |
| `Projection` | `Alias` |
| `UnnormalizedProjection` | (see below) |
| `Opaque` | `Alias` |
| `Param` | XXX Placeholder? |
| `Bound` | `BoundVar` |
| `Placeholder` | `Placeholder` |
| `Infer` | `InferenceVar` |
| `Error` | `Error` |
