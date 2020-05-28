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
            "Unique"
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
            "Unique"
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
            "Unique"
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
            "Unique"
        }
    }
}
