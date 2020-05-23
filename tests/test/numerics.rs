//! Tests related to integer/float variable kinds

use super::*;

/// If we know that the type is an integer, we can narrow down the possible
/// types. This test is based on the following example:
/// ```ignore
/// let x: &[u32];
/// let i = 1;
/// x[i]
/// ```
/// `i` must be `usize` because that is the only integer type used in `Index`
/// impls for slices.
#[test]
fn integer_index() {
    test! {
        program {
            trait Index<T> {}
            struct Slice {}
            struct Foo {}

            impl Index<usize> for Slice {}
            impl Index<Foo> for Slice {}
        }

        goal {
            exists<int N> {
                Slice: Index<N>
            }
        } yields {
            "Unique; substitution [?0 := Uint(Usize)]"
        }
    }
}

/// A more straightforward version of the `integer_index` test where the
/// variable is on the impl side of the trait ref.
#[test]
fn integer_kind_trait() {
    test! {
        program {
            trait Foo {}
            struct Bar {}

            impl Foo for usize {}
            impl Foo for Bar {}
        }

        goal {
            exists<int N> {
                N: Foo
            }
        } yields {
            "Unique; substitution [?0 := Uint(Usize)]"
        }
    }
}

/// The `integer_kind_trait` test, but for floats
#[test]
fn float_kind_trait() {
    test! {
        program {
            trait Foo {}
            struct Bar {}

            impl Foo for f32 {}
            impl Foo for Bar {}
        }

        goal {
            exists<float N> {
                N: Foo
            }
        } yields {
            "Unique; substitution [?0 := Float(F32)]"
        }
    }
}
