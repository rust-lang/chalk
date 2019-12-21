### Controlling representation with `TypeFamily`

The purpose of the [`TypeFamily`] trait is to give control over how
types and other bits of chalk-ir are represented in memory. This is
done via an "indirection" strategy. We'll explain that strategy here
in terms of the [`Ty`] and [`TyData`], the two types used to represent
Rust types, but the same pattern is repeated for many other things.

[`TypeFamily`]: http://rust-lang.github.io/chalk/chalk_ir/family/trait.TypeFamily.html
[`Ty`]: http://rust-lang.github.io/chalk/chalk_ir/struct.Ty.html
[`TyData`]: http://rust-lang.github.io/chalk/chalk_ir/enum.TyData.html

Types are represented by a [`Ty<TF>`] type and the [`TyData<TF>`] enum.
There is no *direct* connection between them. The link is rather made
by the [`TypeFamily`] trait, via the [`InternedTy`] associated type:

[`Ty<TF>`]: http://rust-lang.github.io/chalk/chalk_ir/struct.Ty.html
[`TyData<TF>`]: http://rust-lang.github.io/chalk/chalk_ir/enum.TyData.html
[`InternedTy`]: http://rust-lang.github.io/chalk/chalk_ir/family/trait.TypeFamily.html#associatedtype.InternedType

```rust,ignore
struct Ty<TF: TypeFamily>(TF::InternedTy);
enum TyData<TF: TypeFamily> { .. }
```

The way this works is that the [`TypeFamily`] trait has an associated
type [`InternedTy`] and two related methods, [`intern_ty`] and [`ty_data`]:

[`intern_ty`]: http://rust-lang.github.io/chalk/chalk_ir/family/trait.TypeFamily.html#tymethod.intern_ty
[`ty_data`]: http://rust-lang.github.io/chalk/chalk_ir/family/trait.TypeFamily.html#tymethod.ty_data

```rust,ignore
trait TypeFamily {
    type InternedTy;
    
    fn intern_ty(&self, data: &TyData<Self>) -> Self::InternedTy;
    fn ty_data(data: &Self::InternedTy) -> &TyData<Self>;
}
```

However, as a user you are not meant to use these directly. Rather,
they are encapsulated in methods on the [`Ty`] and [`TyData`] types:

```rust,ignore
impl<TF: TypeFamily> Ty<TF> {
  fn data(&self) -> &TyData<TF> {
    TF::lookup_ty(self)
  }
}
```

and

```rust,ignore
impl<TF: TypeFamily> TyData<TF> {
  fn intern(&self, tf: &TF) -> Ty<TF> {
    Ty(tf.intern_ty(self))
  }
}
```

Note that there is an assumption here that [`ty_data`] needs no
context. This effectively constrains the [`InternedTy`] representation
to be a `Box` or `&` type. To be more general, at the cost of some
convenience, we could make that a method as well, so that one would
invoke `ty.data(tf)` instead of just `ty.data()`. This would permit us
to use (for example) integers to represent interned types, which might
be nice (e.g., to permit using generational indices).

**Question:** Should we make `lookup_ty` a proper method?

