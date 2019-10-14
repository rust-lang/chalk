//! Tests related to the basic conversion of impls into logical predicates
//! and other core logic functions.

use super::*;

#[test]
fn prove_clone() {
    test! {
        program {
            struct Foo { }
            struct Bar { }
            struct Vec<T> { }
            trait Clone { }
            impl<T> Clone for Vec<T> where T: Clone { }
            impl Clone for Foo { }
        }

        goal {
            Vec<Foo>: Clone
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            Foo: Clone
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            Bar: Clone
        } yields {
            "No possible solution"
        }

        goal {
            Vec<Bar>: Clone
        } yields {
            "No possible solution"
        }
    }
}

/// Test that given `?0: Map<?1>` where *either* `?0` or `?1` is
/// known, we can infer the other (but if neither is known, we get an
/// ambiguous result).
///
/// In rustc today, if `?0` is not known we will not attempt to match
/// impls.
#[test]
fn prove_infer() {
    test! {
        program {
            struct Foo { }
            struct Bar { }
            trait Map<T> { }
            impl Map<Bar> for Foo { }
            impl Map<Foo> for Bar { }
        }

        goal {
            exists<A, B> { A: Map<B> }
        } yields {
            "Ambiguous; no inference guidance"
        }

        goal {
            exists<A> { A: Map<Bar> }
        } yields {
            "Unique; substitution [?0 := Foo], lifetime constraints []"
        }

        goal {
            exists<A> { Foo: Map<A> }
        } yields {
            "Unique; substitution [?0 := Bar], lifetime constraints []"
        }
    }
}

/// Test the interaction of `forall` goals and impls. For example,
/// test that we can prove things like
///
/// ```notrust
/// forall<T> { Vec<T>: Marker }
/// ```
///
/// given a suitably generic impl.
#[test]
fn prove_forall() {
    test! {
        program {
            struct Foo { }
            struct Vec<T> { }

            trait Marker { }
            impl<T> Marker for Vec<T> { }

            trait Clone { }
            impl Clone for Foo { }

            impl<T> Clone for Vec<T> where T: Clone { }
        }

        goal {
            forall<T> { T: Marker }
        } yields {
            "No possible solution"
        }

        goal {
            forall<T> { not { T: Marker } }
        } yields {
            "No"
        }

        goal {
            not { forall<T> { T: Marker } }
        } yields {
            "Unique"
        }

        // If we assume `T: Marker`, then obviously `T: Marker`.
        goal {
            forall<T> { if (T: Marker) { T: Marker } }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        // We don't have to know anything about `T` to know that
        // `Vec<T>: Marker`.
        goal {
            forall<T> { Vec<T>: Marker }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        // Here, we don't know that `T: Clone`, so we can't prove that
        // `Vec<T>: Clone`.
        goal {
            forall<T> { Vec<T>: Clone }
        } yields {
            "No possible solution"
        }

        // Here, we do know that `T: Clone`, so we can.
        goal {
            forall<T> {
                if (T: Clone) {
                    Vec<T>: Clone
                }
            }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}

#[test]
fn higher_ranked() {
    test! {
        program {
            struct u8 { }
            struct SomeType<T> { }
            trait Foo<T> { }
            impl<U> Foo<u8> for SomeType<U> { }
        }

        goal {
            exists<V> {
                forall<U> {
                    SomeType<U>: Foo<V>
                }
            }
        } yields {
            "Unique; substitution [?0 := u8], lifetime constraints []"
        }
    }
}

#[test]
fn ordering() {
    test! {
        program {
            trait Foo<T> { }
            impl<U> Foo<U> for U { }
        }

        goal {
            exists<V> {
                forall<U> {
                    U: Foo<V>
                }
            }
        } yields {
            "No possible solution"
        }
    }
}

/// Demonstrates that, given the expected value of the associated
/// type, we can use that to narrow down the relevant impls.
#[test]
fn normalize_rev_infer() {
    test! {
        program {
            trait Identity { type Item; }
            struct u32 { }
            struct i32 { }
            impl Identity for u32 { type Item = u32; }
            impl Identity for i32 { type Item = i32; }
        }

        goal {
            exists<T> {
                T: Identity<Item = u32>
            }
        } yields {
            "Unique; substitution [?0 := u32]"
        }
    }
}

/// Demonstrates that, given the expected value of the associated
/// type, we can use that to narrow down the relevant impls.
#[test]
fn normalize_rev_infer_gat() {
    test! {
        program {
            trait Combine { type Item<T>; }
            struct u32 { }
            struct i32 { }
            struct Either<T, U> { }
            impl Combine for u32 { type Item<U> = Either<u32, U>; }
            impl Combine for i32 { type Item<U> = Either<i32, U>; }
        }

        goal {
            exists<T, U> {
                T: Combine<Item<U> = Either<u32, i32>>
            }
        } yields {
            // T is ?1 and U is ?0, so this is surprising, but correct! (See #126.)
            "Unique; substitution [?0 := i32, ?1 := u32]"
        }
    }
}

/// Basic tests of region equality: we generate constraints.
#[test]
fn region_equality() {
    test! {
        program {
            trait Eq<T> { }
            impl<T> Eq<T> for T { }

            struct Unit { }
            struct Ref<'a, T> { }
        }

        goal {
            forall<'a, 'b> {
                Ref<'a, Unit>: Eq<Ref<'b, Unit>>
            }
        } yields {
            "Unique; substitution [],
                     lifetime constraints \
                     [InEnvironment { environment: Env([]), goal: '!1_1 == '!1_0 }]
                     "
        }

        goal {
            forall<'a> {
                exists<'b> {
                    Ref<'a, Unit>: Eq<Ref<'b, Unit>>
                }
            }
        } yields {
            "Unique; substitution [?0 := '!1_0], lifetime constraints []"
        }
    }
}

/// Tests of region equality and "foralls" -- we generate contraints that are sometimes
/// not solvable.
#[test]
fn forall_equality() {
    test! {
        program {
            trait Eq<T> { }
            impl<T> Eq<T> for T { }

            struct Unit { }
            struct Ref<'a, T> { }
        }

        goal {
            // A valid equality; we get back a series of solvable
            // region constraints, since each region variable must
            // refer to exactly one placeholder region, and they are
            // all in a valid universe to do so (universe 4).
            for<'a, 'b> Ref<'a, Ref<'b, Unit>>: Eq<for<'c, 'd> Ref<'c, Ref<'d, Unit>>>
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            // Note: this equality is false, but we get back successful;
            // this is because the region constraints are unsolvable.
            //
            // Note that `?0` (in universe 2) must be equal to both
            // `!1_0` and `!1_1`, which of course it cannot be.
            for<'a, 'b> Ref<'a, Ref<'b, Ref<'a, Unit>>>: Eq<
                for<'c, 'd> Ref<'c, Ref<'d, Ref<'d, Unit>>>>
        } yields {
            "Unique; substitution [], lifetime constraints [
                 InEnvironment { environment: Env([]), goal: '!1_1 == '!1_0 }
             ]"
        }
    }
}
