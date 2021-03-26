# Opaque types (impl Trait)

This chapter describes how "opaque types" are modeled in chalk. Opaque types are
the underlying concept used to implement "existential `impl Trait`" in Rust.
They don't have a direct surface syntax, but uses of `impl Trait` in particular
source locations create a hidden opaque type:

```rust,ignore
fn as_u32s<'a, T: Copy + Into<u32>>(
    elements: &'a [T],
) -> impl Iterator<Item = u32> + 'a {
    elements.iter().cloned().map(|e| -> u32 { e.into() })
}

#fn main() {
#    let x: &[u16] = &[1, 2, 3];
#    let y = as_u32s(&x);
#    for e in y {
#        println!("e = {}", e);
#    }
#}
```

Conceptually, the function `as_u32s` is desugared to return a reference to an
*opaque type*, let's call it `AsU32sReturn` (note that this is not valid
Rust syntax):

```rust,ignore
opaque type AsU32sReturn<'a, T>: IntoIterator<Item = u32> + 'a
where
    T: Copy + Into<u32>;

fn as_u32s<'a, T: Copy + Into<u32>>(
    elements: &'a [T],
) -> AsU32sReturn<'a, T> {
    ...
}
```

Opaque types are a kind of type alias. They are called *opaque* because, unlike
an ordinary type alias, most Rust code (e.g., the callers of `as_u32s`) doesn't
know what type `AsU32sReturn` represents. It only knows what traits that type
implements (e.g., `IntoIterator<Item = u32>`). The actual type that is inferred
for `AsU32sReturn` is called the "hidden type".

## Chalk syntax for an opaque type declaration

Although the above is not valid Rust syntax, it is quite close to the
format that chalk unit tests use, which looks something like this:

```rust,ignore
opaque type OpaqueTypeName<P0..Pn>: /* bounds */
where
    /* where clauses */
= /* hidden type */;
```

A chalk opaque type declaration has several parts:

* The **name** `OpaqueTypeName`, which is the name we use to refer to the opaque type
  within the chalk file. In real Rust opaque types are not explicitly declared
  and hence they are identified just by internal ids (i.e., they are anonymous
  in the same way that a closure type is anonymous), so this is just for unit
  testing.
* The **generic parameters** `P0..Pn`. In real Rust, these parameters are inherited
  from the context in which the `impl Trait` appeared. In our example, these
  parameters come from the surrounding function. Note that in real Rust the set
  of generic parameters is a *subset* of those that appear on the surrounding
  function: in particular, lifetime parameters may not appear unless they explicitly
  appear in the opaque type's bounds.
* The **bounds**, which would be `IntoIterator<Item = u32> + 'a` in our example.
  These are traits that the *hidden type* (see below) is supposed to implement.
  They come from the `impl IntoIterator<Item = u32> + 'a` type. Even when the hidden
  type is, well, hidden, we can assume that the bounds hold.
* The **where clauses**, which would be `T: Copy` and `T: Into<u32>` in our
  example. These are conditions that must hold on `V0..Vn` for
  `OpaqueTypeName<V0..Vn>` to be a valid type.
    * Note that this contrasts with bounds: bounds are things that the hidden type must meet
      but which the rest of the code can assume to be true. Where clauses are things
      that the rest of the code must prove to be true in order to use the opaque type.
      In our example, then, a type like `AsU32sReturn<'a, String>` would be invalid
      because `String: Copy` does not hold.

## Representing opaque types in chalk types

We represent opaque types as a kind of **[type alias]**. Like any type alias,
we have to define the conditions in which they can be normalized:

[type alias]: ../types/rust_types/alias.md

## Placeholder rules
