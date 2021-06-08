## The role of the `Interner`

Most everything in the IR is parameterized by the [`Interner`] trait:

[`Interner`]: https://rust-lang.github.io/chalk/chalk_ir/interner/trait.Interner.html

```rust,ignore
trait Interner: Copy + Clone + Debug + Eq + Ord {
    ..
}
```

We'll go over the details later, but for now it suffices to say that
the interner is defined by the embedder and can be used to control
(to a certain extent) the actual representation of types, goals, and
other things in memory. For example, the `Interner` trait could be
used to intern all the types, as rustc does, or it could be used to
`Box` them instead, as the chalk testing harness currently does.

### Controlling representation with `Interner`

The purpose of the [`Interner`] trait is to give control over how
types and other bits of chalk-ir are represented in memory. This is
done via an "indirection" strategy. We'll explain that strategy here
in terms of [`Ty`] and [`TyKind`], the two types used to represent
Rust types, but the same pattern is repeated for many other things.

[`Interner`]: https://rust-lang.github.io/chalk/chalk_ir/interner/trait.Interner.html
[`Ty`]: https://rust-lang.github.io/chalk/chalk_ir/struct.Ty.html
[`TyKind`]: https://rust-lang.github.io/chalk/chalk_ir/enum.TyKind.html

Types are represented by a [`Ty<I>`] type and the [`TyKind<I>`] enum.
There is no *direct* connection between them. The link is rather made
by the [`Interner`] trait, via the [`InternedTy`] associated type:

[`Ty<I>`]: https://rust-lang.github.io/chalk/chalk_ir/struct.Ty.html
[`TyKind<I>`]: https://rust-lang.github.io/chalk/chalk_ir/enum.TyKind.html
[`InternedTy`]: https://rust-lang.github.io/chalk/chalk_ir/interner/trait.Interner.html#associatedtype.InternedType

```rust,ignore
struct Ty<I: Interner>(I::InternedTy);
enum TyKind<I: Interner> { .. }
```

The way this works is that the [`Interner`] trait has an associated
type [`InternedTy`] and two related methods, [`intern_ty`] and [`ty_data`]:

[`intern_ty`]: https://rust-lang.github.io/chalk/chalk_ir/interner/trait.Interner.html#tymethod.intern_ty
[`ty_data`]: https://rust-lang.github.io/chalk/chalk_ir/interner/trait.Interner.html#tymethod.ty_data

```rust,ignore
trait Interner {
    type InternedTy;

    fn intern_ty(&self, data: &TyKind<Self>) -> Self::InternedTy;
    fn ty_data(data: &Self::InternedTy) -> &TyData<Self>;
}
```

However, as a user you are not meant to use these directly. Rather,
they are encapsulated in methods on the [`Ty`] and [`TyKind`] types:

```rust,ignore
impl<I: Interner> Ty<I> {
  fn data(&self) -> &TyKind<I> {
    I::lookup_ty(self)
  }
}
```

and

```rust,ignore
impl<I: Interner> TyKind<I> {
  fn intern(&self, i: &I) -> Ty<I> {
    Ty(i.intern_ty(self))
  }
}
```

Note that there is an assumption here that [`ty_data`] needs no
context. This effectively constrains the [`InternedTy`] representation
to be a `Box` or `&` type. To be more general, at the cost of some
convenience, we could make that a method as well, so that one would
invoke `ty.data(i)` instead of just `ty.data()`. This would permit us
to use (for example) integers to represent interned types, which might
be nice (e.g., to permit using generational indices).
