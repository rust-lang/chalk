use crate::test::*;

#[test]
fn function_implement_fn_traits() {
    test! {
        program {
            #[lang(fn_once)]
            trait FnOnce<Args> {
                type Output;
            }

            #[lang(fn_mut)]
            trait FnMut<Args> where Self: FnOnce<Args> { }

            #[lang(fn)]
            trait Fn<Args> where Self: FnMut<Args> { }

            struct Ty { }

            trait Clone { }
            opaque type MyOpaque: Clone = Ty;

        }

        // Simple test: make sure a fully monomorphic type implements FnOnce
        goal {
            fn(u8): FnOnce<(u8,)>
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        // Same as above, but for FnMut
        goal {
            fn(u8): FnMut<(u8,)>
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        // Same as above, but for Fn
        goal {
            fn(u8): Fn<(u8,)>
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        // Function pointres implicity return `()` when no return
        // type is specified - make sure that normalization understands
        // this
        goal {
            Normalize(<fn(u8) as FnOnce<(u8,)>>::Output -> ())
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        // Tests normalizing when an explicit return type is used
        goal {
            Normalize(<fn(u8) -> bool as FnOnce<(u8,)>>::Output -> bool)
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        // Tests that we fail to normalize when there's a mismatch with
        // fully monomorphic types.
        goal {
            Normalize(<fn(u8) -> bool as FnOnce<(u8,)>>::Output -> u8)
        } yields {
            "No possible solution"
        }

        // Ensures that we don't find a solution when doing so would
        // require us to conclude that two different universally quantified
        // types (T and V) are equal.
        goal {
            forall<T, V> {
                Normalize(<fn(u8, V) -> T as FnOnce<(u8, V)>>::Output -> V)
            }
        } yields {
            "No possible solution"
        }

        // Tests that we can normalize a generic function pointer type
        goal {
            forall<T, V> {
                exists<U> {
                    Normalize(<fn(u8, V) -> T as FnOnce<(u8, V)>>::Output -> U)
                }
            }
        } yields {
            "Unique; substitution [?0 := !1_0], lifetime constraints []"
        }

        // Tests that we properly tuple function arguments when constrcting
        // the `FnOnce` impl
        goal {
            fn(u8, u32): FnOnce<(u8,u32)>
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        // Tests that we don't find a solution when fully monomorphic
        // types are mismatched
        goal {
            fn(i32): FnOnce<(bool,)>
        } yields {
            "No possible solution"
        }

        // Tests function pointer types that use the function's binder
        // Universally quantified lifetimes that differ only in their
        // name ('a vs 'b) should be considered equivalent here
        goal {
            forall<'a> {
                for<'b> fn(&'b u8): FnOnce<(&'a u8,)>
            }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        // Tests that a 'stricter' function (requires lifetimes to be the same)
        // can implement `FnOnce` for a 'less strict' signature (dose not require
        // lifetimes to be the same), provided that the lifetimes are *actually*
        // the same.
        goal {
            forall<'a, 'b> {
                for<'c> fn(&'c u8, &'c i32): FnOnce<(&'a u8, &'b i32)>
            }
        } yields {
            "Unique; substitution [], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!1_1 }, InEnvironment { environment: Env([]), goal: '!1_1: '!1_0 }]"
        }

        // Tests the opposite case as the previous test: a 'less strict' function
        // (does not require lifetimes to be the same) can implement `FnOnce` for
        // a 'stricter' signature (requires lifetimes to be the same) without
        // any additional requirements
        goal {
            forall<'a> {
                for<'b, 'c> fn(&'b u8, &'c i32): FnOnce<(&'a u8, &'a i32)>
            }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        // Similiar to the above test, but for types instead of lifetimes:
        // a 'stricter' function (requires types to be the same) can never
        // implement `FnOnce` for a 'less strict' signature (does not require
        // types to be the same)
        goal {
            forall<T, U> {
                fn(T, T): FnOnce<(T, U)>
            }
        } yields {
            "No possible solution"
        }

        // Tests the opposite case as a previous test: a 'less strict'
        // function can never implement 'FnOnce' for a 'more strict' signature
        // (does not require types to bthe same)
        goal {
            forall<T, U> {
                fn(T, U): FnOnce<(T, T)>
            }
        } yields {
            "No possible solution"
        }

        // Tests that we flounder for inference variables
        goal {
            exists<T> {
                T: FnOnce<()>
            }
        } yields_first[SolverChoice::slg(3, None)] {
            "Floundered"
        }

        // Tests that we flounder for alias type (opaque)
        goal {
            MyOpaque: FnOnce<()>
        } yields_first[SolverChoice::slg(3, None)] {
            "Floundered"
        }
    }
}