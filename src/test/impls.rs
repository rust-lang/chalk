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
