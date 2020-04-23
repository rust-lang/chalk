# Application types

An [`ApplicationTy`] is kind of a "normal Rust type", like
`Vec<u32>` or `f32`. Such types are only "equal" to themselves (modulo
aliases, see below), and they may take type arguments.  Note that we
group together *both* user-defined structs/enums/unions (like `Vec`)
as well as built-in types like `f32`, which effectively behave the
same.

[`ApplicationTy`]: http://rust-lang.github.io/chalk/chalk_ir/struct.ApplicationTy.html

An [`ApplicationTy`] contains two fields:

* a "type name" (of type [`TypeName`]); and,
* a list of generic arguments (of type [`Substitution`]).

The [`TypeName`] itself is largely opaque to chalk. We discuss it in
more detail elsewhere. The point is that it represents, semantically,
either the name of some user-defined type (like `Vec`) or builtin-types
like `i32`. It may also represent types like "tuple of arity 2" (`(_,
_)`) or "fixed-length array" `[_; _]`. Note that the precise set of
these built-in types is defined by the `Interner` and is unknown to
chalk-ir.

[`TypeName`]: http://rust-lang.github.io/chalk/chalk_ir/enum.TypeName.html
[`Substitution`]: http://rust-lang.github.io/chalk/chalk_ir/struct.Substitution.html
