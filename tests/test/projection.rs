//! Tests related to projection of associated types and normalization.

use super::*;

#[test]
fn normalize_basic() {
    test! {
        program {
            trait Iterator { type Item; }
            struct Vec<T> { }
            struct Foo { }
            impl<T> Iterator for Vec<T> {
                type Item = T;
            }
        }

        goal {
            forall<T> {
                exists<U> {
                    Normalize(<Vec<T> as Iterator>::Item -> U)
                }
            }
        } yields {
            expect![["Unique; substitution [?0 := !1_0]"]]
        }

        goal {
            forall<T> {
                Vec<T>: Iterator<Item = T>
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<T> {
                if (T: Iterator<Item = Foo>) {
                    <T as Iterator>::Item = Foo
                }
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<T> {
                if (T: Iterator) {
                    exists<U> {
                        T: Iterator<Item = U>
                    }
                }
            }
        } yields {
            expect![["Unique; substitution [?0 := (Iterator::Item)<!1_0>]"]]
        }

        goal {
            forall<T> {
                if (T: Iterator) {
                    exists<U> {
                        T: Iterator<Item = U>
                    }
                }
            }
        } yields {
            expect![["Unique; substitution [?0 := (Iterator::Item)<!1_0>]"]]
        }

        goal {
            forall<T> {
                if (T: Iterator) {
                    <T as Iterator>::Item = <T as Iterator>::Item
                }
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<T> {
                if (T: Iterator) {
                    exists<U> {
                        <T as Iterator>::Item = <U as Iterator>::Item
                    }
                }
            }
        } yields {
            // True for `U = T`, of course, but also true for `U = Vec<<T as Iterator>::Item>`.
            expect![["Ambiguous; no inference guidance"]]
        }
    }
}

#[test]
fn normalize_into_iterator() {
    test! {
        program {
            trait IntoIterator { type Item; }
            trait Iterator { type Item; }
            struct Vec<T> { }
            struct Foo { }
            impl<T> IntoIterator for Vec<T> {
                type Item = T;
            }
            impl<T> IntoIterator for T where T: Iterator {
                type Item = <T as Iterator>::Item;
            }
        }

        goal {
            forall<T> {
                exists<U> {
                    Normalize(<Vec<T> as IntoIterator>::Item -> U)
                }
            }
        } yields {
            expect![["Unique; substitution [?0 := !1_0]"]]
        }
    }
}

#[test]
fn projection_equality() {
    test! {
        program {
            trait Trait1 {
                type Type;
            }
            trait Trait2<T> { }
            impl<T, U> Trait2<T> for U where U: Trait1<Type = T> {}

            struct S {}
            impl Trait1 for S {
                type Type = u32;
            }
        }

        goal {
            exists<U> {
                S: Trait1<Type = U>
            }
        } yields[SolverChoice::slg_default()] {
            // this is wrong, chalk#234
            expect![["Ambiguous; no inference guidance"]]
        } yields[SolverChoice::recursive_default()] {
            expect![["Unique; substitution [?0 := Uint(U32)]"]]
        }

        goal {
            exists<U> {
                S: Trait2<U>
            }
        } yields[SolverChoice::slg_default()] {
            // this is wrong, chalk#234
            expect![["Ambiguous; no inference guidance"]]
        } yields[SolverChoice::recursive_default()] {
            expect![["Unique; substitution [?0 := Uint(U32)]"]]
        }
    }
}

#[test]
fn projection_equality_priority1() {
    test! {
        program {
            trait Trait1<T> {
                type Type;
            }

            struct S1 {}
            struct S2 {}
            struct S3 {}

            impl Trait1<S2> for S1 {
                type Type = u32;
            }
        }

        goal {
            exists<T, U> {
                S1: Trait1<T, Type = U>
            }
        } yields[SolverChoice::slg_default()] {
            // this is wrong, chalk#234
            expect![["Ambiguous; definite substitution for<?U0> { [?0 := S2, ?1 := ^0.0] }"]]
        } yields[SolverChoice::recursive_default()] {
            // This is.. interesting, but not necessarily wrong.
            // It's certainly true that based on the impls we see
            // the only possible value for `U` is `u32`.
            //
            // Can we come to any harm by inferring that `T = S2`
            // here, even though we could've chosen to say that
            // `U = !<S1 as Trait1<T>>::Type` and thus not
            // constrained `T` at all? I can't come up with
            // an example where that's the case, so maybe
            // not. -Niko
            expect![["Unique; substitution [?0 := S2, ?1 := Uint(U32)]"]]
        }
    }
}

#[test]
fn projection_equality_priority2() {
    test! {
        program {
            trait Trait1<T> {
                type Type;
            }

            struct S1 {}
            struct S2 {}
            struct S3 {}

            impl<X> Trait1<S1> for X {
                type Type = u32;
            }
        }

        goal {
            forall<X, Y> {
                if (X: Trait1<Y>) {
                    exists<Out1, Out2> {
                        X: Trait1<Out1, Type = Out2>
                    }
                }
            }
        } yields {
            // Correct: Ambiguous because Out1 = Y and Out1 = S1 are both value.
            expect![["Ambiguous; no inference guidance"]]
        }

        goal {
            forall<X, Y> {
                if (X: Trait1<Y>) {
                    exists<Out1, Out2> {
                        X: Trait1<Out1, Type = Out2>,
                        Out1 = Y
                    }
                }
            }
        } yields {
            // Constraining Out1 = Y gives us only one choice.
            expect![["Unique; substitution [?0 := !1_1, ?1 := (Trait1::Type)<!1_0, !1_1>]"]]
        }

        goal {
            forall<X, Y> {
                if (X: Trait1<Y>) {
                    exists<Out1, Out2> {
                        Out1 = Y,
                        X: Trait1<Out1, Type = Out2>
                    }
                }
            }
        } yields {
            // Constraining Out1 = Y gives us only one choice.
            expect![["Unique; substitution [?0 := !1_1, ?1 := (Trait1::Type)<!1_0, !1_1>]"]]
        }

        goal {
            forall<X, Y> {
                if (X: Trait1<Y>) {
                    exists<Out1, Out2> {
                        Out1 = S1,
                        X: Trait1<Out1, Type = Out2>
                    }
                }
            }
        } yields[SolverChoice::slg_default()] {
            // chalk#234: Constraining Out1 = S1 gives us only the choice to
            // use the impl, but the SLG solver can't decide between
            // the placeholder and the normalized form.
            expect![["Ambiguous; definite substitution for<?U1> { [?0 := S1, ?1 := ^0.0] }"]]
        } yields[SolverChoice::recursive_default()] {
            // Constraining Out1 = S1 gives us only one choice, use the impl,
            // and the recursive solver prefers the normalized form.
            expect![["Unique; substitution [?0 := S1, ?1 := Uint(U32)]"]]
        }
    }
}
#[test]
fn projection_equality_from_env() {
    test! {
        program {
            trait Trait1 {
                type Type;
            }
        }

        goal {
            forall<T> {
                if (T: Trait1<Type = u32>) {
                    exists<U> {
                        <T as Trait1>::Type = U
                    }
                }
            }
        } yields[SolverChoice::slg_default()] {
            // this is wrong, chalk#234
            expect![["Ambiguous; no inference guidance"]]
        } yields[SolverChoice::recursive_default()] {
            expect![["Unique; substitution [?0 := Uint(U32)]"]]
        }
    }
}

#[test]
fn projection_equality_nested() {
    test! {
        program {
            trait Iterator {
                type Item;
            }
        }

        goal {
            forall<I> {
                if (I: Iterator) {
                    if (<I as Iterator>::Item: Iterator<Item = u32>) {
                        exists<U> {
                            <<I as Iterator>::Item as Iterator>::Item = U
                        }
                    }
                }
            }
        } yields[SolverChoice::slg_default()] {
            // this is wrong, chalk#234
            expect![["Ambiguous; no inference guidance"]]
        }  yields[SolverChoice::recursive_default()] {
            expect![["Unique; substitution [?0 := Uint(U32)]"]]
        }
    }
}

#[test]
fn iterator_flatten() {
    test! {
        program {
            trait Iterator {
                type Item;
            }
            #[non_enumerable]
            trait IntoIterator {
                type Item;
                type IntoIter: Iterator<Item = <Self as IntoIterator>::Item>;
            }
            struct Flatten<I> {}

            impl<I, U> Iterator for Flatten<I>
            where
                I: Iterator,
                <I as Iterator>::Item: IntoIterator<IntoIter = U>,
                <I as Iterator>::Item: IntoIterator<Item = <U as Iterator>::Item>,
                U: Iterator
            {
                type Item = <U as Iterator>::Item;
            }
        }

        goal {
            forall<I, U> {
                if (I: Iterator<Item = U>; U: IntoIterator<Item = u32>) {
                    exists<T> {
                        <Flatten<I> as Iterator>::Item = T
                    }
                }
            }
        } yields[SolverChoice::slg_default()] {
            // this is wrong, chalk#234
            expect![["Ambiguous; no inference guidance"]]
        } yields[SolverChoice::recursive_default()] {
            expect![["Unique; substitution [?0 := Uint(U32)]"]]
        }
    }
}

#[test]
fn normalize_gat1() {
    test! {
        program {
            struct Vec<T> { }

            trait Iterable {
                type Iter<'a>;
            }

            impl<T> Iterable for Vec<T> {
                type Iter<'a> = Iter<'a, T>;
            }

            trait Iterator {
                type Item;
            }

            struct Iter<'a, T> { }
            struct Ref<'a, T> { }

            impl<'a, T> Iterator for Iter<'a, T> {
                type Item = Ref<'a, T>;
            }
        }

        goal {
            forall<T> {
                forall<'a> {
                    exists<U> {
                        Normalize(<Vec<T> as Iterable>::Iter<'a> -> U)
                    }
                }
            }
        } yields {
            expect![["Unique; substitution [?0 := Iter<'!2_0, !1_0>]"]]
        }
    }
}

#[test]
fn normalize_gat2() {
    test! {
        program {
            trait StreamingIterator<T> { type Item<'a>; }
            struct Span<'a, T> { }
            struct StreamIterMut<T> { }
            struct Foo { }
            impl<T> StreamingIterator<T> for StreamIterMut<T> {
                type Item<'a> = Span<'a, T>;
            }
        }

        goal {
            forall<'a, T> {
                exists<U> {
                    Normalize(<StreamIterMut<T> as StreamingIterator<T>>::Item<'a> -> U)
                }
            }
        } yields {
            expect![["Unique; substitution [?0 := Span<'!1_0, !1_1>]"]]
        }

        goal {
            forall<'a, T> {
                <StreamIterMut<T> as StreamingIterator<T>>::Item<'a> = Span<'a, T>
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<'a, T, U> {
                if (T: StreamingIterator<U, Item<'a> = Span<'a, U>>) {
                    <T as StreamingIterator<U>>::Item<'a> = Span<'a, U>
                }
            }
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn normalize_gat_const() {
    test! {
        program {
            trait StreamingIterator<T> { type Item<const N>; }
            struct Span<const N, T> { }
            struct StreamIterMut<T> { }
            impl<T> StreamingIterator<T> for StreamIterMut<T> {
                type Item<const N> = Span<N, T>;
            }
        }

        goal {
            forall<const N, T> {
                exists<U> {
                    Normalize(<StreamIterMut<T> as StreamingIterator<T>>::Item<N> -> U)
                }
            }
        } yields {
            expect![["Unique; substitution [?0 := Span<!1_0, !1_1>]"]]
        }

        goal {
            forall<const N, T> {
                <StreamIterMut<T> as StreamingIterator<T>>::Item<N> = Span<N, T>
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<const N, T, U> {
                if (T: StreamingIterator<U, Item<N> = Span<N, U>>) {
                    <T as StreamingIterator<U>>::Item<N> = Span<N, U>
                }
            }
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn normalize_gat_with_where_clause() {
    test! {
        program {
            trait Sized { }
            trait Foo {
                type Item<T> where T: Sized;
            }

            struct Value<T> { }
            struct Sometype { }
            impl Foo for Sometype {
                type Item<T> = Value<T>;
            }
        }

        goal {
            forall<T> {
                exists<U> {
                    Normalize(<Sometype as Foo>::Item<T> -> U)
                }
            }
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            forall<T> {
                exists<U> {
                    if (T: Sized) {
                        Normalize(<Sometype as Foo>::Item<T> -> U)
                    }
                }
            }
        } yields {
            expect![["Unique; substitution [?0 := Value<!1_0>]"]]
        }
    }
}

#[test]
fn normalize_gat_with_where_clause2() {
    test! {
        program {
            trait Bar<T> { }
            trait Foo<T> {
                type Item<U> where U: Bar<T>;
            }

            struct Baz { }
            impl<T> Foo<T> for Baz {
                type Item<U> = U;
            }
        }

        goal {
            forall<T, U> {
                exists<V> {
                    Normalize(<Baz as Foo<T>>::Item<U> -> V)
                }
            }
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            forall<T, U> {
                exists<V> {
                    if (U: Bar<T>) {
                        Normalize(<Baz as Foo<T>>::Item<U> -> V)
                    }
                }
            }
        } yields {
            expect![["Unique; substitution [?0 := !1_1]"]]
        }
    }
}

#[test]
fn normalize_gat_with_higher_ranked_trait_bound() {
    test! {
        program {
            trait Foo<'a, T> { }
            struct Baz { }

            trait Bar<'a, T> {
                type Item<V>: Foo<'a, T> where forall<'b> V: Foo<'b, T>;
            }

            impl<'a, T> Foo<'a, T> for Baz { }
            impl<'a, T> Bar<'a, T> for Baz {
                type Item<V> = Baz;
            }
        }

        goal {
            forall<'a, T, V> {
                if (forall<'b> { V: Foo<'b, T> }) {
                    exists<U> {
                        Normalize(<Baz as Bar<'a, T>>::Item<V> -> U)
                    }
                }
            }
        } yields {
            expect![["Unique; substitution [?0 := Baz]"]]
        }
    }
}

#[test]
fn gat_in_alias_in_alias_eq() {
    test! {
        program {
            trait Foo {
                type Rebind<U>: Foo;
            }

            struct S<T> { }
            impl<T> Foo for S<T> {
                type Rebind<U> = S<U>;
            }
        }

        goal {
            exists<T> {
                <<S<u32> as Foo>::Rebind<i32> as Foo>::Rebind<usize>: Foo
            }
        } yields {
            expect![[r#"Unique"#]]
        }
    }
}

#[test]
fn gat_bound_for_self_type() {
    test! {
        program {
            struct I32 { }
            trait Trait {
                type Assoc: Another<Gat<i32> = usize>;
            }
            trait Another {
                type Gat<T>;
            }
        }

        goal {
            forall<T> {
                exists<U> {
                    if (T: Trait) {
                        <<T as Trait>::Assoc as Another>::Gat<i32> = U
                    }
                }
            }
        } yields[SolverChoice::recursive_default()] {
            expect![[r#"Unique; substitution [?0 := Uint(Usize)]"#]]
        }
    }
}

#[test]
fn forall_projection() {
    test! {
        program {
            trait Eq<T> { }
            impl<T> Eq<T> for T { }

            trait DropLt<'a> { type Item; }
            impl<'a, T> DropLt<'a> for T { type Item = T; }

            struct Unit { }
            struct Ref<'a, T> { }
        }

        goal {
            for<'a> fn(<Unit as DropLt<'a>>::Item): Eq<fn(Unit)>
        } yields {
            expect![["Unique"]]
        }
    }
}

/// Demonstrates that, given the expected value of the associated
/// type, we can use that to narrow down the relevant impls.
#[test]
fn forall_projection_gat() {
    test! {
        program {
            trait Eq<T> { }
            impl<T> Eq<T> for T { }

            trait Sized { }

            trait DropOuter<'a> { type Item<U> where U: Sized; }
            impl<'a, T> DropOuter<'a> for T { type Item<U> = T; }

            struct Unit { }
            struct Ref<'a, T> { }
        }

        goal {
            forall<T> {
                for<'a> fn(<Unit as DropOuter<'a>>::Item<T>): Eq<fn(Unit)>
            }
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            forall<T> {
                if (T: Sized) {
                    for<'a> fn(<Unit as DropOuter<'a>>::Item<T>): Eq<fn(Unit)>
                }
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<'a, T> {
                WellFormed(<Unit as DropOuter<'a>>::Item<T>)
            }
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            forall<T> {
                if (T: Sized) {
                    WellFormed(for<'a> fn(<Unit as DropOuter<'a>>::Item<T>): Eq<fn(Unit)>)
                }
            }
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn gat_in_non_enumerable_trait() {
    test! {
        program {
            #[non_enumerable]
            trait Deref { }

            #[non_enumerable]
            trait PointerFamily {
                type Pointer<T>: Deref;
            }
        }

        goal {
            forall<T> {
                forall<U> {
                    if (T: PointerFamily) {
                        <T as PointerFamily>::Pointer<U>: Deref
                    }
                }
            }
        } yields {
            expect![[r#"Unique"#]]
        }
    }
}

#[test]
fn normalize_under_binder() {
    test! {
        program {
            struct Ref<'a, T> { }
            struct I32 { }

            trait Deref<'a> {
                type Item;
            }

            trait Id<'a> {
                type Item;
            }

            impl<'a, T> Deref<'a> for Ref<'a, T> {
                type Item = T;
            }

            impl<'a, T> Id<'a> for Ref<'a, T> {
                type Item = Ref<'a, T>;
            }
        }

        goal {
            exists<U> {
                forall<'a> {
                    Ref<'a, I32>: Deref<'a, Item = U>
                }
            }
        } yields[SolverChoice::slg_default()] {
            // chalk#234, I think
            expect![["Ambiguous; no inference guidance"]]
        } yields[SolverChoice::recursive_default()] {
            expect![["Unique; substitution [?0 := I32]"]]
        }

        goal {
            exists<U> {
                forall<'a> {
                    Normalize(<Ref<'a, I32> as Deref<'a>>::Item -> U)
                }
            }
        } yields {
            expect![["Unique; substitution [?0 := I32]"]]
        }

        goal {
            forall<'a> {
                exists<U> {
                    Ref<'a, I32>: Id<'a, Item = U>
                }
            }
        } yields[SolverChoice::slg_default()] {
            // chalk#234, I think
            expect![["Ambiguous; no inference guidance"]]
        } yields[SolverChoice::recursive_default()] {
            expect![["Unique; substitution [?0 := Ref<'!1_0, I32>]"]]
        }

        goal {
            forall<'a> {
                exists<U> {
                    Normalize(<Ref<'a, I32> as Id<'a>>::Item -> U)
                }
            }
        } yields {
            expect![["Unique; substitution [?0 := Ref<'!1_0, I32>]"]]
        }

        goal {
            exists<U> {
                forall<'a> {
                    Normalize(<Ref<'a, I32> as Id<'a>>::Item -> U)
                }
            }
        } yields {
            expect![["Unique; for<?U0> { \
             substitution [?0 := Ref<'^0.0, I32>], \
             lifetime constraints [\
             InEnvironment { environment: Env([]), goal: '!1_0: '^0.0 }, \
             InEnvironment { environment: Env([]), goal: '^0.0: '!1_0 }] \
             }"]]
        }
    }
}

#[test]
fn normalize_under_binder_multi() {
    test! {
        program {
            struct Ref<'a, T> { }
            struct I32 { }

            trait Deref<'a> {
                type Item;
            }

            impl<'a, T> Deref<'a> for Ref<'a, T> {
                type Item = T;
            }
        }

        goal {
            exists<U> {
                forall<'a> {
                    Ref<'a, I32>: Deref<'a, Item = U>
                }
            }
        } yields_all {
            expect![["substitution [?0 := I32]"]],
            expect![["for<?U0,?U0> { substitution [?0 := (Deref::Item)<Ref<'^0.0, I32>, '^0.1>], lifetime constraints [\
            InEnvironment { environment: Env([]), goal: '!1_0: '^0.1 }, \
            InEnvironment { environment: Env([]), goal: '^0.1: '!1_0 }, \
            InEnvironment { environment: Env([]), goal: '!1_0: '^0.0 }, \
            InEnvironment { environment: Env([]), goal: '^0.0: '!1_0 }] }"]]
        }

        goal {
            exists<U> {
                forall<'a> {
                    Ref<'a, I32>: Deref<'a, Item = U>
                }
            }
        } yields_first {
            expect![["substitution [?0 := I32]"]]
        }
    }
}

#[test]
fn projection_from_env_a() {
    test! {
        program {
            trait Sized { }

            struct Slice<T> where T: Sized { }
            impl<T> Sized for Slice<T> { }

            trait SliceExt
            {
                type Item;
            }

            impl<T> SliceExt for Slice<T>
            {
                type Item = T;
            }
        }

        goal {
            forall<T> {
                if (
                    <Slice<T> as SliceExt>::Item: Sized
                ) {
                    T: Sized
                }
            }
        } yields {
            expect![["Unique"]]
        }
    }
}

// This variant of the above test used to be achingly slow on SLG
// solvers, before the "trivial answer" green cut was introduced.
//
// The problem was that we wound up enumerating a goal like
//
//     <?0 as SliceExt>::Item = !1_0
//
// which meant "find me the types that normalize to `!1_0`". We had no
// problem finding these types, but after the first such type, we had
// the only unique answer we would ever find, and we wanted to reach
// the point where we could say "no more answers", so we kept
// requesting more answers.
#[test]
fn projection_from_env_slow() {
    test! {
        program {
            trait Clone { }
            trait Sized { }

            struct Slice<T> where T: Sized { }
            impl<T> Sized for Slice<T> { }

            struct Foo { }
            impl Clone for Foo { }
            impl Sized for Foo { }

            trait SliceExt
                where <Self as SliceExt>::Item: Clone
            {
                type Item;
            }

            impl<T> SliceExt for Slice<T>
                where T: Clone
            {
                type Item = T;
            }
        }

        goal {
            forall<T> {
                if (
                    <Slice<T> as SliceExt>::Item: Clone;
                    <Slice<T> as SliceExt>::Item: Sized;
                    T: Clone
                ) {
                    T: Sized
                }
            }
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn gat_unify_with_implied_wc() {
    test! {
        program {
            struct Slice<T> { }

            trait Cast<T> { }
            trait CastingIter<T> {
                type Item<U>: Cast<U> where T: Cast<U>;
            }

            impl<T> CastingIter<T> for Slice<T> {
                type Item<U> = Castable<T, U>;
            }

            struct Castable<T, U> { }
            impl<T, U> Cast<U> for Castable<T, U> { }
        }

        goal {
            forall<T, U, V> {
                if (
                    FromEnv(<Slice<T> as CastingIter<T>>::Item<U>)
                ) {
                    T: Cast<U>
                }
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<T, U, V> {
                T: Cast<U>
            }
        } yields {
            expect![["No possible solution"]]
        }
    }
}

// See rust-lang/chalk#280
#[test]
fn rust_analyzer_regression() {
    test! {
        program {
            trait FnOnce<Args> {
                type Output;
            }

            trait Try {
                type Ok;
                type Error;
            }

            struct Tuple<A, B> { }

            trait ParallelIterator {
                type Item;
            }
        }

        //fn try_reduce_with<PI, R, T>(pi: PI, reduce_op: R) -> Option<T>
        //    where
        //        PI: ParallelIterator<Item = T>,
        //        R: FnOnce(T::Ok) -> T,
        //        T: Try,
        //    {
        //        pi.drive_unindexed()
        //    }
        //
        // where `drive_unindexed` is a method in `ParallelIterator`:
        //
        // fn drive_unindexed(self) -> ();

        goal {
            forall<PI, R, T> {
                if (
                    PI: ParallelIterator<Item = T>;
                    R: FnOnce<Tuple< <T as Try>::Ok, <T as Try>::Ok >>;
                    T: Try
                ) {
                    PI: ParallelIterator
                }
            }
        } yields_first[SolverChoice::slg(4, None)] {
            expect![["Floundered"]]
        }
    }
}

#[test]
fn issue_144_regression() {
    test! {
        program {
            trait Bar { }
            trait Foo { type Item<T>: Bar; }
        }

        goal {
            forall<T, U> {
                if (T: Foo) {
                    <T as Foo>::Item<U>: Bar
                }
            }
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn guidance_for_projection_on_flounder() {
    test! {
        program {
            trait Iterator { type Item; }
            #[non_enumerable]
            trait Step {}

            struct Range<T> {}

            impl<T> Iterator for Range<T> where T: Step {
                type Item = T;
            }
        }

        goal {
            exists<T> {
                exists<U> {
                    <Range<T> as Iterator>::Item = U
                }
            }
        } yields[SolverChoice::recursive_default()] {
            expect![["Ambiguous; definite substitution for<?U0> { [?0 := ^0.0, ?1 := ^0.0] }"]]
        }
    }
}

#[test]
fn projection_to_dyn() {
    test! {
        program {
            trait AsDyn {
                type Dyn;
            }

            #[object_safe]
            trait Debug {}

            impl AsDyn for () {
                type Dyn = dyn Debug + 'static;
            }
        }

        goal {
            <() as AsDyn>::Dyn: Debug
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn projection_to_opaque() {
    test! {
        program {
            #[non_enumerable]
            trait Debug {
                type Output;
            }

            impl Debug for () {
                type Output = ();
            }

            opaque type OpaqueDebug: Debug<Output = ()> = ();

            struct A {}

            trait AsProj {
                type Proj;
            }

            impl AsProj for A {
                type Proj = OpaqueDebug;
            }
        }

        goal {
            <A as AsProj>::Proj: Debug
        } yields {
            expect![["Unique"]]
        }

        goal {
            <<A as AsProj>::Proj as Debug>::Output = ()
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn projection_from_super_trait_bounds() {
    test! {
        program {
            trait Foo {
                type A;
            }
            trait Bar where Self: Foo<A = ()> {}
            impl Foo for i32 {
                type A = ();
            }
            impl Bar for i32 {}
            opaque type Opaque: Bar = i32;
        }

        goal {
            forall<'a> {
                <dyn Bar + 'a as Foo>::A = ()
            }
        } yields {
            expect![[r#"Unique"#]]
        }

        goal {
            <Opaque as Foo>::A = ()
        } yields {
            expect![[r#"Unique"#]]
        }
    }
}

#[test]
fn nested_proj_eq_nested_proj_should_flounder() {
    test! {
        program {
            #[non_enumerable]
            trait Trait1 {
                type Assoc: Trait2;
            }
            #[non_enumerable]
            trait Trait2 {
                type Assoc;
            }

            impl Trait1 for () {
                type Assoc = ();
            }
            impl Trait1 for i32 {
                type Assoc = ();
            }
            impl Trait2 for () {
                type Assoc = ();
            }
        }

        goal {
            exists<T, U> {
                <<T as Trait1>::Assoc as Trait2>::Assoc = <<U as Trait1>::Assoc as Trait2>::Assoc
            }
        } yields[SolverChoice::slg_default()] {
            // FIXME
            expect![[r#"Ambiguous; definite substitution for<?U0> { [?0 := ^0.0, ?1 := ^0.0] }"#]]
        } yields[SolverChoice::recursive_default()] {
            expect![[r#"Ambiguous; no inference guidance"#]]
        }
    }
}

#[test]
fn clauses_for_placeholder_projection_types() {
    test! {
        program {
            trait Iterator { type Item; }
            trait IntoIterator {
                type Item;
                type IntoIter: Iterator<Item = <Self as IntoIterator>::Item>;
            }

            struct Vec<T> { }
            impl<T> IntoIterator for Vec<T> {
                type Item = T;
                type IntoIter = Iter<T>;
            }

            struct Iter<T> { }
            impl<T> Iterator for Iter<T> {
                type Item = T;
            }

            opaque type Opaque<T>: IntoIterator<Item = T> = Vec<T>;
        }

        goal {
            forall<T> {
                <Opaque<T> as IntoIterator>::IntoIter: Iterator
            }
        } yields {
            expect![[r#"Unique"#]]
        }

        goal {
            forall<T> {
                exists<U> {
                    <<Opaque<T> as IntoIterator>::IntoIter as Iterator>::Item = U
                }
            }
        } yields[SolverChoice::slg_default()] {
            // FIXME: chalk#234?
            expect![[r#"Ambiguous; no inference guidance"#]]
        } yields[SolverChoice::recursive_default()] {
            expect![[r#"Unique; substitution [?0 := !1_0]"#]]
        }
    }
}
