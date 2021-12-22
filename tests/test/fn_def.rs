use super::*;

#[test]
fn fn_def_is_well_formed() {
    test! {
        program {
            fn foo();
        }
        goal {
            WellFormed(foo)
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn fn_def_is_sized() {
    test! {
        program {
            #[lang(sized)]
            trait Sized { }

            fn foo();
        }
        goal {
            foo: Sized
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn fn_def_is_copy() {
    test! {
        program {
            #[lang(copy)]
            trait Copy { }

            fn foo();
        }
        goal {
            foo: Copy
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn fn_def_is_clone() {
    test! {
        program {
            #[lang(clone)]
            trait Clone { }

            fn foo();
        }
        goal {
            foo: Clone
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn fn_def_implements_fn_traits() {
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

            fn foo();
            fn bar(one: i32);
            fn baz(one: i32) -> u8;
        }

        goal {
            foo: Fn<()>
        } yields {
            expect![["Unique"]]
        }

        goal {
            Normalize(<foo as FnOnce<()>>::Output -> ())
        } yields {
            expect![["Unique"]]
        }

        goal {
            bar: Fn<(i32,)>
        } yields {
            expect![["Unique"]]
        }

        goal {
            Normalize(<bar as FnOnce<(i32,)>>::Output -> ())
        } yields {
            expect![["Unique"]]
        }

        goal {
            baz: Fn<(i32,)>
        } yields {
            expect![["Unique"]]
        }

        goal {
            Normalize(<baz as FnOnce<(i32,)>>::Output -> u8)
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn generic_fn_implements_fn_traits() {
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

            fn foo<T>(t: T) -> T;
        }

        goal {
            exists<T> { foo<T>: Fn<(T,)> }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<T> { foo<T>: Fn<(T,)> }
        } yields {
            expect![["Unique"]]
        }

        goal {
            exists<T> { Normalize(<foo<T> as FnOnce<(T,)>>::Output -> T) }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<T> { Normalize(<foo<T> as FnOnce<(T,)>>::Output -> T) }
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn fn_defs() {
    test! {
        program {
            trait Foo { }

            struct Bar { }

            struct Xyzzy { }
            impl Foo for Xyzzy { }

            fn baz<T>(quux: T) -> T
                where T: Foo;

            fn garply(thud: i32) -> i32;
        }

        goal {
            WellFormed(baz<Bar>)
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            WellFormed(baz<Xyzzy>)
        } yields {
            expect![["Unique"]]
        }

        goal {
            WellFormed(garply)
        } yields {
            expect![["Unique"]]
        }

    }
}

#[test]
fn fn_def_implied_bounds_from_env() {
    test! {
        program {
            trait Foo { }

            struct Bar { }
            impl Foo for Bar { }

            fn baz<T>() where T: Foo;
        }
        goal {
            if (FromEnv(baz<Bar>)) {
                Bar: Foo
            }
        } yields {
            expect![["Unique"]]
        }
    }
}
