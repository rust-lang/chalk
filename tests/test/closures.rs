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
            "Unique"
        }
        goal {
            WellFormed(bar)
        } yields {
            "Unique"
        }
        goal {
            WellFormed(baz)
        } yields {
            "Unique"
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
            "Unique"
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
            "Unique"
        }
        goal {
            bar: Copy
        } yields {
            "Unique"
        }
        goal {
            baz: Copy
        } yields {
            "Unique"
        }

        // A closure with non-Copy upvars is not copy
        goal {
            forall<'a> {
                foobuzz<'a>: Copy
            }
        } yields {
            "No possible solution"
        }
        // A closure with only Copy upvars is copy
        goal {
            forall<'a> {
                foobar<'a>: Copy
            }
        } yields {
            "Unique"
        }
        goal {
            forall<T> { with_ty<T>: Copy }
        } yields {
            "No possible solution"
        }
        goal {
            forall<T> { if (T: Copy) { with_ty<T>: Copy } }
        } yields {
            "Unique"
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
            "Unique"
        }
        goal {
            bar: Clone
        } yields {
            "Unique"
        }
        goal {
            baz: Clone
        } yields {
            "Unique"
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
            "No possible solution"
        }
        goal {
            foo: FnMut<()>
        } yields {
            "No possible solution"
        }
        goal {
            foo: FnOnce<()>
        } yields {
            "Unique"
        }
        goal {
            Normalize(<foo as FnOnce<()>>::Output -> ())
        } yields {
            "Unique"
        }

        // A closure with kind `Fn` implements all `Fn` traits
        goal {
            bar: Fn<()>
        } yields {
            "Unique"
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
        goal {
            Normalize(<bar as FnOnce<()>>::Output -> ())
        } yields {
            "Unique"
        }

        // A closure with kind `FnMut` implements `FnMut` and `FnOnce`
        goal {
            baz: Fn<()>
        } yields {
            "No possible solution"
        }
        goal {
            baz: FnMut<()>
        } yields {
            "Unique"
        }
        goal {
            baz: FnOnce<()>
        } yields {
            "Unique"
        }
        goal {
            Normalize(<baz as FnOnce<()>>::Output -> ())
        } yields {
            "Unique"
        }
        // A closure also implements the `Fn` traits regardless of upvars
        goal {
            forall<'a> {
                foobar<'a>: FnOnce<(u8, f32)>
            }
        } yields {
            "Unique"
        }
        goal {
            forall<'a> {
                Normalize(<foobar<'a> as FnOnce<(u8, f32)>>::Output -> u32)
            }
        } yields {
            "Unique"
        }
        goal {
            forall<'a> {
                Normalize(<foobar<'a> as FnOnce<(u8, f32)>>::Output -> u32)
            }
        } yields {
            "Unique"
        }
        goal {
            forall<'a> {
                foobuzz<'a>: FnOnce<(u8, f32)>
            }
        } yields {
            "Unique"
        }
        goal {
            forall<'a> {
                Normalize(<foobuzz<'a> as FnOnce<(u8, f32)>>::Output -> u32)
            }
        } yields {
            "Unique"
        }
    }
}
