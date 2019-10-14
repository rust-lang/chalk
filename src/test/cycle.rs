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

            struct i32 { }
            struct Vec<T> { }

            impl<T> A for T where T: B { }
            impl A for i32 { }

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
            struct i32 { }
            impl<T> Foo for S<T> where T: Foo { }
            impl Foo for i32 { }
        }

        // infinite family of solutions: {i32, S<i32>, S<S<i32>>, ... }
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
            struct i32 { }
            impl<T> Foo for S<T> where T: Foo, T: Bar { }
            impl Foo for i32 { }
        }

        goal {
            exists<T> {
                T: Foo
            }
        } yields {
            "Unique; substitution [?0 := i32]"
        }
    }
}
