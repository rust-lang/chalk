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
        // A closure also implements the `Fn` traits regardless of upvars
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
    }
}

#[test]
fn can_override_closure_traits() {
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

            closure foo(&self,) {}
            closure bar(&self,) {}
            closure baz<T>(&mut self,) {}

            impl FnOnce<()> for foo {
                type Output = i32;
            }
            impl FnMut<()> for foo {}
            impl Fn<()> for foo {}

            impl<T> FnOnce<T> for bar {
                type Output = T;
            }
            impl<T> FnMut<T> for bar {}

            impl<T> FnOnce<T> for baz<T> {
                type Output = i32;
            }
        }
        // All 3 traits implemented
        goal {
            foo: Fn<()>
        } yields {
            "Unique"
        }
        goal {
            foo: FnMut<()>
        } yields {
            "Unique"
        }
        goal {
            foo: FnOnce<()>
        } yields {
            "Unique"
        }
        goal {
            Normalize(<foo as FnOnce<()>>::Output -> i32)
        } yields {
            "Unique"
        }
        // and the impl is not wider than expected
        goal {
            foo: FnOnce<i32>
        } yields {
            "No possible solution"
        }

        // Do not implement `Fn` on `&self` closure if there's an override
        goal {
            bar: Fn<()>
        } yields {
            "No possible solution"
        }
        goal {
            bar: FnMut<()>
        } yields {
            "Unique"
        }
        goal {
            bar: FnOnce<()>
        } yields {
            "Unique"
        }
        // also the generic impl still does something reasonable
        goal {
            exists<T> {
                Normalize(<bar as FnOnce<T>>::Output -> T)
            }
        } yields {
            "Unique; for<?U0> { substitution [?0 := ^0.0], lifetime constraints [] }"
        }
        goal {
            exists<T> {
                Normalize(<bar as FnOnce<T>>::Output -> ())
            }
        } yields {
            "Unique; substitution [?0 := 0], lifetime constraints []"
        }

        // Do not implement `Fn` or 'FnMut' on `&mut self` closure if there's an override
        goal {
            exists<T, U> {
                baz<T>: Fn<U>
            }
        } yields {
            "No possible solution"
        }
        goal {
            exists<T, U> {
                baz<T>: FnMut<U>
            }
        } yields {
            "No possible solution"
        }
        goal {
            baz<()>: FnOnce<()>
        } yields {
            "Unique"
        }
    }
}
