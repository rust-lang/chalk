use super::*;

#[test]
fn functions_are_sized() {
    test! {
        program {
            #[lang(sized)]
            trait Sized { }

            trait Copy {}
        }

        goal {
            fn(()): Sized
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            fn(dyn Copy): Sized
        } yields {
            "Unique; substitution [], lifetime constraints []"
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
            "No possible solution"
        }

        goal {
            WellFormed(baz<Xyzzy>)
        } yields {
            "Unique"
        }

        goal {
            WellFormed(garply)
        } yields {
            "Unique"
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
            "Unique"
        }
    }
}

#[test]
fn fn_ptr_implements_fn_traits() {
    test! {
        program {
            #[lang(fn)]
            trait Fn<Args> { }

            #[lang(fnmut)]
            trait FnMut<Args> { }

            #[lang(fnonce)]
            trait FnOnce<Args> { }

            struct Foo { }
        }

        goal {
            fn(i32): Fn<(i32)>
        } yields {
            "Unique"
        }
        goal {
            fn(()): FnMut<(())>
        } yields {
            "Unique"
        }
        goal {
            fn(()): FnOnce<(())>
        } yields {
            "Unique"
        }

        goal {
            fn(()): Fn<(i32)>
        } yields {
            "No possible solution"
        }

        goal {
            fn(()): FnMut<(i32)>
        } yields {
            "No possible solution"
        }

        goal {
            fn(()): FnOnce<(i32)>
        } yields {
            "No possible solution"
        }

        goal {
            fn((i32, i32)): Fn<((i32, i32))>
        } yields {
            "Unique"
        }

        goal {
            fn((i32, i32, (u32, u32))): Fn<(i32, i32, (u32, u32))>
        } yields {
            "Unique"
        }

        goal {
            fn((i32, i32, (u32, u32))): Fn<(i32, i32, (u32, u32, u32))>
        } yields {
            "No possible solution"
        }

        goal {
            exists<Args> {
                Foo: Fn<Args>
            }
        } yields {
            "No possible solution"
        }
    }
}
