# Crate breakdown

Chalk is broken up into a number of crates. This chapter explains the
role of each crate. This crate structure helps to serve Chalk's two goals:

* To serve as the trait engine for compilers and tools like rustc and rust-analyzer
* To be usable as a standalone REPL and testing harness

## Crates for embedding chalk into other programs

The following crates are "public facing" crates that you may use when embedding chalk into
other programs:

* The `chalk-solve` crate, which defines the IR representing Rust concepts like
  traits and impls and the rules that translate Rust IR into logical predicates.
* The `chalk-ir` crate, which defines the IR representing types and logical predicates.

The following crate is an implementation detail, used internally by `chalk-solve`:

* The `chalk-engine` crate, which defines the actual engine that solves logical predicate. This
  engine is quite general and not really specific to Rust.
* The `chalk-derive` crate defines custom derives for the `chalk_ir::fold::TypeFoldable` trait and other
  such things.

## Crates for standalone REPL and testing

The following crates are used to define the REPL and internal testing
harness. These crates build on the crates above. Essentially, they
define a kind of "minimal embedding" of chalk.

* The `chalk-parser` crate can parse Rust syntax to produce an AST.
* The `chalk-integration` crate can take that AST and use it to drive the
  `chalk-solve` crate above. The AST is converted into Rust IR by a process
  called "lowering".
* Finally, the main `chalk` crate, along with the testing crate in the
  `tests` directory, define the actual entry points.

## The chalk-solve crate

| The `chalk-solve` crate |                       |
| ----------------------- | --------------------- |
| Purpose:                | to solve a given goal |
| Depends on IR:          | chalk-ir and rust-ir  |
| Context required:       | `RustIrDatabase`      |

The `chalk-solve` crate exposes a key type called `Solver`.  This is a
solver that, given a goal (expressed in chalk-ir) will solve the goal
and yield up a `Solution`. The solver caches intermediate data between
invocations, so solving the same goal twice in a row (or solving goals
with common subgoals) is faster.

The solver is configured by a type that implements the
`RustIrDatabase` trait. This trait contains some callbacks that
provide needed context for the solver -- notably, the solver can ask:

- **What are the program clauses that might solve given rule?** This
  is answered by the code in the chalk-solve crate.
- **Is this trait coinductive?** This is answered by the chalk-ir.


## The chalk-engine crate

| The `chalk-engine` crate |                                  |
| ------------------------ | -------------------------------- |
| Purpose:                 | define the base solving strategy |
| IR:                      | none                             |
| Context required:        | `Context` trait                  |

For the purposes of chalk, the `chalk-engine` crate is effectively
encapsulated by `chalk-solve`.  It defines the base SLG engine. It is
written in a very generic style that knows next to nothing about Rust
itself. The engine can be configured via the traits defined in
`chalk_engine::context::Context`, which contain (for example)
associated types that define what a goal or clause is, as well as
functions that operate on those things.
