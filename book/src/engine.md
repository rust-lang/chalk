# Chalk engine

The `chalk-engine` crate is the core PROLOG-like solver for logical
predicates. Importantly, it is very general and not specific to Rust,
Rust types, or Rust logic.

## Implemented PROLOG concepts

The engine implements the following PROLOG logic concepts. Some of these
have been published on previously, and some are `Chalk`-specific. This isn't
necessarily an exhaustive list:
- Basic logic
- Negation
- Floundering
- Coinductive solving

## Note

Throughout most of this chapter, the specifics in regards to
`Canonicalization` and `UCanonicalization` are avoided. These are important
concepts to understand, but don't particularly help to understand how
`chalk-engine` *works*. In a few places, it may be highlighted if it *is*
important.
