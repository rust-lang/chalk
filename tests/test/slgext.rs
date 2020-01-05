//! Tests for slg ext

use super::*;

#[test]
fn basic() {
    test! {
        program {
            trait Sized { }

            struct i32 { }
            impl Sized for i32 { }
        }

        goal {
            forall<T> { if (T: Sized) { T: Sized } }
        } yields_all[SolverChoice::slg(10, None)] {
            "substitution [], lifetime constraints []"
        }
    }
}

#[test]
fn breadth_first() {
    test! {
        program {
            trait Sized { }

            struct i32 { }
            impl Sized for i32 { }

            struct Vec<T> { }
            impl<T> Sized for Vec<T> where T: Sized { }

            struct Slice<T> { }
            impl<T> Sized for Slice<T> where T: Sized { }
        }

        goal {
            exists<T> { T: Sized }
        } yields_first[SolverChoice::slg(10, None)] {
            "substitution [?0 := i32], lifetime constraints []",
            "substitution [?0 := Vec<i32>], lifetime constraints []",
            "substitution [?0 := Slice<i32>], lifetime constraints []",
            "substitution [?0 := Vec<Vec<i32>>], lifetime constraints []",
            "substitution [?0 := Slice<Vec<i32>>], lifetime constraints []"
        }
    }
}

#[test]
fn infinite_recursion() {
    test! {
        program {
            trait A { }
            trait B { }
            trait C { }
            trait D { }

            struct Vec<T> { }
            impl<T> A for Vec<T> where T: B { }
            impl<T> B for Vec<T> where T: C { }
            impl<T> C for Vec<T> where T: D { }
            impl<T> D for Vec<T> where T: A { }
        }

        goal {
            exists<T> { T: A }
        } yields_all[SolverChoice::slg(10, None)] {
            "substitution [], lifetime constraints []"
        }
    }
}

/// Make sure we don't get a stack overflow or other badness for this
/// test from scalexm.
#[test]
fn subgoal_abstraction() {
    test! {
        program {
            trait Foo { }
            struct Box<T> { }
            impl<T> Foo for T where Box<T>: Foo { }
        }

        goal {
            exists<T> { T: Foo }
        } yields_all[SolverChoice::slg(50, None)] {
            "substitution [], lifetime constraints []"
        }
    }
}

#[test]
fn flounder() {
    test! {
        program {
            trait A { }

            struct Vec<T> { }
            impl<T> A for Vec<T> { }
        }

        goal {
            exists<T> { not { T: A } }
        } yields_first[SolverChoice::slg(10, None)] {
            "Floundered"
        }
    }
}

// Test that, when solving `?T: Sized`, we only wind up pulling a few
// answers before we stop.
// This is similar to the `breadth_first` test, except the order of the
// FIXME: This is basically the same as `breadth_first`. Is it testing something different?
#[test]
fn only_draw_so_many() {
    test! {
        program {
            trait Sized { }

            struct Vec<T> { }
            impl<T> Sized for Vec<T> where T: Sized { }

            struct i32 { }
            impl Sized for i32 { }

            struct Slice<T> { }
            impl<T> Sized for Slice<T> where T: Sized { }
        }

        goal {
            exists<T> { T: Sized }
        } yields_first[SolverChoice::slg(10, None)] {
            "substitution [?0 := i32], lifetime constraints []",
            "substitution [?0 := Slice<i32>], lifetime constraints []",
            "substitution [?0 := Vec<i32>], lifetime constraints []"
        }

        goal {
            exists<T> { T: Sized }
        } yields[SolverChoice::slg(10, Some(2))] {
            "Ambiguous; no inference guidance"
        }
    }
}

#[test]
fn only_draw_so_many_blow_up() {
    test! {
        program {
            trait Sized { }
            trait Foo { }

            struct Vec<T> { }
            impl<T> Sized for Vec<T> where T: Sized { }
            impl<T> Foo for Vec<T> where T: Sized { }

            struct i32 { }
            impl Sized for i32 { }

            struct Slice<T> { }
            impl<T> Sized for Slice<T> where T: Sized { }
        }

        goal {
            exists<T> { T: Foo }
        } yields[SolverChoice::slg(10, Some(2))] {
            "Ambiguous; definite substitution for<?U0> { [?0 := Vec<^0>] }"
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
            struct u32 { }

            forall<> { u32: P if not { u32: Q } }
            forall<> { u32: Q if not { u32: P } }
        }

        goal {
            u32: P
        } yields_all[SolverChoice::slg(10, None)] {
            // Negative cycle -> panic
            ""
        }
    }
}

