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
fn tuples_are_copy() {
    test! {
        program {
            #[lang(copy)]
            trait Copy { }

            struct S {}

            impl Copy for u8 {}
        }

        goal {
            ([u8],): Copy
        } yields {
            "No possible solution"
        }

        goal {
            (u8, [u8]): Copy
        } yields {
            "No possible solution"
        }

        goal {
            ([u8], u8): Copy
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

        goal {
            exists<T> { (T, u8): Copy }
        } yields {
            "Ambiguous"
        }

        goal {
            forall<T> { if (T: Copy) { (T, u8): Copy } }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}

#[test]
fn tuples_are_sized() {
    test! {
        program {
            #[lang(sized)]
            trait Sized { }
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
fn tuples_are_clone() {
    test! {
        program {
            #[lang(clone)]
            trait Clone { }

            struct S {}

            impl Clone for u8 {}
        }

        goal {
            ([u8],): Clone
        } yields {
            "No possible solution"
        }

        goal {
            (u8, [u8]): Clone
        } yields {
            "No possible solution"
        }

        goal {
            ([u8], u8): Clone
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

        goal {
            exists<T> { (T, u8): Clone }
        } yields {
            "Ambiguous"
        }

        goal {
            forall<T> { if (T: Clone) { (T, u8): Clone } }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}

#[test]
fn tuples_are_wf() {
    test! {
        program {
            #[lang(sized)]
            trait Sized { }
        }

        goal {
            WellFormed(())
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            WellFormed((u8,))
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            WellFormed((u8, u8))
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            WellFormed(([u8],))
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            WellFormed((u8, [u8]))
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            WellFormed(([u8], u8))
        } yields {
            "No possible solution"
        }

        goal {
            exists<T> { WellFormed((T, u8)) }
        } yields {
            "Ambiguous; no inference guidance"
        }

        goal {
            forall<T> { WellFormed((T, u8)) }
        } yields {
            "No possible solution"
        }

        goal {
            forall<T> { if (T: Sized) { WellFormed((T, u8)) } }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}
