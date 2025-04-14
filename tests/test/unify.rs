//! Tests targeting the unification logic directly. Note that tests
//! related to *associated type normalization* are included in
//! `projection.rs`, however.

use super::*;

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
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!1_1 }, InEnvironment { environment: Env([]), goal: '!1_1: '!1_0 }]"]]
        }

        goal {
            forall<'a> {
                exists<'b> {
                    Ref<'a, Unit>: Eq<Ref<'b, Unit>>
                }
            }
        } yields {
            expect![["Unique; substitution [?0 := '!1_0]"]]
        }
    }
}

/// Temporary test extracted from the first goal in forall_equality for the sake of independent investigation.
#[test]
fn forall_equality_solveable_simple() {
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
            for<'a> fn(Ref<'a, Unit>): Eq<for<'c> fn(Ref<'c, Unit>)>
        } yields {
            expect![["Unique"]]
        }
    }
}

/// Temporary test extracted from the second goal in forall_equality for the sake of independent investigation.
#[test]
fn forall_equality_unsolveable_simple() {
    test! {
        program {
            trait Eq<T> { }
            impl<T> Eq<T> for T { }

            struct Unit { }
            struct Ref<'a, T> { }
        }

        goal {
            // Note: this equality is false, but we get back successful;
            // this is because the region constraints are unsolvable.
            //
            // Note that `?0` (in universe 2) must be equal to both
            // `!1_0` and `!1_1`, which of course it cannot be.
            for<'a, 'b> fn(Ref<'a, Ref<'b, Ref<'a, Unit>>>): Eq<
                for<'c, 'd> fn(Ref<'c, Ref<'d, Ref<'d, Unit>>>)>
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!1_1 }, InEnvironment { environment: Env([]), goal: '!1_1: '!1_0 }, InEnvironment { environment: Env([]), goal: '!2_0: '!2_1 }, InEnvironment { environment: Env([]), goal: '!2_1: '!2_0 }]"]]
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
            for<'a, 'b> fn(Ref<'a, Ref<'b, Unit>>): Eq<for<'c, 'd> fn(Ref<'c, Ref<'d, Unit>>)>
        } yields {
            expect![["Unique"]]
        }

        goal {
            // Note: this equality is false, but we get back successful;
            // this is because the region constraints are unsolvable.
            //
            // Note that `?0` (in universe 2) must be equal to both
            // `!1_0` and `!1_1`, which of course it cannot be.
            for<'a, 'b> fn(Ref<'a, Ref<'b, Ref<'a, Unit>>>): Eq<
                for<'c, 'd> fn(Ref<'c, Ref<'d, Ref<'d, Unit>>>)>
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!1_1 }, InEnvironment { environment: Env([]), goal: '!1_1: '!1_0 }, InEnvironment { environment: Env([]), goal: '!2_0: '!2_1 }, InEnvironment { environment: Env([]), goal: '!2_1: '!2_0 }]"]]
        }

        goal {
            // Function pointers with different ABIs should not be equal.
            extern "Rust" fn(): Eq<extern "C" fn()>
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            // Function pointers with identical ABIs should be equal.
            extern "Rust" fn(): Eq<extern "Rust" fn()>
        } yields {
            expect![["Unique"]]
        }

        goal {
            // Function pointers with different safety should not be equal.
            unsafe fn(): Eq<fn()>
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            // Function pointers with identical safety should be equal.
            unsafe fn(): Eq<unsafe fn()>
        } yields {
            expect![["Unique"]]
        }

        goal {
            // Function pointers with different parameters should not be equal.
            fn(u32): Eq<fn(u32, u32)>
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            // Variadic function pointers should not be equal to non-variadic fn pointers.
            fn(u8, ...): Eq<fn(u8)>
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            // Variadic function pointers should be equal to variadic fn pointers.
            fn(u8, ...): Eq<fn(u8, ...)>
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn unify_quantified_lifetimes() {
    test! {
        // Check that `'a` (here, `'^0.0`) is not unified
        // with `'!1_0`, because they belong to incompatible
        // universes.
        goal {
            exists<'a> {
                forall<'b> {
                    'a = 'b
                }
            }
        } yields {
            expect![["Unique; for<?U0> { \
             substitution [?0 := '^0.0], \
             lifetime constraints [\
             InEnvironment { environment: Env([]), goal: '!1_0: '^0.0 }, \
             InEnvironment { environment: Env([]), goal: '^0.0: '!1_0 }\
             ] \
             }"]]
        }

        // Similar to the previous test, but indirect.
        goal {
            exists<'a> {
                forall<'b> {
                    exists<'c> {
                        'a = 'c,
                        'c = 'b
                    }
                }
            }
        } yields[SolverChoice::slg(10, None)] {
            expect![["Unique; for<?U0> { \
             substitution [?0 := '^0.0, ?1 := '!1_0], \
             lifetime constraints [\
             InEnvironment { environment: Env([]), goal: '!1_0: '^0.0 }, \
             InEnvironment { environment: Env([]), goal: '^0.0: '!1_0 }\
             ] \
             }"]]
        } yields[SolverChoice::recursive_default()] {
            // only difference is in the value of ?1, which is equivalent
            expect![["Unique; for<?U0> { \
             substitution [?0 := '^0.0, ?1 := '^0.0], \
             lifetime constraints [\
             InEnvironment { environment: Env([]), goal: '!1_0: '^0.0 }, \
             InEnvironment { environment: Env([]), goal: '^0.0: '!1_0 }\
             ] \
             }"]]
        }
    }
}

