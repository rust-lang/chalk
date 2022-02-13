//! Tests targeting coinduction specifically

use super::*;

#[test]
fn mixed_semantics() {
    test! {
        program {
            #[coinductive] trait Send { }
            trait Foo { }

            struct Bar { }

            impl Send for Bar where Bar: Foo { }
            impl Foo for Bar where Bar: Send { }
        }

        // We have a cycle `(T: Send) :- (T: Foo) :- (T: Send)` with a non-coinductive
        // inner component `T: Foo` so we reject it.
        goal {
            Bar: Send
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            Bar: Foo
        } yields {
            expect![["No possible solution"]]
        }
    }
}

#[test]
fn coinductive_unification_forall() {
    test! {
        program {
            #[coinductive]
            trait C1 { }
            #[coinductive]
            trait C2 { }
            #[coinductive]
            trait C3 { }

            struct X { }
            struct Y { }

            forall<T> { T: C1 if T: C2, T = X }
            forall<T> { T: C2 if T: C3, T = Y }
            forall<T> { T: C3 if T: C1, T: C2 }
        }

        goal {
            forall<T> { T: C1 }
        } yields {
            expect![["No possible solution"]]
        }
    }
}

#[test]
fn coinductive_unification_exists() {
    test! {
        program {
            #[coinductive]
            trait C1 { }
            #[coinductive]
            trait C2 { }
            #[coinductive]
            trait C3 { }

            struct X { }
            struct Y { }

            forall<T> { T: C1 if T: C2, T = X }
            forall<T> { T: C2 if T: C3, T = Y }
            forall<T> { T: C3 if T: C1, T: C2 }
        }

        goal {
            exists<T> { T: C1 }
        } yields {
            expect![["No possible solution"]]
        }
    }
}

#[test]
fn coinductive_nontrivial() {
    test! {
        program {
            #[coinductive]
            trait C1 { }
            trait C2 { }

            struct X { }
            struct Y { }

            forall<A, B> { A: C1 if B: C1, B = X, A: C2 }
            impl C2 for Y { }
        }

        goal {
            exists<T> { T: C1 }
        } yields {
            expect![["No possible solution"]]
        }
    }
}

#[test]
fn coinductive_trivial_variant1() {
    test! {
        program {
            #[coinductive]
            trait C1<T> { }
            #[coinductive]
            trait C2<T> { }

            struct X { }

            forall<A, B> { A: C1<B> if A: C2<B>, A = X, B = X }
            forall<A, B> { A: C2<B> if B: C1<A> }
        }

        goal {
            exists<T, U> { T: C1<U> }
        } yields {
            expect![["Unique; substitution [?0 := X, ?1 := X]"]]
        }
    }
}

#[test]
fn coinductive_trivial_variant2() {
    test! {
        program {
            #[coinductive]
            trait C1<T> { }
            #[coinductive]
            trait C2<T> { }

            struct X { }
            struct Y { }

            forall<A, B> { A: C1<B> if A: C2<B>, A = X }
            forall<A, B> { A: C2<B> if B: C1<A> }
        }

        goal {
            exists<T, U> { T: C1<U> }
        } yields {
            expect![["Unique; substitution [?0 := X, ?1 := X]"]]
        }
    }
}

#[test]
fn coinductive_trivial_variant3() {
    test! {
        program {
            #[coinductive]
            trait C1<T> { }

            forall<A, B> { A: C1<B> if B: C1<A> }
        }

        goal {
            exists<T, U> { T: C1<U> }
        } yields {
            expect![["Unique; for<?U0,?U0> { substitution [?0 := ^0.0, ?1 := ^0.1] }"]]
        }
    }
}

/// Test a tricky case for coinductive handling:
///
/// While proving C1, we try to prove C2, which recursively requires
/// proving C1.  If you are naive, you will assume that C2 therefore
/// holds -- but this is wrong, because C1 later fails when proving
/// C3.
#[test]
fn coinductive_unsound1() {
    test! {
        program {
            trait C1orC2 { }

            #[coinductive]
            trait C1 { }

            #[coinductive]
            trait C2 { }

            #[coinductive]
            trait C3 { }

            forall<T> {
                T: C1 if T: C2, T: C3
            }

            forall<T> {
                T: C2 if T: C1
            }

            forall<T> {
                T: C1orC2 if T: C1
            }

            forall<T> {
                T: C1orC2 if T: C2
            }
        }

        goal {
            forall<X> { X: C1orC2 }
        } yields {
            expect![["No possible solution"]]
        }
    }
}

/// The only difference between this test and `coinductive_unsound1`
/// is the order of the final `forall` clauses.
#[test]
fn coinductive_unsound2() {
    test! {
        program {
            trait C1orC2 { }

            #[coinductive]
            trait C1 { }

            #[coinductive]
            trait C2 { }

            #[coinductive]
            trait C3 { }

            forall<T> {
                T: C1 if T: C2, T: C3
            }

            forall<T> {
                T: C2 if T: C1
            }

            forall<T> {
                T: C1orC2 if T: C2
            }

            forall<T> {
                T: C1orC2 if T: C1
            }
        }

        goal {
            forall<X> { X: C1orC2 }
        } yields {
            expect![["No possible solution"]]
        }
    }
}

