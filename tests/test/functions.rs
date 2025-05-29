use super::*;

#[test]
fn functions_are_sized() {
    test! {
        program {
            #[lang(sized)]
            trait Sized { }
        }

        goal {
            fn(()): Sized
        } yields {
            expect![["Unique"]]
        }

        goal {
            fn([u8]): Sized
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn functions_are_copy() {
    test! {
        program {
            #[lang(copy)]
            trait Copy { }
        }

        goal {
            fn(()): Copy
        } yields {
            expect![["Unique"]]
        }

        goal {
            fn([u8]): Copy
        } yields {
            expect![["Unique"]]
        }
    }
}

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

            #[lang(future)]
            trait Future {
                type Output;
            }

            #[lang(async_fn_once)]
            trait AsyncFnOnce<Args> {
                type CallOnceFuture: Future<Output = <Self as AsyncFnOnce<Args>>::Output>;
                #[lang(async_fn_once_output)]
                type Output;
            }

            #[lang(async_fn_mut)]
            trait AsyncFnMut<Args> where Self: AsyncFnOnce<Args> { }

            #[lang(async_fn)]
            trait AsyncFn<Args> where Self: AsyncFnMut<Args> { }

            struct Ty { }

            trait Clone { }

            impl Clone for Ty { }

            opaque type MyOpaque: Clone = Ty;

            struct ConcreteFuture<T> { }

            impl<T> Future for ConcreteFuture<T> {
                type Output = T;
            }
        }

        // Simple test: make sure a fully monomorphic type implements FnOnce
        goal {
            fn(u8): FnOnce<(u8,)>
        } yields {
            expect![["Unique"]]
        }

        // Same as above, but for FnMut
        goal {
            fn(u8): FnMut<(u8,)>
        } yields {
            expect![["Unique"]]
        }

        // Same as above, but for Fn
        goal {
            fn(u8): Fn<(u8,)>
        } yields {
            expect![["Unique"]]
        }

        // Same as above, but for AsyncFnOnce
        goal {
            fn(u8) -> ConcreteFuture<()>: AsyncFnOnce<(u8,)>
        } yields {
            expect![["Unique"]]
        }

        // Same as above, but for AsyncFnMut
        goal {
            fn(u8) -> ConcreteFuture<()>: AsyncFnMut<(u8,)>
        } yields {
            expect![["Unique"]]
        }

        // Same as above, but for AsyncFn
        goal {
            fn(u8) -> ConcreteFuture<()>: AsyncFn<(u8,)>
        } yields {
            expect![["Unique"]]
        }

        // Make sure unsafe function pointers don't implement FnOnce
        goal {
            unsafe fn(u8): FnOnce<(u8,)>
        } yields {
            expect![["No possible solution"]]
        }
        // Same as above but for FnMut
        goal {
            unsafe fn(u8): FnMut<(u8,)>
        } yields {
            expect![["No possible solution"]]
        }
        // Same as above but for Fn
        goal {
            unsafe fn(u8): Fn<(u8,)>
        } yields {
            expect![["No possible solution"]]
        }
        // Same as above but for AsyncFnOnce
        goal {
            unsafe fn(u8) -> ConcreteFuture<()>: AsyncFnOnce<(u8,)>
        } yields {
            expect![["No possible solution"]]
        }
        // Same as above but for AsyncFnMut
        goal {
            unsafe fn(u8) -> ConcreteFuture<()>: AsyncFnMut<(u8,)>
        } yields {
            expect![["No possible solution"]]
        }
        // Same as above but for AsyncFn
        goal {
            unsafe fn(u8) -> ConcreteFuture<()>: AsyncFn<(u8,)>
        } yields {
            expect![["No possible solution"]]
        }

        // Function pointers implicit return `()` when no return
        // type is specified - make sure that normalization understands
        // this
        goal {
            Normalize(<fn(u8) as FnOnce<(u8,)>>::Output -> ())
        } yields {
            expect![["Unique"]]
        }

        // Tests normalizing when an explicit return type is used
        goal {
            Normalize(<fn(u8) -> bool as FnOnce<(u8,)>>::Output -> bool)
        } yields {
            expect![["Unique"]]
        }

        // Normalizing pointer which returns `Future<Output = T>` with `AsycFnOnce::Output`
        goal {
            Normalize(<fn(u8) -> ConcreteFuture<bool> as AsyncFnOnce<(u8,)>>::Output -> bool)
        } yields {
            expect![["Unique"]]
        }

        // Tests that we fail to normalize when there's a mismatch with
        // fully monomorphic types.
        goal {
            Normalize(<fn(u8) -> bool as FnOnce<(u8,)>>::Output -> u8)
        } yields {
            expect![["No possible solution"]]
        }
        goal {
            Normalize(<fn(u8) -> ConcreteFuture<bool> as AsyncFnOnce<(u8,)>>::Output -> u8)
        } yields {
            expect![["No possible solution"]]
        }

        // Ensures that we don't find a solution when doing so would
        // require us to conclude that two different universally quantified
        // types (T and V) are equal.
        goal {
            forall<T, V> {
                Normalize(<fn(u8, V) -> T as FnOnce<(u8, V)>>::Output -> V)
            }
        } yields {
            expect![["No possible solution"]]
        }
        goal {
            forall<T, V> {
                Normalize(<fn(u8, V) -> ConcreteFuture<T> as AsyncFnOnce<(u8, V)>>::Output -> V)
            }
        } yields {
            expect![["No possible solution"]]
        }

        // Tests that we can normalize a generic function pointer type
        goal {
            forall<T, V> {
                exists<U> {
                    Normalize(<fn(u8, V) -> T as FnOnce<(u8, V)>>::Output -> U)
                }
            }
        } yields {
            expect![["Unique; substitution [?0 := !1_0]"]]
        }
        goal {
            forall<T, V> {
                exists<U> {
                    Normalize(<fn(u8, V) -> ConcreteFuture<T> as AsyncFnOnce<(u8, V)>>::Output -> U)
                }
            }
        } yields {
            expect![["Unique; substitution [?0 := !1_0]"]]
        }

        // Tests that we properly tuple function arguments when constructing
        // the `FnOnce` impl
        goal {
            fn(u8, u32): FnOnce<(u8,u32)>
        } yields {
            expect![["Unique"]]
        }

        // Tests that we properly tuple function arguments when constructing
        // the `AsyncFnOnce` impl
        goal {
            fn(u8, u32) -> ConcreteFuture<()>: AsyncFnOnce<(u8,u32)>
        } yields {
            expect![["Unique"]]
        }

        // Tests that we don't find a solution when fully monomorphic
        // types are mismatched
        goal {
            fn(i32): FnOnce<(bool,)>
        } yields {
            expect![["No possible solution"]]
        }
        goal {
            fn(i32) -> ConcreteFuture<()>: AsyncFnOnce<(bool,)>
        } yields {
            expect![["No possible solution"]]
        }

        // Tests function pointer types that use the function's binder
        // Universally quantified lifetimes that differ only in their
        // name ('a vs 'b) should be considered equivalent here
        goal {
            forall<'a> {
                for<'b> fn(&'b u8): FnOnce<(&'a u8,)>
            }
        } yields {
            expect![["Unique"]]
        }
        goal {
            forall<'a> {
                for<'b> fn(&'b u8) -> ConcreteFuture<()>: AsyncFnOnce<(&'a u8,)>
            }
        } yields {
            expect![["Unique"]]
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
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!1_1 }, InEnvironment { environment: Env([]), goal: '!1_1: '!1_0 }]"]]
        }
        goal {
            forall<'a, 'b> {
                for<'c> fn(&'c u8, &'c i32) -> ConcreteFuture<()>: AsyncFnOnce<(&'a u8, &'b i32)>
            }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!1_1 }, InEnvironment { environment: Env([]), goal: '!1_1: '!1_0 }]"]]
        }

        // Tests the opposite case as the previous test: a 'less strict' function
        // (does not require lifetimes to be the same) can implement `FnOnce/AsyncFnOnce`
        // for a 'stricter' signature (requires lifetimes to be the same) without
        // any additional requirements
        goal {
            forall<'a> {
                for<'b, 'c> fn(&'b u8, &'c i32): FnOnce<(&'a u8, &'a i32)>
            }
        } yields {
            expect![["Unique"]]
        }
        goal {
            forall<'a> {
                for<'b, 'c> fn(&'b u8, &'c i32) -> ConcreteFuture<()>: AsyncFnOnce<(&'a u8, &'a i32)>
            }
        } yields {
            expect![["Unique"]]
        }

        // Similar to the above test, but for types instead of lifetimes:
        // a 'stricter' function (requires types to be the same) can never
        // implement `FnOnce/AsyncFnOnce` for a 'less strict' signature (does
        // not require types to be the same)
        goal {
            forall<T, U> {
                fn(T, T): FnOnce<(T, U)>
            }
        } yields {
            expect![["No possible solution"]]
        }
        goal {
            forall<T, U> {
                fn(T, T) -> ConcreteFuture<()>: AsyncFnOnce<(T, U)>
            }
        } yields {
            expect![["No possible solution"]]
        }

        // Tests the opposite case as a previous test: a 'less strict'
        // function can never implement 'FnOnce/AsyncFnOnce' for a 'more
        // strict' signature (does not require types to bthe same)
        goal {
            forall<T, U> {
                fn(T, U): FnOnce<(T, T)>
            }
        } yields {
            expect![["No possible solution"]]
        }
        goal {
            forall<T, U> {
                fn(T, U) -> ConcreteFuture<()>: AsyncFnOnce<(T, T)>
            }
        } yields {
            expect![["No possible solution"]]
        }

        // Tests that we flounder for inference variables
        goal {
            exists<T> {
                T: FnOnce<()>
            }
        } yields_first[SolverChoice::slg(3, None)] {
            expect![["Floundered"]]
        }
        goal {
            exists<T> {
                T: AsyncFnOnce<()>
            }
        } yields_first[SolverChoice::slg(3, None)] {
            expect![["Floundered"]]
        }

        // No solution for alias type
        goal {
            MyOpaque: FnOnce<()>
        } yields {
            expect![["No possible solution"]]
        }
        goal {
            MyOpaque: AsyncFnOnce<()>
        } yields {
            expect![["No possible solution"]]
        }
    }
}

