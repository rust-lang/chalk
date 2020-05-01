use crate::test::*;

#[test]
fn tuples_are_clone() {
    test! {
        program {
            #[non_enumerable] // see above
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
