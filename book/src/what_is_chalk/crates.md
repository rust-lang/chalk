# Crate breakdown

Chalk is broken up into a number of crates. This chapter explains the
role of each crate. This crate structure helps to serve Chalk's two goals:

* To serve as the trait engine for compilers and tools like rustc and rust-analyzer
* To be usable as a standalone REPL and testing harness

## Crates for embedding chalk into other programs

The following crates are "public facing" crates that you may use when embedding chalk into
other programs:

* The `chalk-solve` crate, which defines the rules that translate Rust IR into logical predicates.
* The `chalk-ir` crate, which defines the IR representing types and logical predicates.
* The `chalk-rust-ir` crate, which defines the IR representing Rust concepts like traits and impls.

The following crate is an implementation detail, used internally by `chalk-solve`:

* The `chalk-engine` crate, which defines the actual engine that solves logical predicate. This 
  engine is quite general and not really specific to Rust.
* The `chalk-derive` crate defines custom derives for the `chalk_ir::fold::Fold` trait and other
  such things.
* The `chalk-macros` crate defines a few miscellaneous utility macros.

## Crates for standalone REPL and testing

The following crates are used to define the REPL and internal testing
harness. These crates build on the crates above. Essentially, they
define a kind of "minimal embedding" of chalk.

* The `chalk-parser` crate can parse Rust syntax to product an AST.
* The `chalk-integration` crate can take that AST and use it to drive
  the `chalk-solve` crate above. The AST is converted into
  `chalk-rust-ir` by a process called "lowering'.
* Finally, the main `chalk` crate, along with the testing crate in the
  `tests` directory, define the actual entry points.