#[test]
fn functions_implement_fn_ptr_trait() {
    test! {
        program {
            #[lang(fn_ptr_trait)]
            trait FnPtr {}

            closure closure_ty(self,) {}
            fn fn_def();
        }

        goal {
            fn(()) -> (): FnPtr
        } yields {
            expect![[r#"Unique"#]]
        }

        goal {
            unsafe fn(): FnPtr
        } yields {
            expect![[r#"Unique"#]]
        }

        goal {
            extern "C" fn(u32, ...): FnPtr
        } yields {
            expect![[r#"Unique"#]]
        }

        goal {
            for<'a> fn(&'a ()): FnPtr
        } yields {
            expect![[r#"Unique"#]]
        }

        goal {
            forall<T, U> {
                fn(T) -> U: FnPtr
            }
        } yields {
            expect![[r#"Unique"#]]
        }

        goal {
            i32: FnPtr
        } yields {
            expect![[r#"No possible solution"#]]
        }

        goal {
            closure_ty: FnPtr
        } yields {
            expect![[r#"No possible solution"#]]
        }

        goal {
            fn_def: FnPtr
        } yields {
            expect![[r#"No possible solution"#]]
        }

        goal {
            exists<T> {
                T: FnPtr
            }
        } yields {
            expect![[r#"Ambiguous; no inference guidance"#]]
        }
    }
}
