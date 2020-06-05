use super::*;

#[test]
fn function_implement_fn_once() {
    test! {
        program {
            #[lang(fn_once)]
            trait FnOnce<Args> { }
        }

        goal {
            fn(u8): FnOnce<(u8,)>
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            fn(u8, u32): FnOnce<(u8,u32)>
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            fn(i32): FnOnce<(bool,)>
        } yields {
            "No possible solution"
        }

        goal {
            forall<'a> {
                for<'b> fn(&'b u8): FnOnce<(&'a u8,)>
            }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            forall<'a, 'b> {
                for<'c> fn(&'c u8, &'c i32): FnOnce<(&'a u8, &'b i32)>
            }
        } yields {
            "Unique; substitution [], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!1_1 }, InEnvironment { environment: Env([]), goal: '!1_1: '!1_0 }]"
        }

        goal {
            forall<T, U> {
                fn(T, T): FnOnce<(T, U)>
            }
        } yields {
            "No possible solution"
        }

        goal {
            forall<T, U> {
                fn(T, U): FnOnce<(T, T)>
            }
        } yields {
            "No possible solution"
        }
    }
}

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
            fn([u8]): Sized
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
