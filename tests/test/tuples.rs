use super::*;

#[test]
fn tuple_trait_impl() {
    test! {
        program {
            trait Foo { }
            struct S1 { }
            impl Foo for (S1, S1) { }
            impl Foo for () { }
        }
        goal {
            (S1, S1): Foo
        } yields {
            "Unique"
        }

        goal {
            (): Foo
        } yields {
            "Unique"
        }
    }
    test! {
        program {
            trait Foo { }
            impl Foo for (i32, i32, (i32,)) { }
        }

        goal {
            (i32, i32, (i32, )): Foo
        } yields {
            "Unique"
        }
    }
}

#[test]
fn tuples_are_sized() {
    test! {
        program {
            #[lang(sized)]
            trait Sized { }

            trait Foo {}
        }

        goal {
            (dyn Foo,): Sized
        } yields {
            "No possible solution"
        }

        goal {
            (u8, dyn Foo): Sized
        } yields {
            "No possible solution"
        }

        // It should not be well-formed because for tuples, only
        // the last element is allowed not to be Sized.
        goal {
            (dyn Foo, u8): Sized
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            (): Sized
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            (u8,): Sized
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            (u8, u8): Sized
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}

#[test]
fn tuples_are_copy() {
    test! {
        program {
            #[lang(copy)]
            trait Copy { }

            trait Foo {}

            impl Copy for u8 {}
        }

        goal {
            (dyn Foo,): Copy
        } yields {
            "No possible solution"
        }

        goal {
            (u8, dyn Foo): Copy
        } yields {
            "No possible solution"
        }

        goal {
            (dyn Foo, u8): Copy
        } yields {
            "No possible solution"
        }

        goal {
            (): Copy
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            (u8,): Copy
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            (u8, u8): Copy
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}

#[test]
fn tuples_are_clone() {
    test! {
        program {
            #[lang(clone)]
            trait Clone { }

            trait Foo {}

            impl Clone for u8 {}
        }

        goal {
            (dyn Foo,): Clone
        } yields {
            "No possible solution"
        }

        goal {
            (u8, dyn Foo): Clone
        } yields {
            "No possible solution"
        }

        goal {
            (dyn Foo, u8): Clone
        } yields {
            "No possible solution"
        }

        goal {
            (): Clone
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            (u8,): Clone
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            (u8, u8): Clone
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}
