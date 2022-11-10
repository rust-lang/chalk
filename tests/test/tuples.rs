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
            expect![["Unique"]]
        }

        goal {
            (): Foo
        } yields {
            expect![["Unique"]]
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
            expect![["Unique"]]
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
            expect![["No possible solution"]]
        }

        goal {
            (u8, [u8]): Copy
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            ([u8], u8): Copy
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            (): Copy
        } yields {
            expect![["Unique"]]
        }

        goal {
            (u8,): Copy
        } yields {
            expect![["Unique"]]
        }

        goal {
            (u8, u8): Copy
        } yields {
            expect![["Unique"]]
        }

        goal {
            exists<T> { (T, u8): Copy }
        } yields {
            expect![["Ambiguous; no inference guidance"]]
        }

        goal {
            forall<T> { if (T: Copy) { (T, u8): Copy } }
        } yields {
            expect![["Unique"]]
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
            expect![["No possible solution"]]
        }

        goal {
            (u8, [u8]): Sized
        } yields {
            expect![["No possible solution"]]
        }

        // It should not be well-formed because for tuples, only
        // the last element is allowed not to be Sized.
        goal {
            ([u8], u8): Sized
        } yields {
            expect![["Unique"]]
        }

        goal {
            (): Sized
        } yields {
            expect![["Unique"]]
        }

        goal {
            (u8,): Sized
        } yields {
            expect![["Unique"]]
        }

        goal {
            (u8, u8): Sized
        } yields {
            expect![["Unique"]]
        }

        goal {
            exists<T> { (T, u8): Sized }
        } yields {
            expect![["Unique; for<?U0> { substitution [?0 := ^0.0] }"]]
        }

        goal {
            forall<T> { (T, u8): Sized }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<T> { (u8, T): Sized }
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            forall<T> { if (T: Sized) { (u8, T): Sized } }
        } yields {
            expect![["Unique"]]
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
            expect![["No possible solution"]]
        }

        goal {
            (u8, [u8]): Clone
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            ([u8], u8): Clone
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            (): Clone
        } yields {
            expect![["Unique"]]
        }

        goal {
            (u8,): Clone
        } yields {
            expect![["Unique"]]
        }

        goal {
            (u8, u8): Clone
        } yields {
            expect![["Unique"]]
        }

        goal {
            exists<T> { (T, u8): Clone }
        } yields {
            expect![["Ambiguous; no inference guidance"]]
        }

        goal {
            forall<T> { if (T: Clone) { (T, u8): Clone } }
        } yields {
            expect![["Unique"]]
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
            expect![["Unique"]]
        }

        goal {
            WellFormed((u8,))
        } yields {
            expect![["Unique"]]
        }

        goal {
            WellFormed((u8, u8))
        } yields {
            expect![["Unique"]]
        }

        goal {
            WellFormed(([u8],))
        } yields {
            expect![["Unique"]]
        }

        goal {
            WellFormed((u8, [u8]))
        } yields {
            expect![["Unique"]]
        }

        goal {
            WellFormed(([u8], u8))
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            exists<T> { WellFormed((T, u8)) }
        } yields {
            expect![["Ambiguous; no inference guidance"]]
        }

        goal {
            forall<T> { WellFormed((T, u8)) }
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            forall<T> { if (T: Sized) { WellFormed((T, u8)) } }
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn tuples_implement_tuple_trait() {
    test! {
        program {
            #[lang(tuple_trait)]
            trait Tuple { }
        }

        goal {
            (): Tuple
        } yields {
            expect![["Unique"]]
        }

        goal {
            (u8,): Tuple
        } yields {
            expect![["Unique"]]
        }

        goal {
            (i32, i32): Tuple
        } yields {
            expect![["Unique"]]
        }

        goal {
            ([u8],): Tuple
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<T> { (T,): Tuple }
        } yields {
            expect![["Unique"]]
        }

        goal {
            i32: Tuple
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            exists<T> { T: Tuple }
        } yields {
            expect![["Ambiguous; no inference guidance"]]
        }
    }
}
