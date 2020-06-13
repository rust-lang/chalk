use super::*;

#[test]
fn closure_is_well_formed() {
    test! {
        program {
            closure foo(self);
            closure bar(refself);
            closure baz(mutself);
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

            closure foo(self);
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

            closure foo(self);
            closure bar(refself);
            closure baz(mutself);
        }

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
    }
}

#[test]
fn closure_is_clone() {
    test! {
        program {
            #[lang(clone)]
            trait Clone { }

            closure foo(self);
            closure bar(refself);
            closure baz(mutself);
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

            closure foo(self);
            closure bar(refself);
            closure baz(mutself);
        }

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
    }
}