// FIXME: well-formed problems?
#[test]
#[ignore]
fn subgoal_cycle_uninhabited() {
    test! {
        program {
            trait Foo { }
            struct Box<T> { }
            struct Vec<T> { }
            struct u32 { }
            impl<T> Foo for Box<T> where Box<Vec<T>>: Foo { }
        }

        // There is no solution here with a finite proof, so we get
        // back: 0 answer(s) found.
        goal {
            exists<T> { T: Foo }
        } yields_all[SolverChoice::slg(2, None)] {
            "substitution [], lifetime constraints []"
        }

        // Unsurprisingly, applying negation succeeds then.
        goal {
            not { exists<T> { T: Foo } }
        } yields_all[SolverChoice::slg(2, None)] {
            "substitution [], lifetime constraints []"
        }

        // Equivalent to the previous.
        goal {
            forall<T> { not { T: Foo } }
        } yields_all[SolverChoice::slg(2, None)] {
            "substitution [], lifetime constraints []"
        }

        // However, if we come across a negative goal that exceeds our
        // size threshold, we have a problem.
        goal {
            exists<T> { T = Vec<u32>, not { Vec<Vec<T>>: Foo } }
        } yields_all[SolverChoice::slg(2, None)] {
            ""
        }

        // Same query with larger threshold works fine, though.
        goal {
            exists<T> { T = Vec<u32>, not { Vec<Vec<T>>: Foo } }
        } yields_all[SolverChoice::slg(4, None)] {
            "substitution [?0 := Vec<u32>], lifetime constraints []"
        }

        // Here, due to the hypothesis, there does indeed exist a suitable T, `U`.
        goal {
            forall<U> { if (U: Foo) { exists<T> { T: Foo } } }
        } yields_all[SolverChoice::slg(2, None)] {
            "substitution [?0 := !1_0], lifetime constraints []"
        }
    }
}

#[test]
fn subgoal_cycle_inhabited() {
    test! {
        program {
            trait Foo { }
            struct Box<T> { }
            struct Vec<T> { }
            struct u32 { }
            impl<T> Foo for Box<T> where Box<Vec<T>>: Foo { }
            impl Foo for u32 { }
        }

        goal {
            exists<T> { T: Foo }
        } yields_all[SolverChoice::slg(3, None)] {
            "substitution [?0 := u32], lifetime constraints []"
        }
    }
}

#[test]
fn basic_region_constraint_from_positive_impl() {
    test! {
        program {
            trait Foo { }
            struct Ref<'a, 'b, T> { }
            struct u32 { }
            impl<'x, T> Foo for Ref<'x, 'x, T> { }
        }

        goal {
            forall<'a, 'b, T> { Ref<'a, 'b, T>: Foo }
        } yields_all[SolverChoice::slg(3, None)] {
            "substitution [], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_1 == '!1_0 }]"
        }
    }
}

#[test]
#[allow(non_snake_case)]
fn example_2_1_EWFS() {
    test! {
        program {
            trait Edge<B> { }
            trait TransitiveClosure<B> { }
            struct a { }
            struct b { }
            struct c { }

            forall<> { a: Edge<b> }
            forall<> { b: Edge<c> }
            forall<> { b: Edge<a> }
            forall<X, Y> { X: TransitiveClosure<Y> if X: Edge<Y> }
            forall<X, Y, Z> { X: TransitiveClosure<Y> if X: Edge<Z>, Z: TransitiveClosure<Y> }
        }

        goal {
            exists<V> { a: TransitiveClosure<V> }
        } yields_all[SolverChoice::slg(3, None)] {
            "substitution [?0 := b], lifetime constraints []",
            "substitution [?0 := c], lifetime constraints []",
            "substitution [?0 := a], lifetime constraints []"
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
            "substitution [], lifetime constraints []"
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
            ""
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
            ""
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
            struct u32 { }

            forall<> { u32: P if not { u32: P } }
        }

        goal {
            u32: P
        } yields_all[SolverChoice::slg(3, None)] {
            // Negative cycle -> panic
            ""
        }
    }
}

