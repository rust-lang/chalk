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
