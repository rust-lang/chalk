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
            for<'a, 'b> fn(Ref<'a, Ref<'b, Unit>>): Eq<for<'c, 'd> fn(Ref<'c, Ref<'d, Unit>>)>
        } yields {
            "Unique; substitution [], lifetime constraints []"
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
            "Unique; substitution [], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_1 == '!1_0 }, InEnvironment { environment: Env([]), goal: '!2_1 == '!2_0 }]"
        }
    }
}

#[test]
fn unify_quantified_lifetimes() {
    test! {
        program {
        }

        // Check that `'a` (here, `'^0`) is not unified
        // with `'!1_0`, because they belong to incompatible
        // universes.
        goal {
            exists<'a> {
                forall<'b> {
                    'a = 'b
                }
            }
        } yields {
            "Unique; for<?U0> { \
             substitution [?0 := '^0], \
             lifetime constraints [InEnvironment { environment: Env([]), goal: '^0 == '!1_0 }] \
             }"
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
        } yields {
            "Unique; for<?U0> { \
             substitution [?0 := '^0, ?1 := '!1_0], \
             lifetime constraints [InEnvironment { environment: Env([]), goal: '^0 == '!1_0 }] \
             }"
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
                    for<'c> fn(Ref<'c, T>) = for<> fn(Ref<'a, T>)
                }
            }
        } yields {
            "Unique; for<?U1> { \
                 substitution [?0 := '^0], \
                 lifetime constraints [InEnvironment { environment: Env([]), goal: '!2_0 == '^0 }] \
             }"
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
            "Unique; substitution [], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_1 == '!1_0 }]"
        }

        goal {
            for<'a> fn(Ref<'a, 'a>) = for<'b, 'c> fn(Ref<'b, 'c>)
        } yields {
            "Unique; substitution [], lifetime constraints [InEnvironment { environment: Env([]), goal: '!2_0 == '!2_1 }]"
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
            "Unique; for<?U0,?U0> { \
                 substitution [?0 := '^0, ?1 := ^1, ?2 := ^1], \
                 lifetime constraints []\
             }"
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
            "Unique; for<?U0> { \
                 substitution [?0 := '^0, ?1 := S, ?2 := S], \
                 lifetime constraints [] \
             }"
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
            "Unique; for<?U0,?U0> { substitution [?0 := '^0, ?1 := ^1, ?2 := ^1], "
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
            // Our GAT parameter <X> is mapped to ?0; all others appear left to right
            // in our Normalize(...) goal.
            "Unique; for<?U0,?U0,?U0> { \
                substitution [?0 := ^0, ?1 := '^1, ?2 := ^2, ?3 := ^0, ?4 := ^2], "
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
            "Unique"
        }

        goal {
            for<'a, 'b> fn(fn2<'a, 'b>) = for<'b, 'a> fn(fn2<'a, 'b>)
        } yields {
            "Unique"
        }

        goal {
            forall<'a> { for<> fn(fn1<'a>): Foo }
        } yields {
            // Lifetime constraints are unsatisfiable
            "Unique; substitution [], \
            lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0 == '!3_0 }]"
        }
    }
}
