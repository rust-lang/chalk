## The role of the `TypeFamily`

Most everything in the IR is parameterized by the [`TypeFamily`] trait:

[`TypeFamily`]: http://rust-lang.github.io/chalk/chalk_ir/family/trait.TypeFamily.html

```rust,ignore
trait TypeFamily: Copy + Clone + Debug + Eq + Ord { 
    ..
}
```

We'll go over the details later, but for now it suffices to say that
the type family is defined by the embedded and can be used to control
(to a certain extent) the actual representation of types, goals, and
other things in memory. For example, the `TypeFamily` trait could be
used to intern all the types, as rustc does, or it could be used to
`Box` them instead, as the chalk testing harness currently does.

**Question:** What to name the `TypeFamily`? Since instances of it
must be passed around, I've been considering `TypeContext`, as well,
and to call those instances `tcx: TypeContext`, like in the
compiler...?

