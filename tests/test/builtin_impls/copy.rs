use crate::test::*;

#[test]
fn tuples_are_copy() {
    test! {
        program {
            // FIXME: If we don't declare Copy non-enumerable, `exists<T> { T:
            // Copy }` gives wrong results, because it doesn't consider the
            // built-in impls.
            #[non_enumerable]
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