#[test]
fn equality_binder() {
    test! {
        program {
            struct Ref<'a, T> { }
        }

        // Check that `'a` (here, `'?0`) is not unified
        // with `'!1_0`, because they belong to incompatible
        // universes.
        goal {
            forall<T> {
                exists<'a> {
                    for<'c> fn(Ref<'c, T>) = fn(Ref<'a, T>)
                }
            }
        } yields {
            expect![["Unique; for<?U1> { \
                 substitution [?0 := '^0.0], \
                 lifetime constraints [\
                 InEnvironment { environment: Env([]), goal: '!2_0: '^0.0 }, \
                 InEnvironment { environment: Env([]), goal: '^0.0: '!2_0 }\
                 ] \
             }"]]
        }
    }
}

#[test]
fn equality_binder2() {
    test! {
        program {
            struct Ref<'a, 'b> { }
        }

        goal {
            for<'b, 'c> fn(Ref<'b, 'c>) = for<'a> fn(Ref<'a, 'a>)
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!1_1 }, InEnvironment { environment: Env([]), goal: '!1_1: '!1_0 }]"]]
        }

        goal {
            for<'a> fn(Ref<'a, 'a>) = for<'b, 'c> fn(Ref<'b, 'c>)
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: '!2_0: '!2_1 }, InEnvironment { environment: Env([]), goal: '!2_1: '!2_0 }]"]]
        }
    }
}

#[test]
fn mixed_indices_unify() {
    test! {
        program {
            struct Ref<'a, T> { }
        }

        goal {
            exists<T> {
                exists<'a> {
                    exists<U> {
                        Ref<'a, T> = Ref<'a, U>
                    }
                }
            }
        } yields {
            expect![["Unique; for<?U0,?U0> { substitution [?0 := '^0.0, ?1 := ^0.1, ?2 := ^0.1] }"]]
        }
    }
}

#[test]
fn mixed_indices_match_program() {
    test! {
        program {
            struct S { }
            struct Bar<'a, T, U> { }
            trait Foo {}
            impl<'a> Foo for Bar<'a, S, S> {}
        }

        goal {
            exists<T> {
                exists<'a> {
                    exists<U> {
                        Bar<'a, T, U>: Foo
                    }
                }
            }
        } yields {
            expect![["Unique; for<?U0> { substitution [?0 := '^0.0, ?1 := S, ?2 := S] }"]]
        }
    }
}

#[test]
fn mixed_indices_normalize_application() {
    test! {
        program {
            struct Ref<'a, T> { }
            trait Foo {
                type T;
            }

            impl<U, 'a> Foo for Ref<'a, U> {
                type T = U;
            }
        }

        goal {
            exists<T> {
                exists<'a> {
                    exists<U> {
                        Normalize(<Ref<'a, T> as Foo>::T -> U)
                    }
                }
            }
        } yields {
            expect![["Unique; for<?U0,?U0> { substitution [?0 := '^0.0, ?1 := ^0.1, ?2 := ^0.1] }"]]
        }
    }
}

#[test]
fn mixed_indices_normalize_gat_application() {
    test! {
        program {
            struct Either<T, U> { }
            struct Ref<'a, T> { }
            trait Foo {
                type T<X>;
            }

            impl<U, 'a> Foo for Ref<'a, U> {
                type T<X> = Either<X, U>;
            }
        }

        goal {
            exists<T, 'a, U, Y, X> {
                Normalize(<Ref<'a, T> as Foo>::T<X> -> Either<U, Y>)
            }
        } yields {
            expect![["Unique; for<?U0,?U0,?U0> { substitution [?0 := '^0.0, ?1 := ^0.1, ?2 := ^0.2, ?3 := ^0.2, ?4 := ^0.1] }"]]
        }
    }
}

#[test]
fn quantified_types() {
    test! {
        program {
            trait Foo { }
            struct fn1<'a> { }
            struct fn2<'a, 'b> { }
            impl Foo for for<'a> fn(fn1<'a>) { }
        }

        goal {
            for<'a> fn(fn1<'a>): Foo
        } yields {
            expect![["Unique"]]
        }

        goal {
            for<'a, 'b> fn(fn2<'a, 'b>) = for<'b, 'a> fn(fn2<'a, 'b>)
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<'a> { fn(fn1<'a>): Foo }
        } yields {
            // Lifetime constraints are unsatisfiable
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!2_0 }, InEnvironment { environment: Env([]), goal: '!2_0: '!1_0 }]"]]
        }
    }
}
