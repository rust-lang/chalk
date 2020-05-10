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
