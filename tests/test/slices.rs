use super::*;

#[test]
fn slices_are_not_sized() {
    test! {
        program {
            #[lang(sized)]
            trait Sized { }
        }

        goal {
            forall<T> { not { [T]: Sized } }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}

#[test]
fn slices_are_well_formed() {
    test! {
        program {
        }

        goal {
            forall<T> { WellFormed([T]) }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}
