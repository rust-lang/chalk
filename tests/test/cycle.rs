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
        } yields[SolverChoice::recursive_default()] {
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
    }
}

// Regression test for chalk#571
#[test]
fn cycle_with_ambiguity() {
    test! {
        program {
            #[lang(sized)]
            trait Sized { }
            trait From<T> {}
            trait ToOwned {
                type Owned;
            }

            impl<T> ToOwned for [T] where T: Sized {
                type Owned = Vec<T>;
            }

            struct Rc<T> { }

            struct Vec<T> {}
            struct Cow<T> {}

            impl<T> From<Vec<T>> for Rc<[T]> {}
            impl<B> From<Cow<B>> for Rc<B>
            where
                B: ToOwned,
                Rc<B>: From<<B as ToOwned>::Owned>
            {
            }
        }

        goal {
            exists<S, T> {
                Rc<S>: From<T>
            }
        } yields[SolverChoice::slg_default()] {
            "Ambiguous; no inference guidance"
        }
    }
}
