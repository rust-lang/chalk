# Fold and the Folder trait

The [`Fold`] trait permits one to traverse a type or other term in the
chalk-ir and make a copy of it, possibly making small substitutions or
alterations along the way. Folding also allows copying a term from one
interner to another.

[`Fold`]: https://rust-lang.github.io/chalk/chalk_ir/fold/trait.Fold.html

To use the [`Fold`] trait, one invokes the [`Fold::fold_with`] method, supplying some
"folder" as well as the number of "in scope binders" for that term (typically `0`
to start):

```rust,ignore
let output_ty = input_ty.fold_with(&mut folder, 0);
```

[`Fold::fold_with`]: https://rust-lang.github.io/chalk/chalk_ir/fold/trait.Fold.html#tymethod.fold_with

The folder is some instance of the [`Folder`] trait. This trait
defines a few key callbacks that allow you to substitute different
values as the fold proceeds. For example, when a type is folded, the
folder can substitute a new type in its place.

[`Folder`]: https://rust-lang.github.io/chalk/chalk_ir/fold/trait.Folder.html

## Uses for folders

A common use for `Fold` is to permit a substitution -- that is,
replacing generic type parameters with their values.

## From Fold to Folder to SuperFold and back again

The overall flow of folding is like this.

1. [`Fold::fold_with`] is invoked on the outermost term. It recursively
   walks the term.
2. For those sorts of terms (types, lifetimes, goals, program clauses) that have
   callbacks in the [`Folder`] trait, invoking [`Fold::fold_with`] will in turn
   invoke the corresponding method on the [`Folder`] trait, such as `Folder::fold_ty`.
3. The default implementation of `Folder::fold_ty`, in turn, invokes
   `SuperFold::super_fold_with`.  This will recursively fold the
   contents of the type. In some cases, the `super_fold_with`
   implementation invokes more specialized methods on [`Folder`], such
   as [`Folder::fold_free_var_ty`], which makes it easier to write
   folders that just intercept *certain* types.

[`Folder::fold_free_var_ty`]: https://rust-lang.github.io/chalk/chalk_ir/fold/trait.Folder.html#method.fold_free_var_ty

Thus, as a user, you can customize folding by:

* Defining your own `Folder` type
* Implementing the appropriate methods to "intercept" types/lifetimes/etc at the right level of
  detail
* In those methods, if you find a case where you would prefer not to
  substitute a new value, then invoke `SuperFold::super_fold_with` to
  return to the default behavior.

## The `binders` argument

Each callback in the [`Folder`] trait takes a `binders` argument. This indicates
the number of binders that we have traversed during folding, which is relevant for debruijn indices.
So e.g. a bound variable with depth 1, if invoked with a `binders` value of 1, indicates something that was bound to something external to the fold.

XXX explain with examples and in more detail
  
## The `Fold::Result` associated type

The `Fold` trait defines a [`Result`] associated type, indicating the
type that will result from folding.

[`Result`]: https://rust-lang.github.io/chalk/chalk_ir/fold/trait.Fold.html#associatedtype.Result

## When to implement the Fold and SuperFold traits

Any piece of IR that represents a kind of "term" (e.g., a type, part
of a type, or a goal, etc) in the logic should implement `Fold`. We
also implement `Fold` for common collection types like `Vec` as well
as tuples, references, etc.

The `SuperFold` trait should only be implemented for those types that
have a callback defined on the `Folder` trait (e.g., types and
lifetimes).

## Derives

Using the `chalk-derive` crate, you can auto-derive the `Fold` trait.
There isn't presently a derive for `SuperFold` since it is very rare
to require it. The derive for `Fold` is a bit cludgy and requires:

* You must import `Fold` into scope.
* The type you are deriving `Fold` on must have either:
  * A type parameter that has a `Interner` bound, like `I: Interner`
  * A type parameter that has a `HasInterner` bound, like `I: HasInterner`
  * The `has_interner(XXX)` attribute.


