use super::*;

#[test]
fn closure_is_well_formed() {
    test! {
        program {
            closure foo(self,) {}
            closure bar(&self,) {}
            closure baz(&mut self,) {}
        }

        goal {
            WellFormed(foo)
        } yields {
            expect![["Unique"]]
        }
        goal {
            WellFormed(bar)
        } yields {
            expect![["Unique"]]
        }
        goal {
            WellFormed(baz)
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn closure_is_sized() {
    test! {
        program {
            #[lang(sized)]
            trait Sized { }

            closure foo(self,) {}
        }

        goal {
            foo: Sized
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn closure_is_copy() {
    test! {
        program {
            #[lang(copy)]
            trait Copy { }
            impl<'a, T> Copy for &'a T {}
            impl Copy for u8 {}
            impl Copy for u16 {}
            impl Copy for u32 {}

            closure foo(self,) {}
            closure bar(&self,) {}
            closure baz(&mut self,) {}

            closure foobuzz<'a>(self, a: u8, b: f32) -> u32 {
                u8;
                &'a u16;
                &'a mut u32
            }
            closure foobar<'a>(self, a: u8, b: f32) -> u32 {
                u8;
                &'a u16
            }
            closure with_ty<T>(self,) { T }
        }

        // A closure with no upvars is also copy, regardless of kind
        goal {
            foo: Copy
        } yields {
            expect![["Unique"]]
        }
        goal {
            bar: Copy
        } yields {
            expect![["Unique"]]
        }
        goal {
            baz: Copy
        } yields {
            expect![["Unique"]]
        }

        // A closure with non-Copy upvars is not copy
        goal {
            forall<'a> {
                foobuzz<'a>: Copy
            }
        } yields {
            expect![["No possible solution"]]
        }
        // A closure with only Copy upvars is copy
        goal {
            forall<'a> {
                foobar<'a>: Copy
            }
        } yields {
            expect![["Unique"]]
        }
        goal {
            forall<T> { with_ty<T>: Copy }
        } yields {
            expect![["No possible solution"]]
        }
        goal {
            forall<T> { if (T: Copy) { with_ty<T>: Copy } }
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn closure_is_clone() {
    test! {
        program {
            #[lang(clone)]
            trait Clone { }

            closure foo(self,) {}
            closure bar(&self,) {}
            closure baz(&mut self,) {}
        }
        goal {
            foo: Clone
        } yields {
            expect![["Unique"]]
        }
        goal {
            bar: Clone
        } yields {
            expect![["Unique"]]
        }
        goal {
            baz: Clone
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn closure_implements_fn_traits() {
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

            struct ConcreteFuture<T> { }

            impl<T> Future for ConcreteFuture<T> {
                type Output = T;
            }

            closure foo(self,) {}
            closure bar(&self,) {}
            closure baz(&mut self,) {}

            closure foobuzz<'a>(self, a: u8, b: f32) -> u32 {
                u8;
                &'a u16;
                &'a mut u32
            }
            closure foobar<'a>(self, a: u8, b: f32) -> u32 {
                u8;
                &'a u16
            }

            closure foo_async(self,) -> ConcreteFuture<()> { }
            closure bar_async(&self,) -> ConcreteFuture<()> { }
            closure baz_async(&mut self,) -> ConcreteFuture<()> { }

            closure foobuzz_async<'a>(self, a: u8, b: f32) -> ConcreteFuture<u32> {
                u8;
                &'a u16;
                &'a mut u32
            }
            closure foobar_async<'a>(self, a: u8, b: f32) -> ConcreteFuture<u32> {
                u8;
                &'a u16
            }
        }

        // A closure with kind `FnOnce` only implements `FnOnce`
        goal {
            foo: Fn<()>
        } yields {
            expect![["No possible solution"]]
        }
        goal {
            foo: FnMut<()>
        } yields {
            expect![["No possible solution"]]
        }
        goal {
            foo: FnOnce<()>
        } yields {
            expect![["Unique"]]
        }
        goal {
            Normalize(<foo as FnOnce<()>>::Output -> ())
        } yields {
            expect![["Unique"]]
        }

        // A closure with kind `AsyncFnOnce` only implements `AsyncFnOnce`
        goal {
            foo_async: AsyncFn<()>
        } yields {
            expect![["No possible solution"]]
        }
        goal {
            foo_async: AsyncFnMut<()>
        } yields {
            expect![["No possible solution"]]
        }
        goal {
            foo_async: AsyncFnOnce<()>
        } yields {
            expect![["Unique"]]
        }
        goal {
            Normalize(<foo_async as AsyncFnOnce<()>>::Output -> ())
        } yields {
            expect![["Unique"]]
        }

        // A closure with kind `Fn` implements all `Fn` traits
        goal {
            bar: Fn<()>
        } yields {
            expect![["Unique"]]
        }
        goal {
            bar: FnMut<()>
        } yields {
            expect![["Unique"]]
        }
        goal {
            bar: FnOnce<()>
        } yields {
            expect![["Unique"]]
        }
        goal {
            Normalize(<bar as FnOnce<()>>::Output -> ())
        } yields {
            expect![["Unique"]]
        }

        // A closure with kind `AsyncFn` implements all `AsyncFn` traits
        goal {
            bar_async: AsyncFn<()>
        } yields {
            expect![["Unique"]]
        }
        goal {
            bar_async: AsyncFnMut<()>
        } yields {
            expect![["Unique"]]
        }
        goal {
            bar_async: AsyncFnOnce<()>
        } yields {
            expect![["Unique"]]
        }
        goal {
            Normalize(<bar_async as AsyncFnOnce<()>>::Output -> ())
        } yields {
            expect![["Unique"]]
        }

        // A closure with kind `FnMut` implements `FnMut` and `FnOnce`
        goal {
            baz: Fn<()>
        } yields {
            expect![["No possible solution"]]
        }
        goal {
            baz: FnMut<()>
        } yields {
            expect![["Unique"]]
        }
        goal {
            baz: FnOnce<()>
        } yields {
            expect![["Unique"]]
        }
        goal {
            Normalize(<baz as FnOnce<()>>::Output -> ())
        } yields {
            expect![["Unique"]]
        }

        // A closure with kind `AsyncFnMut` implements `AsyncFnMut` and `AsyncFnOnce`
        goal {
            baz_async: AsyncFn<()>
        } yields {
            expect![["No possible solution"]]
        }
        goal {
            baz_async: AsyncFnMut<()>
        } yields {
            expect![["Unique"]]
        }
        goal {
            baz_async: AsyncFnOnce<()>
        } yields {
            expect![["Unique"]]
        }
        goal {
            Normalize(<baz_async as AsyncFnOnce<()>>::Output -> ())
        } yields {
            expect![["Unique"]]
        }

        // A closure also implements the `Fn/AsyncFn` traits regardless of upvars
        goal {
            forall<'a> {
                foobar<'a>: FnOnce<(u8, f32)>
            }
        } yields {
            expect![["Unique"]]
        }
        goal {
            forall<'a> {
                Normalize(<foobar<'a> as FnOnce<(u8, f32)>>::Output -> u32)
            }
        } yields {
            expect![["Unique"]]
        }
        goal {
            forall<'a> {
                Normalize(<foobar<'a> as FnOnce<(u8, f32)>>::Output -> u32)
            }
        } yields {
            expect![["Unique"]]
        }
        goal {
            forall<'a> {
                foobuzz<'a>: FnOnce<(u8, f32)>
            }
        } yields {
            expect![["Unique"]]
        }
        goal {
            forall<'a> {
                Normalize(<foobuzz<'a> as FnOnce<(u8, f32)>>::Output -> u32)
            }
        } yields {
            expect![["Unique"]]
        }
        goal {
            forall<'a> {
                foobar_async<'a>: AsyncFnOnce<(u8, f32)>
            }
        } yields {
            expect![["Unique"]]
        }
        goal {
            forall<'a> {
                Normalize(<foobar_async<'a> as AsyncFnOnce<(u8, f32)>>::Output -> u32)
            }
        } yields {
            expect![["Unique"]]
        }
        goal {
            forall<'a> {
                Normalize(<foobar_async<'a> as AsyncFnOnce<(u8, f32)>>::Output -> u32)
            }
        } yields {
            expect![["Unique"]]
        }
        goal {
            forall<'a> {
                foobuzz_async<'a>: AsyncFnOnce<(u8, f32)>
            }
        } yields {
            expect![["Unique"]]
        }
        goal {
            forall<'a> {
                Normalize(<foobuzz_async<'a> as AsyncFnOnce<(u8, f32)>>::Output -> u32)
            }
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn closures_propagate_auto_traits() {
    test! {
        program {
            #[auto]
            trait Send { }

            closure foo(self,) {}

            closure with_ty<T>(self,) { T }
        }

        goal {
            foo: Send
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<T> { with_ty<T>: Send }
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            forall<T> { if (T: Send) { with_ty<T>: Send } }
        } yields {
            expect![["Unique"]]
        }
    }
}
