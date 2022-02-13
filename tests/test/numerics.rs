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
            expect![["Unique; substitution [?0 := Uint(Usize)]"]]
        }
    }
}

/// A more straightforward version of the `integer_index` test where the
/// variable is on the impl side of the trait ref.
#[test]
fn integer_kind_trait() {
    test! {
        program {
            // this should even work for non-enumerable traits, because we don't
            // need to enumerate *all* impls for this!
            #[non_enumerable]
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
            expect![["Unique; substitution [?0 := Uint(Usize)]"]]
        }
    }
}

/// The `integer_kind_trait` test, but for floats
#[test]
fn float_kind_trait() {
    test! {
        program {
            #[non_enumerable]
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
            expect![["Unique; substitution [?0 := Float(F32)]"]]
        }
    }
}

/// You can still get ambiguous results with integer variables
#[test]
fn integer_ambiguity() {
    test! {
        program {
            trait Foo {}

            impl Foo for usize {}
            impl Foo for isize {}
        }

        goal {
            exists<int N> {
                N: Foo
            }
        } yields {
            expect![["Ambiguous; no inference guidance"]]
        }
    }
}

/// You can still get ambiguous results with float variables
#[test]
fn float_ambiguity() {
    test! {
        program {
            trait Foo {}

            impl Foo for f32 {}
            impl Foo for f64 {}
        }

        goal {
            exists<float N> {
                N: Foo
            }
        } yields {
            expect![["Ambiguous; no inference guidance"]]
        }
    }
}

/// Integer/float type kinds are just specialized type kinds, so they can unify
/// with general type kinds.
#[test]
fn integer_and_float_are_specialized_ty_kinds() {
    test! {
        goal {
            exists<T, int N> {
                T = N, N = usize
            }
        } yields {
            expect![["Unique; substitution [?0 := Uint(Usize), ?1 := Uint(Usize)]"]]
        }

        goal {
            exists<T, float N> {
                T = N, N = f32
            }
        } yields {
            expect![["Unique; substitution [?0 := Float(F32), ?1 := Float(F32)]"]]
        }
    }
}

/// Once a general type kind is unified with a specific type kind, it cannot be
/// unified with an incompatible type (ex. integer type kind with char)
#[test]
fn general_ty_kind_becomes_specific() {
    test! {
        goal {
            exists<T, int N> {
                T = N, T = char
            }
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            exists<T, float N> {
                T = N, T = char
            }
        } yields {
            expect![["No possible solution"]]
        }
    }
}

/// Integer and float type kinds can not be equated
#[test]
fn integers_are_not_floats() {
    test! {
        goal {
            exists<int I, float F> {
                I = F
            }
        } yields {
            expect![["No possible solution"]]
        }
    }
}

#[test]
fn integers_are_copy() {
    test! {
        program {
            #[lang(copy)]
            trait Copy { }
        }

        goal {
            exists<int I> {
                I: Copy
            }
        } yields {
            expect![["Unique; for<?U0> { substitution [?0 := ^0.0] }"]]
        }
    }
}

#[test]
fn integers_are_sized() {
    test! {
        program {
            #[lang(sized)]
            trait Sized { }
        }

        goal {
            exists<int I> {
                I: Sized
            }
        } yields {
            expect![["Unique; for<?U0> { substitution [?0 := ^0.0] }"]]
        }
    }
}

/// Simplified version of a goal that needs to be solved for type checking
/// `1 + 2`.
#[test]
fn ambiguous_add() {
    test! {
        program {
            #[non_enumerable]
            trait Add<Rhs> { type Output; }

            impl<'a> Add<&'a u32> for u32 { type Output = <u32 as Add<u32>>::Output; }
            impl Add<u32> for u32 { type Output = u32; }
        }

        goal {
            exists<int T, U, V> {
                <T as Add<U>>::Output = V
            }
        } yields {
            expect![["Ambiguous; no inference guidance"]]
        }
    }
}

/// Simplified version of a goal that needs to be solved for type checking
/// `1 << &2`.
#[test]
fn shl_ice() {
    test! {
        program {
            //#[non_enumerable]
            trait Shl<Rhs> { }

            impl<'a> Shl<&'a u32> for u32 { }
            impl<'a> Shl<&'a u16> for u32 { }
        }

        goal {
            exists<U> {
                u32: Shl<U>
            }
        } yields[SolverChoice::slg_default()] {
            expect![["Ambiguous; definite substitution for<?U0,?U0> { [?0 := (&'^0.0 ^0.1)] }"]]
        } yields[SolverChoice::recursive_default()] {
            expect![["Ambiguous; no inference guidance"]]
        }
    }
}

/// Regression test for rust-analyzer#5495 ("var_universe invoked on bound
/// variable" crash).
#[test]
fn unify_general_then_specific_ty() {
    test! {
        program {
            #[non_enumerable]
            trait Foo {}
            struct Bar<T> {}

            impl<T> Foo for Bar<(T, T, i32, i32)> {}
        }

        goal {
            exists<T, int N> {
                Bar<(N, T, T, T)>: Foo
            }
        } yields {
            expect![["Unique; substitution [?0 := Int(I32), ?1 := Int(I32)]"]]
        }
    }
}
