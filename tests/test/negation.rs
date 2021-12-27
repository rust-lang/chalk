//! Tests related to `not { }` goals.

use super::*;

#[test]
fn simple_negation() {
    test! {
        program {
            struct Bar {}
            trait Foo {}
        }

        goal {
            not { Bar: Foo }
        } yields {
            expect![["Unique"]]
        }

        goal {
            not {
                not { Bar: Foo }
            }
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            not {
                not {
                    not { Bar: Foo }
                }
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            exists<T> {
                not { T: Foo }
            }
        } yields {
            expect![["Ambiguous; no inference guidance"]]
        }

        goal {
            forall<T> {
                not { T: Foo }
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            not {
                exists<T> { T: Foo }
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            not {
                forall<T> { T: Foo }
            }
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn deep_negation() {
    test! {
        program {
            struct Foo<T> {}
            trait Bar {}
            trait Baz {}

            impl<T> Bar for Foo<T> where T: Baz {}
        }

        goal {
            not {
                exists<T> { T: Baz }
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            not {
                exists<T> { Foo<T>: Bar }
            }
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn negation_quantifiers() {
    test! {
        program {
            struct Alice {}
            struct Bob {}
        }

        goal {
            not {
                forall<T, U> {
                    T = U
                }
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            not {
                exists<T, U> {
                    T = U
                }
            }
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            forall<T, U> {
                not {
                    T = U
                }
            }
        } yields {
            expect![["No possible solution"]]
        }
    }
}

#[test]
fn negation_free_vars() {
    test! {
        program {
            struct Vec<T> {}
            struct Alice {}
            struct Bob {}
            trait Foo {}
            impl Foo for Vec<Bob> {}
        }

        goal {
            exists<T> {
                not { Vec<T>: Foo }
            }
        } yields {
            expect![["Ambiguous; no inference guidance"]]
        }
    }
}

/// Here, P and Q depend on one another through a negative loop.
#[test]
#[should_panic(expected = "negative cycle")]
fn negative_loop() {
    test! {
        program {
            trait P { }
            trait Q { }
            struct Alice { }

            forall<> { Alice: P if not { Alice: Q } }
            forall<> { Alice: Q if not { Alice: P } }
        }

        goal {
            Alice: P
        } yields_all[SolverChoice::slg(10, None)] {
            // Negative cycle -> panic
            expect![[""]]
        }
    }
}

#[test]
#[allow(non_snake_case)]
fn example_2_2_EWFS() {
    test! {
        program {
            trait M { }
            trait P { }
            trait Q { }

            struct a { }
            struct b { }
            struct c { }

            forall<X> { X: M if not { X: P } }
            forall<> { a: P }
            forall<X> { X: P if X: Q }
            forall<> { b: Q }
            forall<X> { X: Q if X: P }
        }

        goal {
            c: M
        } yields_all[SolverChoice::slg(3, None)] {
            expect![[""]]
        }
    }
}

#[test]
#[should_panic(expected = "negative cycle")]
#[allow(non_snake_case)]
fn example_2_3_EWFS() {
    test! {
        program {
            trait W { }
            trait M<A> { }
            trait P { }

            struct a { }
            struct b { }
            struct c { }

            forall<X, Y> { X: W if X: M<Y>, not { Y: W }, Y: P }
            forall<> { a: M<b> }
            forall<> { b: M<c> }
            forall<> { c: M<b> }
            forall<> { b: P }
        }

        goal {
            a: W
        } yields_all[SolverChoice::slg(3, None)] {
            // Negative cycle -> panic
            expect![[""]]
        }
    }
}

#[test]
#[should_panic(expected = "negative cycle")]
#[allow(non_snake_case)]
fn example_3_3_EWFS() {
    test! {
        program {
            trait S { }
            trait P { }
            trait Q { }

            struct a { }

            forall<> { a: S if not { a: P }, not { a: Q } }
            forall<> { a: P if not { a: S }, a: Q }
            forall<> { a: Q if not { a: S }, a: P }
        }

        goal {
            a: S
        } yields_all[SolverChoice::slg(3, None)] {
            // Negative cycle -> panic
            expect![[""]]
        }
    }
}

/// Here, P is neither true nor false. If it were true, then it would
/// be false, and so forth.
#[test]
#[should_panic(expected = "negative cycle")]
fn contradiction() {
    test! {
        program {
            trait P { }
            struct Alice { }

            forall<> { Alice: P if not { Alice: P } }
        }

        goal {
            Alice: P
        } yields_all[SolverChoice::slg(3, None)] {
            // Negative cycle -> panic
            expect![[""]]
        }
    }
}

/// Here, P depends on Q negatively, but Q depends only on itself.
#[test]
#[should_panic(expected = "negative cycle")]
fn negative_answer_ambiguous() {
    test! {
        program {
            trait P { }
            trait Q { }
            struct Alice { }

            forall<> { Alice: P if not { Alice: Q } }
            forall<> { Alice: Q if not { Alice: Q } }
        }

        goal {
            Alice: P
        } yields_all[SolverChoice::slg(3, None)] {
            // Negative cycle -> panic
            expect![[""]]
        }
    }
}

#[test]
fn negative_reorder() {
    test! {
        program {
            struct Foo { }
            struct Bar { }

            trait IsFoo { }
            impl IsFoo for Foo { }

            trait Enumerable { }
            impl Enumerable for Foo { }
            impl Enumerable for Bar { }

            // In this test, we first try to solve to solve `not { T:
            //  IsFoo }` but then we discover it's
            // non-enumerable, and so we push it off for later. Then
            // we try to solve the `T: Enumerable` trait.

            trait Debug1 { }
            forall<T> {
                T: Debug1 if T: Enumerable, not { T: IsFoo }
            }

            trait Debug2 { }
            forall<T> {
                T: Debug2 if not { T: IsFoo }, T: Enumerable
            }
        }

        goal {
            exists<A> { A: Debug1 }
        } yields_all[SolverChoice::slg(3, None)] {
            expect![["substitution [?0 := Bar]"]]
        }


        goal {
            exists<A> { A: Debug2 }
        } yields_all[SolverChoice::slg(3, None)] {
            expect![["substitution [?0 := Bar]"]]
        }
    }
}
