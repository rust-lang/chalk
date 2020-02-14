## The role of the `Interner`

Most everything in the IR is parameterized by the [`Interner`] trait:

[`Interner`]: http://rust-lang.github.io/chalk/chalk_ir/interner/trait.Interner.html

```rust,ignore
trait Interner: Copy + Clone + Debug + Eq + Ord { 
    ..
}
```

We'll go over the details later, but for now it suffices to say that
the interner is defined by the embedded and can be used to control
(to a certain extent) the actual representation of types, goals, and
other things in memory. For example, the `Interner` trait could be
used to intern all the types, as rustc does, or it could be used to
`Box` them instead, as the chalk testing harness currently does.
