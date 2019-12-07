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
            "No possible solution"
        }

        goal {
            Bar: Foo
        } yields {
            "No possible solution"
        }
    }
}

#[test]
fn coinductive_unification() {
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
            r"No possible solution"
        }
    }

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
            r"No possible solution"
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
            r"No possible solution"
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
            r"Unique; substitution [?0 := X, ?1 := X], lifetime constraints []"
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
            r"Unique; for<?U0> { substitution [?0 := X, ?1 := ^0], lifetime constraints [] }"
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
            r"Unique; for<?U0,?U0> { substitution [?0 := ^0, ?1 := ^1], lifetime constraints [] }"
        }
    }
}
