use crate::test::*;

#[test]
fn tuples_are_sized() {
    test! {
        program {
            #[lang(sized)]
            trait Sized { }

            trait Foo {}
        }

        goal {
            ([u8],): Sized
        } yields {
            "No possible solution"
        }

        goal {
            (u8, [u8]): Sized
        } yields {
            "No possible solution"
        }

        // It should not be well-formed because for tuples, only
        // the last element is allowed not to be Sized.
        goal {
            ([u8], u8): Sized
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

        goal {
            exists<T> { (T, u8): Sized }
        } yields {
            "Unique; for<?U0> { substitution [?0 := ^0.0], lifetime constraints [] }"
        }

        goal {
            forall<T> { (T, u8): Sized }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            forall<T> { (u8, T): Sized }
        } yields {
            "No possible solution"
        }

        goal {
            forall<T> { if (T: Sized) { (u8, T): Sized } }
        } yields {
            "Unique; substitution [], lifetime constraints []"
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
fn scalars_are_sized() {
    test! {
        program {
            #[lang(sized)] trait Sized { }
        }

        goal { i8: Sized } yields { "Unique" }
        goal { i16: Sized } yields { "Unique" }
        goal { i32: Sized } yields { "Unique" }
        goal { i64: Sized } yields { "Unique" }
        goal { i128: Sized } yields { "Unique" }
        goal { isize: Sized } yields { "Unique" }
        goal { u8: Sized } yields { "Unique" }
        goal { u16: Sized } yields { "Unique" }
        goal { u32: Sized } yields { "Unique" }
        goal { u64: Sized } yields { "Unique" }
        goal { u128: Sized } yields { "Unique" }
        goal { usize: Sized } yields { "Unique" }
        goal { f32: Sized } yields { "Unique" }
        goal { f64: Sized } yields { "Unique" }
        goal { bool: Sized } yields { "Unique" }
        goal { char: Sized } yields { "Unique" }
    }
}
