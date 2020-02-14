# Representing and manipulating Rust types

## Goal of the chalk-ir crate

To have an ergonomic, flexible library that can abstractly represent
Rust types and logical predicates. The library should be expose a
"minimal" set of types that is nonetheless able to capture the full
range of Rust types. "Minimal" here means that some of the surface
differences in Rust types -- e.g., the distinction between built-in
types like `u32` and user-defined types like a struct -- ought to be
minimized, so that code that works with these types (e.g., trait
solving) can focus on the most important differences.

## Goal: support embedding and a variety of contexts

One of our goals is to create a type representation that can be
readily embedded into a variety of contexts. Most specifically, we
would like to be able to embed into rustc and rust-analyzer, and
permit those two projects to use distinct memory management
strategies. This is primarily achieved via the `Interner` trait.

Initially, at least in rustc, the goal is to be able to easily and
"reasonably efficiently" convert back and forth between rustc's native
type representation and chalk's representation. Once chalk's design
has stabilized, however, the goal would be for rustc to adopt this
format as its "native" representation.

Note that even if the chalk type library were used everywhere,
however, it would still be useful for rustc to be able to control the
memory management strategy. (In other words, different consumers might
wish to use it in different ways.)

## Note on status

At the moment, this documentation is a "proposal". That means that it
diverges in some places from what is actually implemented. It has also
not been thoroughly discussed by the Rust compiler team as a whole.

Here is a (partial) list of some things that have to be adapted in
Chalk as of today to match this document:

* `Parameter<I>` needs to be renamed to `GenericArgument`
* `Vec<Parameter<I>>` needs to be replaced with `GenericArguments`
* Extract `TypeName` into something opaque to chalk-ir.
* Dyn type equality should probably be driven by entailment.
* Projections need to be renamed to aliases.
* The variant we use for impl traits should be removed and folded into type aliases. 
* Remove placeholders and projection placeholders from apply and create placeholder types.
* Move `Error` from a `TypeName` to its own variant.
* Introduce `GeneratorWitness` into chalk
* Complete transition from `ForAll` to `Fn` in chalk 