/// Tests whether a nested coinductive cycle
/// that is also unsound is handled correctly.
#[test]
fn coinductive_unsound_nested() {
    test! {
        program {
            trait C1orC2 { }

            #[coinductive]
            trait C1 { }

            #[coinductive]
            trait C2 { }

            #[coinductive]
            trait C3 { }

            #[coinductive]
            trait C4 { }

            forall<T> {
                T: C4 if T:C2, T: C3
            }

            forall<T> {
                T: C1 if T: C2, T: C3
            }

            forall<T> {
                T: C2 if T: C1, T: C4
            }

            forall<T> {
                T: C1orC2 if T: C1
            }

            forall<T> {
                T: C1orC2 if T: C2
            }
        }

        goal {
            forall<X> { X: C1orC2 }
        } yields {
            expect![["No possible solution"]]
        }
    }
}

/// Test with two nested coinductive cycles where the inner fails
/// whereas the outer holds. No false positives should be kept from
/// the inner cycle.
#[test]
fn coinductive_unsound_nested2() {
    test! {
        program {
            trait C1andC2 { }

            #[coinductive]
            trait C1 { }

            #[coinductive]
            trait C2 { }

            #[coinductive]
            trait C3 { }

            #[coinductive]
            trait C4 { }

            #[coinductive]
            trait C5 { }

            #[coinductive]
            trait C6 { }

            #[coinductive]
            trait C7 { }

            forall<T> {
                T: C2 if T: C5
            }

            forall<T> {
                T: C6 if T: C2, T: C7
            }

            forall<T> {
                T: C5 if T:C6
            }

            forall<T> {
                T: C4 if T: C1
            }

            forall<T> {
                T: C3 if T: C5
            }

            forall<T> {
                T: C3 if T: C4
            }

            forall<T> {
                T: C1 if T: C3
            }

            forall<T> {
                T: C1andC2 if T: C1, T: C2
            }
        }

        goal {
            forall<X> { X: C1andC2 }
        } yields {
            expect![["No possible solution"]]
        }
    }
}

/// Another test with two nested coinductive cycles.
/// Here the inner cycle is also dependent on the outer one.
#[test]
fn coinductive_unsound_inter_cycle_dependency() {
    test! {
        program {
            trait C1andC2 { }

            #[coinductive]
            trait C1 { }

            #[coinductive]
            trait C2 { }

            #[coinductive]
            trait C3 { }

            #[coinductive]
            trait C4 { }

            #[coinductive]
            trait C5 { }

            #[coinductive]
            trait C6 { }

            #[coinductive]
            trait C7 { }

            forall<T> {
                T: C2 if T: C5, T: C1
            }

            forall<T> {
                T: C6 if T: C2, T: C7
            }

            forall<T> {
                T: C5 if T:C6
            }

            forall<T> {
                T: C4 if T: C1
            }

            forall<T> {
                T: C3 if T: C5
            }

            forall<T> {
                T: C3 if T: C4
            }

            forall<T> {
                T: C1 if T: C3
            }

            forall<T> {
                T: C1andC2 if T: C1, T: C2
            }
        }

        goal {
            forall<X> { X: C1andC2 }
        } yields {
            expect![["No possible solution"]]
        }
    }
}

#[test]
fn coinductive_multicycle1() {
    test! {
        program {
            trait Any { }

            #[coinductive]
            trait C1 { }

            #[coinductive]
            trait C2 { }

            #[coinductive]
            trait C3 { }

            forall<T> {
                T: C1 if T: C2
            }

            forall<T> {
                T: C2 if T: C3
            }

            forall<T> {
                T: C3 if T: C1
            }

            forall<T> {
                T: Any if T: C3
            }

            forall<T> {
                T: Any if T: C2
            }

            forall<T> {
                T: Any if T: C1
            }
        }

        goal {
            forall<X> { X: Any }
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn coinductive_multicycle2() {
    test! {
        program {
            trait Any { }

            #[coinductive]
            trait C1 { }

            #[coinductive]
            trait C2 { }

            #[coinductive]
            trait C3 { }

            forall<T> {
                T: C1 if T: C2
            }

            forall<T> {
                T: C2 if T: C3
            }

            forall<T> {
                T: C3 if T: C1, T: C2
            }

            forall<T> {
                T: Any if T: C1
            }
        }

        goal {
            forall<X> { X: Any }
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn coinductive_multicycle3() {
    test! {
        program {
            trait Any { }

            #[coinductive]
            trait C1 { }

            #[coinductive]
            trait C2 { }

            #[coinductive]
            trait C3 { }

            trait C4 { }

            forall<T> {
                T: C1 if T: C2
            }

            forall<T> {
                T: C2 if T: C3, T: C4
            }

            forall<T> {
                T: C3 if T: C1
            }

            forall<T> {
                T: Any if T: C3
            }

            forall<T> {
                T: Any if T: C2
            }

            forall<T> {
                T: Any if T: C1
            }
        }

        goal {
            forall<X> { X: Any }
        } yields {
            expect![["No possible solution"]]
        }
    }
}

#[test]
fn coinductive_multicycle4() {
    test! {
        program {
            trait Any { }

            #[coinductive]
            trait C1 { }

            #[coinductive]
            trait C2 { }

            #[coinductive]
            trait C3 { }

            trait C4 { }

            forall<T> {
                T: C1 if T: C2
            }

            forall<T> {
                T: C2 if T: C3
            }

            forall<T> {
                T: C3 if T: C1, T: C4
            }

            forall<T> {
                T: Any if T: C3
            }

            forall<T> {
                T: Any if T: C2
            }

            forall<T> {
                T: Any if T: C1
            }
        }

        goal {
            forall<X> { X: Any }
        } yields {
            expect![["No possible solution"]]
        }
    }
}
