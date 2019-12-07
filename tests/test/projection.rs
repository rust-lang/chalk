//! Tests related to projection of associated types and normalization.

use super::*;

#[test]
fn normalize_basic() {
    test! {
        program {
            trait Iterator { type Item; }
            struct Vec<T> { }
            struct u32 { }
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
            "Unique; substitution [?0 := !1_0], lifetime constraints []"
        }

        goal {
            forall<T> {
                Vec<T>: Iterator<Item = T>
            }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            forall<T> {
                if (T: Iterator<Item = u32>) {
                    <T as Iterator>::Item = u32
                }
            }
        } yields {
            "Unique; substitution []"
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
            "Unique; substitution [?0 := (Iterator::Item)<!1_0>]"
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
            "Unique; substitution [?0 := (Iterator::Item)<!1_0>]"
        }

        goal {
            forall<T> {
                if (T: Iterator) {
                    <T as Iterator>::Item = <T as Iterator>::Item
                }
            }
        } yields {
            "Unique"
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
            // True for `U = T`, of course, but also true for `U = Vec<T>`.
            "Ambiguous"
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
            struct u32 { }
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
            "Unique"
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

            struct u32 {}
            struct S {}
            impl Trait1 for S {
                type Type = u32;
            }
        }

        goal {
            exists<U> {
                S: Trait2<U>
            }
        } yields {
            // FIXME(rust-lang/chalk#234) -- there is really only one
            // *reasonable* solution here, which is `u32`, but we get
            // confused because `(Trait1::Type)<S>` seems valid too.
            "Ambiguous; no inference guidance"
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
            "Unique; substitution [?0 := Iter<'!2_0, !1_0>], lifetime constraints []"
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
            struct u32 { }
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
            "Unique; substitution [?0 := Span<'!1_0, !1_1>], lifetime constraints []"
        }

        goal {
            forall<'a, T> {
                <StreamIterMut<T> as StreamingIterator<T>>::Item<'a> = Span<'a, T>
            }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            forall<'a, T, U> {
                if (T: StreamingIterator<U, Item<'a> = Span<'a, U>>) {
                    <T as StreamingIterator<U>>::Item<'a> = Span<'a, U>
                }
            }
        } yields {
            "Unique; substitution [], lifetime constraints []"
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
            "No possible solution"
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
            "Unique; substitution [?0 := Value<!1_0>]"
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

            struct i32 { }
            impl<T> Foo<T> for i32 {
                type Item<U> = U;
            }
        }

        goal {
            forall<T, U> {
                exists<V> {
                    Normalize(<i32 as Foo<T>>::Item<U> -> V)
                }
            }
        } yields {
            "No possible solution"
        }

        goal {
            forall<T, U> {
                exists<V> {
                    if (U: Bar<T>) {
                        Normalize(<i32 as Foo<T>>::Item<U> -> V)
                    }
                }
            }
        } yields {
            "Unique; substitution [?0 := !1_1]"
        }
    }
}

#[test]
fn normalize_gat_with_higher_ranked_trait_bound() {
    test! {
        program {
            trait Foo<'a, T> { }
            struct i32 { }

            trait Bar<'a, T> {
                type Item<V>: Foo<'a, T> where forall<'b> V: Foo<'b, T>;
            }

            impl<'a, T> Foo<'a, T> for i32 { }
            impl<'a, T> Bar<'a, T> for i32 {
                type Item<V> = i32;
            }
        }

        goal {
            forall<'a, T, V> {
                if (forall<'b> { V: Foo<'b, T> }) {
                    exists<U> {
                        Normalize(<i32 as Bar<'a, T>>::Item<V> -> U)
                    }
                }
            }
        } yields {
            "Unique; substitution [?0 := i32], lifetime constraints []"
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
            for<'a> <Unit as DropLt<'a>>::Item: Eq<for<> Unit>
        } yields {
            "Unique; substitution [], lifetime constraints []"
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
                for<'a> <Unit as DropOuter<'a>>::Item<T>: Eq<for<> Unit>
            }
        } yields {
            "No possible solution"
        }

        goal {
            forall<T> {
                if (T: Sized) {
                    for<'a> <Unit as DropOuter<'a>>::Item<T>: Eq<for<> Unit>
                }
            }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            forall<'a, T> {
                WellFormed(<Unit as DropOuter<'a>>::Item<T>)
            }
        } yields {
            "No possible solution"
        }

        goal {
            forall<T> {
                if (T: Sized) {
                    WellFormed(for<'a> <Unit as DropOuter<'a>>::Item<T>: Eq<for<> Unit>)
                }
            }
        } yields {
            "Unique; substitution [], lifetime constraints []"
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
        } yields {
            "Ambiguous"
        }

        goal {
            exists<U> {
                forall<'a> {
                    Normalize(<Ref<'a, I32> as Deref<'a>>::Item -> U)
                }
            }
        } yields {
            "Unique; substitution [?0 := I32], lifetime constraints []"
        }

        goal {
            forall<'a> {
                exists<U> {
                    Ref<'a, I32>: Id<'a, Item = U>
                }
            }
        } yields {
            "Ambiguous"
        }

        goal {
            forall<'a> {
                exists<U> {
                    Normalize(<Ref<'a, I32> as Id<'a>>::Item -> U)
                }
            }
        } yields {
            "Unique; substitution [?0 := Ref<'!1_0, I32>], lifetime constraints []"
        }

        goal {
            exists<U> {
                forall<'a> {
                    Normalize(<Ref<'a, I32> as Id<'a>>::Item -> U)
                }
            }
        } yields {
            "Unique; for<?U0> { \
             substitution [?0 := Ref<'^0, I32>], \
             lifetime constraints [InEnvironment { environment: Env([]), goal: '^0 == '!1_0 }] \
             }"
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
            "substitution [?0 := I32], lifetime constraints []",
            "for<?U0,?U0> { substitution [?0 := (Deref::Item)<Ref<'^0, I32>, '^1>], lifetime constraints [InEnvironment { environment: Env([]), goal: '^0 == '!1_0 }, InEnvironment { environment: Env([]), goal: '^1 == '!1_0 }] }"
        }

        goal {
            exists<U> {
                forall<'a> {
                    Ref<'a, I32>: Deref<'a, Item = U>
                }
            }
        } yields_first {
            "substitution [?0 := I32], lifetime constraints []"
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
            "Unique"
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

            struct u32 { }
            impl Clone for u32 { }
            impl Sized for u32 { }

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
            "Unique"
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
            "Unique"
        }

        goal {
            forall<T, U, V> {
                T: Cast<U>
            }
        } yields {
            "No possible solution"
        }
    }
}

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
        } yields[SolverChoice::SLG { max_size: 4 }] {
            "substitution [], lifetime constraints []"
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
            "Unique"
        }
    }
}
