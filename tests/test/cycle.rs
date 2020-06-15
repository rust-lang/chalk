//! Tests related to cycles amongst impls, which we try to handle with
//! grace.

use super::*;

#[test]
fn inner_cycle() {
    // Interesting test that shows why recursive solver needs to run
    // to an inner fixed point during iteration. Here, the first
    // round, we get that `?T: A` has a unique sol'n `?T = i32`.  On
    // the second round, we ought to get ambiguous: but if we don't
    // run the `?T: B` to a fixed point, it will terminate with `?T =
    // i32`, leading to an (incorrect) unique solution.
    test! {
        program {
            #[marker]
            trait A { }
            #[marker]
            trait B { }

            struct Foo { }
            struct Vec<T> { }

            impl<T> A for T where T: B { }
            impl A for Foo { }

            impl<T> B for T where T: A { }
            impl<T> B for Vec<T> where T: B { }
        }

        goal {
            exists<T> { T: A }
        } yields {
            "Ambiguous"
        }
    }
}

#[test]
fn cycle_no_solution() {
    test! {
        program {
            trait Foo { }
            struct S<T> { }
            impl<T> Foo for S<T> where T: Foo { }
        }

        // only solution: infinite type S<S<S<...
        goal {
            exists<T> {
                T: Foo
            }
        } yields {
            "No possible solution"
        }
    }
}

#[test]
fn cycle_many_solutions() {
    test! {
        program {
            trait Foo { }
            struct S<T> { }
            struct Zero { }
            impl<T> Foo for S<T> where T: Foo { }
            impl Foo for Zero { }
        }

        // infinite family of solutions: {Zero, S<Zero>, S<S<Zero>>, ... }
        goal {
            exists<T> {
                T: Foo
            }
        } yields {
            "Ambiguous; no inference guidance"
        }
    }
}

#[test]
fn cycle_unique_solution() {
    test! {
        program {
            trait Foo { }
            trait Bar { }
            struct S<T> { }
            struct Zero { }
            impl<T> Foo for S<T> where T: Foo, T: Bar { }
            impl Foo for Zero { }
        }

        goal {
            exists<T> {
                T: Foo
            }
        } yields {
            "Unique; substitution [?0 := Zero]"
        }
    }
}

#[test]
fn multiple_ambiguous_cycles() {
    test! {
        program {
            trait WF { }
            trait Sized { }

            struct Vec<T> { }
            struct Int { }

            impl Sized for Int { }
            impl WF for Int { }

            impl<T> WF for Vec<T> where T: Sized { }
            impl<T> Sized for Vec<T> where T: WF, T: Sized { }
        }

        //          ?T: WF
        //             |
        //             |
        //             |
        // Int: WF. <-----> (Vec<?T>: WF) :- (?T: Sized)
        //                              |
        //                              |
        //                              |
        //              Int: Sized. <-------> (Vec<?T>: Sized) :- (?T: Sized), (?T: WF)
        //                                                            |            |
        //                                                            |            |
        //                                                            |            |
        //                                                          cycle        cycle
        //
        // Depending on the evaluation order of the above tree (which cycle we come upon first),
        // we may fail to reach a fixed point if we loop continuously because `Ambig` does not perform
        // any unification. We must stop looping as soon as we encounter `Ambig`. In fact without
        // this strategy, the above program will not even be loaded because of the overlap check which
        // will loop forever.
        goal {
            exists<T> {
                T: WF
            }
        } yields {
            "Ambig"
        }
    }
}

#[test]
fn overflow() {
    test! {
        program {
            trait Q { }
            struct Z { }
            struct G<X> { }
            struct S<X> { }

            impl Q for Z { }
            impl<X> Q for G<X> where X: Q { }
            impl<X> Q for S<X> where X: Q, S<G<X>>: Q { }
        }

        // Will try to prove S<G<Z>>: Q then S<G<G<Z>>>: Q etc ad infinitum
        goal {
            S<Z>: Q
        } yields[SolverChoice::slg(10, None)] {
            "Ambiguous; no inference guidance"
        } yields[SolverChoice::recursive()] {
            "Ambiguous; no inference guidance"
        }
    }
}

