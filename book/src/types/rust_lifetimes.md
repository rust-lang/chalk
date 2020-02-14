# Rust lifetimes

Lifetimes are represented by the `Lifetime<I>` and `LifetimeData<I>`
types. As with types, the actual representation of a lifetime is
defined by the associated type `I::InternedLifetime`.

### The `LifetimeData` variants

This section covers the variants we use to categorize lifetimes.

#### Variants and their equivalents in Rust syntax

| Chalk variant | Example Rust types |
| ------------- | ------------------ |
| `BoundVar` | the `'a` in a type like `for<'a> fn(&'a u8)`, before it is instantiated |
| `InferenceVar` | a lifetime whose value is being inferred |
| `Placeholder` | how we represent `'a` when type checking `fn foo<'a>() { .. }` |
| `Static` | the lifetime `'static` |
