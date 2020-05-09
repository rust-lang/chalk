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

#[test]
fn slices_are_not_copy() {
    test! {
        program {
            #[lang(copy)]
            trait Copy { }
        }

        goal {
            forall<T> { not { [T]: Copy } }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}

#[test]
fn slices_are_not_clone() {
    test! {
        program {
            #[lang(clone)]
            trait Clone { }
        }

        goal {
            forall<T> { not { [T]: Clone } }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}