#[test]
fn overflow_universe() {
    test! {
        program {
            struct Foo { }

            trait Bar { }

            // When asked to solve X: Bar, we will produce a
            // requirement to solve !1_0: Bar. And then when asked to
            // solve that, we'll produce a requirement to solve !1_1:
            // Bar.  And so forth.
            forall<X> { X: Bar if forall<Y> { Y: Bar } }
        }

        goal {
            Foo: Bar
        } yields {
            // The internal universe canonicalization in the on-demand/recursive
            // solver means that when we are asked to solve (e.g.)
            // `!1_1: Bar`, we rewrite that to `!1_0: Bar`, identifying a
            // cycle.
            "No possible solution"
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
        }

        goal {
            exists<T> { T: A }
        } yields[SolverChoice::recursive()] {
            "No possible solution"
        }
    }
}

#[test]
fn negative_dependency() {
    test! {
        program {
            trait A { }
            trait B { }
            trait C { }

            forall<T> { T: A if T: B, T: C }
            forall<T> { T: B if not { T: A } }
            forall<T> { T: C }
        }

        goal {
            u32: A
        } yields[SolverChoice::recursive()] {
            "Ambiguous"
        }
    }
}

/// This example comes from the recursive solver documentation.
#[test]
fn negative_cycle_unprovable_subgoal() {
    test! {
        program {
            trait G1 { }
            trait G2 { }
            trait G3 { }

            struct A { }

            forall<> { A: G1 if not { A: G2 }, A: G3  }
            forall<> { A: G2 if not { A: G1 } }
        }

        goal {
            A: G1
        } yields[SolverChoice::recursive()] {
            "No possible solution"
        }

        goal {
            A: G2
        } yields[SolverChoice::recursive()] {
            "Unique"
        }
    }
}

/// This is adapted from Example 2.3 of EWFS. It shows a negative cycle where we
/// are able to successfully compute a result, but to do so we have to wait
/// until the negative cycle is fully resolved. I've modified the setup
/// to make it more amenable to the recursive solver which does not enumerate
/// answers.
#[test]
fn negative_cycle_figure_2_3_forward() {
    test! {
        program {
            trait W { }
            trait P { }

            struct A { }
            struct B { }
            struct C { }

            forall<> { A: W if not { B: W }, B: P  }
            forall<> { B: W if not { C: W }, C: P  }
            forall<> { C: W if not { B: W }, B: P  }
            forall<> { B: P }
        }

        // The correct result here is non-obvious, but in short
        // `A: W` and `C: W` are true, and `B: W` is not.
        //
        // How do we know? Well, `B: W` is false because `C: P` is
        // false, and the rest follows from there.
        //
        // The solver gets in a bit of a trap because it encounters
        // a negative cycle between `B: W` if not `C: W`, and
        // `C: W` if not `B: W`. But if we wait long enough,
        // it gets resolved.

        goal {
            A: W
        } yields[SolverChoice::recursive()] {
            "Unique"
        }

        goal {
            B: W
        } yields[SolverChoice::recursive()] {
            "No possible solution"
        }

        // This is an important test when caching is involved: computing `A: W`
        // winds up computing, as an intermediate value, `C: W` and *in that
        // computation* the result is ambiguous, because it relies on a negative
        // cycle. But we don't cache that result, so now, when we recompute `C:
        // W`, it depends on a cached value of `A: W`, and we are able to get
        // the correct value. (If we did cache the result of `C: W` earlier, we
        // would be getting ambiguous here.)
        goal {
            C: W
        } yields[SolverChoice::recursive()] {
            "Unique"
        }
    }
}

/// Same as `negative_cycle_figure_2_3_forward` except
/// for the clause order. This is because we sometimes
/// don't even encounter the negative cycle if we do
/// things in the "wrong" order.
#[test]
fn negative_cycle_figure_2_3_reverse() {
    test! {
        program {
            trait W { }
            trait P { }

            struct A { }
            struct B { }
            struct C { }

            forall<> { A: W if B: P, not { B: W } }
            forall<> { B: W if C: P, not { C: W } }
            forall<> { C: W if B: P, not { B: W } }
            forall<> { B: P }
        }

        goal {
            A: W
        } yields[SolverChoice::recursive()] {
            "Unique"
        }

        goal {
            B: W
        } yields[SolverChoice::recursive()] {
            "No possible solution"
        }

        goal {
            C: W
        } yields[SolverChoice::recursive()] {
            "Unique"
        }
    }
}

#[test]
fn negative_self_cycle() {
    test! {
        program {
            trait W { }

            struct A { }

            forall<> { A: W if not { A: W } }
        }

        goal {
            A: W
        } yields[SolverChoice::recursive()] {
            "Ambiguous"
        }
    }
}
