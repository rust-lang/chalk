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
            expect![["Unique"]]
        }

        goal {
            Foo: Clone
        } yields {
            expect![["Unique"]]
        }

        goal {
            Bar: Clone
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            Vec<Bar>: Clone
        } yields {
            expect![["No possible solution"]]
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
            expect![["Ambiguous; no inference guidance"]]
        }

        goal {
            exists<A> { A: Map<Bar> }
        } yields {
            expect![["Unique; substitution [?0 := Foo]"]]
        }

        goal {
            exists<A> { Foo: Map<A> }
        } yields {
            expect![["Unique; substitution [?0 := Bar]"]]
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
            expect![["No possible solution"]]
        }

        goal {
            forall<T> { not { T: Marker } }
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            not { forall<T> { T: Marker } }
        } yields {
            expect![["Unique"]]
        }

        // If we assume `T: Marker`, then obviously `T: Marker`.
        goal {
            forall<T> { if (T: Marker) { T: Marker } }
        } yields {
            expect![["Unique"]]
        }

        // We don't have to know anything about `T` to know that
        // `Vec<T>: Marker`.
        goal {
            forall<T> { Vec<T>: Marker }
        } yields {
            expect![["Unique"]]
        }

        // Here, we don't know that `T: Clone`, so we can't prove that
        // `Vec<T>: Clone`.
        goal {
            forall<T> { Vec<T>: Clone }
        } yields {
            expect![["No possible solution"]]
        }

        // Here, we do know that `T: Clone`, so we can.
        goal {
            forall<T> {
                if (T: Clone) {
                    Vec<T>: Clone
                }
            }
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn higher_ranked() {
    test! {
        program {
            struct BestType { }
            struct SomeType<T> { }
            trait Foo<T> { }
            impl<U> Foo<BestType> for SomeType<U> { }
        }

        goal {
            exists<V> {
                forall<U> {
                    SomeType<U>: Foo<V>
                }
            }
        } yields {
            expect![["Unique; substitution [?0 := BestType]"]]
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
            expect![["No possible solution"]]
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
            struct A { }
            struct B { }
            impl Identity for A { type Item = A; }
            impl Identity for B { type Item = B; }
        }

        goal {
            exists<T> {
                T: Identity<Item = A>
            }
        } yields {
            expect![["Unique; substitution [?0 := A]"]]
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
            struct A { }
            struct B { }
            struct Either<T, U> { }
            impl Combine for A { type Item<U> = Either<A, U>; }
            impl Combine for B { type Item<U> = Either<B, U>; }
        }

        goal {
            exists<T, U> {
                T: Combine<Item<U> = Either<A, B>>
            }
        } yields {
            expect![["Unique; substitution [?0 := A, ?1 := B]"]]
        }
    }
}

#[test]
fn generic_trait() {
    test! {
        program {
            struct Int { }
            struct Uint { }

            trait Eq<T> { }

            impl Eq<Int> for Int { }
            impl Eq<Uint> for Uint { }
        }

        goal {
            Int: Eq<Int>
        } yields {
            expect![["Unique"]]
        }

        goal {
            Uint: Eq<Uint>
        } yields {
            expect![["Unique"]]
        }

        goal {
            Int: Eq<Uint>
        } yields {
            expect![["No possible solution"]]
        }
    }
}

#[test]
// Test that we properly detect failure even if there are applicable impls at
// the top level, if we can't find anything to fill in those impls with
fn deep_failure() {
    test! {
        program {
            struct Foo<T> {}
            trait Bar {}
            trait Baz {}

            impl<T> Bar for Foo<T> where T: Baz {}
        }

        goal {
            exists<T> { T: Baz }
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            exists<T> { Foo<T>: Bar }
        } yields {
            expect![["No possible solution"]]
        }
    }
}

#[test]
// Test that we infer a unique solution even if it requires multiple levels of
// search to do so
fn deep_success() {
    test! {
        program {
            struct Foo<T> {}
            struct ImplsBaz {}
            trait Bar {}
            trait Baz {}

            impl Baz for ImplsBaz {}
            impl<T> Bar for Foo<T> where T: Baz {}
        }

        goal {
            exists<T> { Foo<T>: Bar }
        } yields {
            expect![["Unique; substitution [?0 := ImplsBaz]"]]
        }
    }
}

#[test]
fn definite_guidance() {
    test! {
        program {
            trait Display {}
            trait Debug {}
            struct Foo<T> {}
            struct Bar {}
            struct Baz {}

            impl Display for Bar {}
            impl Display for Baz {}

            impl<T> Debug for Foo<T> where T: Display {}
        }

        goal {
            exists<T> {
                T: Debug
            }
        } yields {
            expect![["Ambiguous; definite substitution for<?U0> { [?0 := Foo<^0.0>] }"]]
        }
    }
}

#[test]
fn suggested_subst() {
    test! {
        program {
            trait SomeTrait<A> {}
            struct Foo {}
            struct Bar {}
            struct Baz {}
            struct Qux {}
            impl SomeTrait<Baz> for Foo {}
            impl SomeTrait<Qux> for Bar {}
            impl SomeTrait<Baz> for Bar {}
        }

        goal {
            exists<T> {
                Foo: SomeTrait<T>
            }
        } yields {
            expect![["Unique; substitution [?0 := Baz]"]]
        }

        goal {
            exists<T> {
                if (Baz: SomeTrait<Qux>) {
                    Baz: SomeTrait<T>
                }
            }
        } yields {
            expect![["Unique; substitution [?0 := Qux]"]]
        }

        goal {
            exists<T> {
                if (Baz: SomeTrait<Qux>) {
                    Foo: SomeTrait<T>
                }
            }
        } yields {
            expect![["Unique; substitution [?0 := Baz]"]]
        }

        goal {
            exists<T> {
                if (Foo: SomeTrait<Baz>) {
                    Foo: SomeTrait<T>
                }
            }
        } yields {
            expect![["Unique; substitution [?0 := Baz]"]]
        }

        goal {
            exists<T> {
                if (Foo: SomeTrait<Qux>) {
                    Foo: SomeTrait<T>
                }
            }
        } yields {
            // FIXME: we need to rework the "favor environment" heuristic.
            // Should be: "Ambiguous; suggested substitution [?0 := bool]"
            expect![["Ambiguous; no inference guidance"]]
        }

        goal {
            exists<T> {
                if (Foo: SomeTrait<bool>) {
                    if (Foo: SomeTrait<Baz>) {
                        Foo: SomeTrait<T>
                    }
                }
            }
        } yields {
            expect![["Ambiguous; no inference guidance"]]
        }

        goal {
            exists<T> {
                Bar: SomeTrait<T>
            }
        } yields {
            expect![["Ambiguous; no inference guidance"]]
        }

        goal {
            exists<T> {
                if (Bar: SomeTrait<Qux>) {
                    Bar: SomeTrait<T>
                }
            }
        } yields {
            // FIXME: same as above, should be: expect![["Ambiguous; suggested substitution [?0 := bool]"]]
            expect![["Ambiguous; no inference guidance"]]
        }

        goal {
            exists<T> {
                if (Bar: SomeTrait<Qux>) {
                    if (Bar: SomeTrait<Baz>) {
                        Bar: SomeTrait<T>
                    }
                }
            }
        } yields {
            expect![["Ambiguous; no inference guidance"]]
        }
    }
}

#[test]
fn where_clause_trumps() {
    test! {
        program {
            struct Foo { }

            trait Marker { }
            impl Marker for Foo { }
        }

        goal {
            forall<T> {
                if (T: Marker) {
                    T: Marker
                }
            }
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn inapplicable_assumption_does_not_shadow() {
    test! {
        program {
            struct A { }
            struct B { }

            trait Foo<T> { }

            impl<T> Foo<A> for T { }
        }

        goal {
            forall<T> {
                exists<U> {
                    if (A: Foo<T>) {
                        T: Foo<U>
                    }
                }
            }
        } yields {
            expect![["Unique; substitution [?0 := A]"]]
        }
    }
}

#[test]
fn partial_overlap_2() {
    test! {
        program {
            trait Marker<T> {}
            trait Foo {}
            trait Bar {}

            struct TypeA {}
            struct TypeB {}

            impl<T> Marker<TypeA> for T where T: Foo {}
            impl<T> Marker<TypeB> for T where T: Bar {}
        }

        goal {
            forall<T> {
                if (T: Foo; T: Bar) {
                    exists<A> { T: Marker<A> }
                }
            }
        } yields {
            expect![["Ambiguous; no inference guidance"]]
        }

        goal {
            forall<T> {
                if (T: Foo; T: Bar) {
                    T: Marker<TypeB>
                }
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<T> {
                if (T: Foo; T: Bar) {
                    T: Marker<TypeA>
                }
            }
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn partial_overlap_3() {
    test! {
        program {
            #[marker] trait Marker {}
            trait Foo {}
            trait Bar {}

            impl<T> Marker for T where T: Foo {}
            impl<T> Marker for T where T: Bar {}

            struct Struct {}
            impl Foo for Struct {}
            impl Bar for Struct {}
        }

        goal {
            forall<T> {
                if (T: Foo; T: Bar) { T: Marker }
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            Struct: Marker
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn clauses_in_if_goals() {
    test! {
        program {
            trait Foo { }
            struct Vec<T> { }
            struct A { }
        }

        goal {
            if (forall<T> { T: Foo }) {
                forall<T> { T: Foo }
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<T> {
                if (Vec<T>: Foo :- T: Foo) {
                    if (T: Foo) {
                        Vec<T>: Foo
                    }
                }
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            if (forall<T> { Vec<T>: Foo :- T: Foo }) {
                if (A: Foo) {
                    Vec<A>: Foo
                }
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            if (forall<T> { Vec<T>: Foo :- T: Foo }) {
                Vec<A>: Foo
            }
        } yields {
            expect![["No possible solution"]]
        }
    }
}

#[test]
fn unify_types_in_ambiguous_impl() {
    test! {
        program {
            #[non_enumerable]
            trait Constraint {}
            trait Trait<T> {}
            struct A<T> {}
            impl<T> Trait<T> for A<T> where T: Constraint {}
        }

        goal {
            exists<T,U> { A<T>: Trait<U> }
        } yields {
            expect![["Ambiguous; definite substitution for<?U0> { [?0 := ^0.0, ?1 := ^0.0] }"]]
        }
    }
}

#[test]
fn unify_types_in_impl() {
    test! {
        program {
            #[non_enumerable]
            trait Constraint {}
            trait Trait<T> {}
            struct A<T> {}
            impl<T> Trait<T> for A<T> {}
        }

        goal {
            exists<T,U> { A<T>: Trait<U> }
        } yields {
            expect![["Unique; for<?U0> { substitution [?0 := ^0.0, ?1 := ^0.0] }"]]
        }
    }
}
