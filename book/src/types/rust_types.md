# Rust types

Rust types are represented by the [`Ty`] and [`TyData`] types.
You use [`Ty`] to represent "some Rust type". But to actually inspect
what sort of type you have, you invoke the [`data`] method, which
returns a [`TyData`]. As described earlier, the actual in-memory
representation of types is controlled by the [`Interner`] trait.

[`Interner`]: http://rust-lang.github.io/chalk/chalk_ir/interner/trait.Interner.html
[`Ty`]: http://rust-lang.github.io/chalk/chalk_ir/struct.Ty.html
[`TyData`]: http://rust-lang.github.io/chalk/chalk_ir/enum.TyData.html
[`data`]: http://rust-lang.github.io/chalk/chalk_ir/struct.Ty.html#method.data

## The `TyData` variants and how they map to Rust syntax

This section covers the variants we use to categorize types. We have
endeavored to create a breakdown that simplifies the Rust "surface
syntax" of types to their "essence". In particular, the goal is to
group together types that are largely treated identically by the
system and to separate types when there are important semantic
differences in how they are handled.

| Chalk variant | Example Rust types |
| ------------- | ------------------ |
| `Apply` | `Vec<u32>`, `f32` |
| `Placeholder` | how we represent `T` when type checking `fn foo<T>() { .. }` |
| `Dyn` | `dyn Trait` |
| `Fn` | `fn(&u8)` |
| `Alias` | `<T as Iterator>::Item`, or the `Foo` in `type Foo = impl Trait` and `type Foo = u32` |
| `BoundVariable` | an uninstantiated generic parameter like the `T` in `struct Foo<T>` |

## Justification for each variant

Each variant of `TyData` generally wraps a single struct, which
represents a type known to be of that particular variant. This section
goes through the variants in a bit more detail, and in particular
describes why each variant exists.

### Application types

The `Apply` variant contains an `ApplicationTy`. These are kind of the
"normal Rust types", like `Vec<u32>` or `f32`. They consist of a "type
name" (in our examples, `Vec` and `f32` respecively) and zero or more
generic arguments (respectively, `[u32]` and `[]`).

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

However, we choose not to represent placeholder types as type names
because they need to be created during type unification and other
operations, and hence that would require treating `TypeName` less opaquely.

Moreover, when proving negative goals, e.g., `not { Implemented(T:
Trait) }`, placeholders are treated quite differently from application
types, since they do not (in fact) represent a known type. When
solving negative goals, placeholderes are replaced with inference
variables -- the idea is that this goal is only true if there is *no
type* `T` that implements `Trait`. Therefore, if we can find no
answeres for `exists<T> { Implemented(T: Trait) }`, then we know that
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

The `Fn` variant wraps a `FnTy` struct and represents a `fn()` type
(in other words, a function pointer). In some ways, fn types are like
application types, but with one crucial difference: they also contain
a `forall` binder that for lifetimes whose value is determined when
the function is called. Consider e.g. a type like `fn(&u32)` or --
more explicitly -- `for<'a> fn(&'a u32)`.
  
Two `Fn` types `A, B` are equal `A = B` if `A <: B` and `B <: A`

Two `Fn` types `A, B` are subtypes `A <: B` if

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

### Generator witness types

The `GeneratorWitness` variant wraps a `GeneratorWitness` type.  These
witnesses represent the types that may be part of a generator
state. Unlike other types, witnesses include bound, existential
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

### Alias types

The `Alias` variant wraps an `AliasTy` and is used to represent some form of *type
alias*. These correspond to associated type projections like `<T as Iterator>::Item`
but also `impl Trait` types and named type aliases like `type Foo<X> = Vec<X>`. 

Each alias has an alias id as well as parameters. Aliases effectively
represent a *type function*.

Aliases are quite special when equating types. In general, an alias
type `A` can also be equal to *any other type* `T` if evaluating the
alias `A` yields `T` (this is currently handled in Chalk via a
`ProjectionEq` goal, but it would be renamed to `AliasEq` under this
proposal).

However, some alias types can also be instantiated as "alias
placeholders". This occurs when the precise type of the alias is not
known, but we know that there is *some type* that it evaluates to (for
example, `<T as Iterator>::Item` might be treated opaquely as
`T::Item`; similarly `impl Trait` types are treated opaquely until the
latter phases of the compiler). Alias placeholders are not represented
with the `Alias` variant, but rather with the placeholder variant
described previously.

### Bound variables

The `BoundVariable` variant represents some variable that is bound in
an outer term. For example, given a term like `forall<X> {
Implemented(X: Trait) }`, the `X` is bound. Bound variables in chalk
(like rustc) use de bruijin indices (See below).

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

The rustc [`TyKind`] enum has a lot more variants than chalk. This
section describes how the rustc types can be mapped to chalk
types. The intention is that, at least when transitioning, rustc would
implement the `Interner` trait and would map from the [`TyKind`]
enum to chalk's `TyData` on the fly, when `data()` is invoked.

[`TyKind`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc/ty/enum.TyKind.html

This section describes how each of rustc's variants can be mapped to
Chalk variants.

| rustc type | chalk variant (and some notes) |
| ------------- | ------------------ |
| `Bool` | `Apply` |
| `Char` | `Apply` |
| `Int(_)` | `Apply` |
| `Uint(_)` | `Apply` |
| `Float(_)` | `Apply` |
| `Adt(_, _)` | `Apply` |
| `Foreign(_)` | `Apply` |
| `Str` | `Apply` |
| `Array(_, _)` | `Apply` |
| `Slice(_)` | `Apply` |
| `RawPtr(_)` | `Apply` |
| `Ref(_, _, _)` | `Apply` |
| `FnDef(_, _)` | `Apply` |
| `FnPtr(_, _)` | `Fn` |
| `Dynamic(_, _)` | `Dyn` |
| `Closure(_, _)` | `Apply` |
| `Generator(_, _)` | `Apply` |
| `GeneratorWitness(_)` | `GeneratorWitness` |
| `Never` | `Apply` |
| `Tuple(_)` | `Apply` |
| `Projection(_)` | `Alias` |
| `UnnormalizedProjection(_)` | (see below) |
| `Opaque(_, _)` | `Alias` |
| `Param(_)` | XXX Placeholder? |
| `Bound(_, _)` | `BoundVariable` |
| `Placeholder(_)` | `Placeholder` |
| `Infer(_)` | `InferenceVar` |
| `Error` | `Error` |
