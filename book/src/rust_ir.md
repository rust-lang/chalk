# Intermediate representations

Intermediate representations (IR) are used to represent parts of Rust programs such as traits and impls.

Chalk contains three levels of IR:

- The **AST**. This is used purely for writing test cases
  with a Rust-like syntax. This is consumed by **lowering** code, which
  takes AST and products **Rust IR** (the next bullet point).
- The **Rust IR**. This is a "HIR-like" notation that defines the
  interesting properties of things like traits, impls, and structs.
  It is an input to the **rules** code, which produces
- The **Chalk IR**. This is most "Prolog-like" of the various IRs. It
  contains the definition of **types** as well as prolog-like concepts
  such as goals (things that must be proven true) and clauses (things
  that are assumed to be true).