/// Test (along with the other `cached_answers` tests) that the
/// ordering in which we we encounter clauses doesn't affect the final
/// set of answers we get. In particular, all of them should get 5
/// answers, but in Ye Olde Days Of Yore there were sometimes bugs
/// that came up when replaying tabled answers that led to fewer
/// answers being produced.
///
/// This test is also a test for ANSWER ABSTRACTION: the only reason
/// we get 5 answers is because of the max size of 2.
#[test]
fn cached_answers_1() {
    test! {
        program {
            trait Sour { }
            struct Lemon { }
            struct Vinegar { }
            struct HotSauce<T> { }

            // Use explicit program clauses here rather than traits
            // and impls to avoid hashmaps and other things that
            // sometimes alter the final order of the program clauses:
            forall<> { Lemon: Sour }
            forall<> { Vinegar: Sour }
            forall<T> { HotSauce<T>: Sour if T: Sour }
        }

        goal {
            exists<T> { T: Sour }
        } yields_all[SolverChoice::slg(2, None)] {
            "substitution [?0 := Lemon], lifetime constraints []",
            "substitution [?0 := Vinegar], lifetime constraints []",
            "substitution [?0 := HotSauce<Lemon>], lifetime constraints []",
            "substitution [?0 := HotSauce<Vinegar>], lifetime constraints []",
            "Ambiguous(for<?U0> { substitution [?0 := HotSauce<^0>], lifetime constraints [] })"
        }
    }
}

/// See `cached_answers_1`.
#[test]
fn cached_answers_2() {
    test! {
        program {
            trait Sour { }
            struct Lemon { }
            struct Vinegar { }
            struct HotSauce<T> { }

            forall<T> { HotSauce<T>: Sour if T: Sour }
            forall<> { Lemon: Sour }
            forall<> { Vinegar: Sour }
        }

        goal {
            exists<T> { T: Sour }
        } yields_all[SolverChoice::slg(2, None)] {
            "substitution [?0 := Lemon], lifetime constraints []",
            "substitution [?0 := Vinegar], lifetime constraints []",
            "substitution [?0 := HotSauce<Lemon>], lifetime constraints []",
            "substitution [?0 := HotSauce<Vinegar>], lifetime constraints []",
            "Ambiguous(for<?U0> { substitution [?0 := HotSauce<^0>], lifetime constraints [] })"
        }
    }
}

/// See `cached_answers_1`.
#[test]
fn cached_answers_3() {
    test! {
        program {
            trait Sour { }
            struct Lemon { }
            struct Vinegar { }
            struct HotSauce<T> { }

            forall<> { Lemon: Sour }
            forall<T> { HotSauce<T>: Sour if T: Sour }
            forall<> { Vinegar: Sour }
        }

        goal {
            exists<T> { T: Sour }
        } yields_all[SolverChoice::slg(2, None)] {
            "substitution [?0 := Lemon], lifetime constraints []",
            "substitution [?0 := HotSauce<Lemon>], lifetime constraints []",
            "substitution [?0 := Vinegar], lifetime constraints []",
            "Ambiguous(for<?U0> { substitution [?0 := HotSauce<^0>], lifetime constraints [] })",
            "substitution [?0 := HotSauce<Vinegar>], lifetime constraints []"
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
            struct u32 { }

            forall<> { u32: P if not { u32: Q } }
            forall<> { u32: Q if not { u32: Q } }
        }

        goal {
            u32: P
        } yields_all[SolverChoice::slg(3, None)] {
            // Negative cycle -> panic
            ""
        }
    }
}

