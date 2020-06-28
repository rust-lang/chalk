//! Tests for `LoggingRustIrDatabase` which tests its functionality to record
//! types and stubs.
//!
//! Each tests records the trait solver solving something, and then runs the
//! solver on the output `LoggingRustIrDatabase` writes.These tests _don't_ test
//! that the output program is identical to the input, only that the resulting
//! program allows solving the same goals.
//!
//! Note that this does not, and should not, test the majority of the rendering
//! code. The code to render specific items and syntax details is rigorously
//! tested in `tests/display/`.
#[macro_use]
mod util;

#[test]
fn records_struct_trait_and_impl() {
    logging_db_output_sufficient! {
        program {
            struct S {}

            trait Trait {}

            impl Trait for S {}
        }

        goal {
            S: Trait
        } yields {
            "Unique"
        }
    }
}

#[test]
fn records_opaque_type() {
    logging_db_output_sufficient! {
        program {
            struct S {}

            trait Trait {}
            impl Trait for S {}

            opaque type Foo: Trait = S;
        }

        goal {
            Foo: Trait
        } yields {
            "Unique"
        }
    }
}

#[test]
fn records_fn_def() {
    logging_db_output_sufficient! {
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
fn records_generics() {
    logging_db_output_sufficient! {
        program {
            struct Foo<T> {}
            trait Bar {}
            impl Bar for Foo<()> {}
        }

        goal {
            Foo<()>: Bar
        } yields {
            "Unique"
        }
        goal {
            Foo<i32>: Bar
        } yields {
            "No possible solution"
        }
    }
}

#[test]
fn records_parents_parent() {
    logging_db_output_sufficient! {
        program {
            struct S {}

            trait Grandparent {}
            trait Parent where Self: Grandparent {}
            trait Child where Self: Parent {}
            impl Grandparent for S {}
            impl Parent for S {}
            impl Child for S {}
        }

        goal {
            S: Child
        } yields {
            "Unique"
        }
    }
}

#[test]
fn records_associated_type_bounds() {
    logging_db_output_sufficient! {
        program {
            trait Foo {
                type Assoc: Bar;
            }
            trait Bar {

            }

            struct S {}
            impl Foo for S {
                type Assoc = S;
            }
            impl Bar for S {}
        }

        goal {
            S: Foo
        } yields {
            "Unique"
        }
    }
}

#[test]
fn records_generic_impls() {
    logging_db_output_sufficient! {
        program {
            struct S {}
            struct V {}

            trait Foo {}
            trait Bar {}

            impl Foo for S {}

            impl<T> Bar for T where T: Foo {

            }
        }

        goal {
            S: Bar
        } yields {
            "Unique"
        }
    }

    logging_db_output_sufficient! {
        program {
            struct S {}
            struct V {}

            trait Foo {}
            trait Bar {}

            impl Foo for S {}

            impl<T> Bar for T where T: Foo {

            }
        }

        goal {
            V: Bar
        } yields {
            "No possible solution"
        }
    }
}

#[test]
fn catches_assoc_type_bounds_not_mentioned() {
    logging_db_output_sufficient! {
        program {
            trait Foo {
                type Assoc: Bar;
            }
            trait Bar {}
            impl Foo for () {
                type Assoc = ();
            }
        }

        goal {
            (): Foo
        } yields {
            "Unique"
        }
    }
}

#[test]
fn catches_assoc_type_values_not_mentioned() {
    logging_db_output_sufficient! {
        program {
            trait Foo {
                type Assoc;
            }
            struct Baz {}
            impl Foo for () {
                type Assoc = Baz;
            }
        }

        goal {
            (): Foo
        } yields {
            "Unique"
        }
    }
}

#[test]
fn stubs_types_from_opaque_ty_bounds() {
    logging_db_output_sufficient! {
        program {
            trait Foo {}
            trait Fuu {}
            struct Baz {}
            opaque type Bar: Foo + Fuu = Baz;
        }

        goal {
            Bar: Foo
        } yields {
            "Unique"
        }
    }
}

#[test]
fn does_not_need_necessary_separate_impl() {
    // this should leave out "impl Bar for Fox" and the result should pass the
    // test (it won't be well-formed, but that's OK.)
    logging_db_output_sufficient! {
        program {
            trait Box {
                type Assoc: Bar;
            }
            trait Bar {}

            struct Foo {}
            impl Box for Foo {
                type Assoc = Fox;
            }

            struct Fox {}
            impl Bar for Fox {}
        }

        goal {
            Foo: Box
        } yields {
            "Unique"
        }
    }
}