#[test]
fn non_enumerable_traits_direct() {
    test! {
        program {
            struct Foo { }
            struct Bar { }

            #[non_enumerable]
            trait NonEnumerable { }
            impl NonEnumerable for Foo { }
            impl NonEnumerable for Bar { }

            trait Enumerable { }
            impl Enumerable for Foo { }
            impl Enumerable for Bar { }
        }

        goal {
            exists<A> { A: NonEnumerable }
        } yields_first[SolverChoice::slg(3, None)] {
            "Floundered"
        }

        goal {
            exists<A> { A: Enumerable }
        } yields_all[SolverChoice::slg(3, None)] {
            "substitution [?0 := Foo], lifetime constraints []",
            "substitution [?0 := Bar], lifetime constraints []"
        }

        goal {
            Foo: NonEnumerable
        } yields_all[SolverChoice::slg(3, None)] {
            "substitution [], lifetime constraints []"
        }
    }
}

#[test]
fn non_enumerable_traits_indirect() {
    test! {
        program {
            struct Foo { }
            struct Bar { }

            #[non_enumerable]
            trait NonEnumerable { }
            impl NonEnumerable for Foo { }
            impl NonEnumerable for Bar { }

            trait Debug { }
            impl<T> Debug for T where T: NonEnumerable { }
        }

        goal {
            exists<A> { A: Debug }
        } yields_first[SolverChoice::slg(3, None)] {
            "Floundered"
        }
    }
}

#[test]
fn non_enumerable_traits_double() {
    test! {
        program {
            struct Foo { }
            struct Bar { }

            #[non_enumerable]
            trait NonEnumerable1 { }
            impl NonEnumerable1 for Foo { }
            impl NonEnumerable1 for Bar { }

            #[non_enumerable]
            trait NonEnumerable2 { }
            impl NonEnumerable2 for Foo { }
            impl NonEnumerable2 for Bar { }

            trait Debug { }
            impl<T> Debug for T where T: NonEnumerable1, T: NonEnumerable2  { }
        }

        goal {
            exists<A> { A: Debug }
        } yields_first[SolverChoice::slg(3, None)] {
            "Floundered"
        }
    }
}

#[test]
fn non_enumerable_traits_reorder() {
    test! {
        program {
            struct Foo { }
            struct Bar { }

            #[non_enumerable]
            trait NonEnumerable { }
            impl NonEnumerable for Foo { }
            impl NonEnumerable for Bar { }

            trait Enumerable { }
            impl Enumerable for Foo { }

            // In this test, we first try to solve to solve `T:
            // NonEnumerable` but then we discover it's
            // non-enumerable, and so we push it off for later. Then
            // we try to solve the `T: Enumerable` trait.

            trait Debug1 { }
            impl<T> Debug1 for T where T: Enumerable, T: NonEnumerable { }

            trait Debug2 { }
            impl<T> Debug2 for T where T: NonEnumerable, T: Enumerable { }
        }

        goal {
            exists<A> { A: Debug1 }
        } yields_all[SolverChoice::slg(3, None)] {
            "substitution [?0 := Foo], lifetime constraints []"
        }


        goal {
            exists<A> { A: Debug2 }
        } yields_all[SolverChoice::slg(3, None)] {
            "substitution [?0 := Foo], lifetime constraints []"
        }
    }
}

#[test]
fn auto_traits_flounder() {
    test! {
        program {
            struct Foo { }
            struct Bar { }

            #[auto]
            trait Send { }
        }

        goal {
            exists<A> { A: Send }
        } yields_first[SolverChoice::slg(3, None)] {
            "Floundered"
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
            "substitution [?0 := Bar], lifetime constraints []"
        }


        goal {
            exists<A> { A: Debug2 }
        } yields_all[SolverChoice::slg(3, None)] {
            "substitution [?0 := Bar], lifetime constraints []"
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
        } yields_all[SolverChoice::slg(3, None)] {
            ""
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
        } yields_all[SolverChoice::slg(3, None)] {
            ""
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
        } yields_all[SolverChoice::slg(3, None)] {
            "substitution [], lifetime constraints []"
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
        } yields_all[SolverChoice::slg(3, None)] {
            "substitution [], lifetime constraints []"
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
        } yields_all[SolverChoice::slg(3, None)] {
            ""
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
        } yields_all[SolverChoice::slg(3, None)] {
            ""
        }
    }
}
