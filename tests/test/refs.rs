use super::*;

#[test]
fn immut_refs_are_well_formed() {
    test! {
        program { }

        goal {
            forall<'a, T> { WellFormed(&'a T) }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}

#[test]
fn immut_refs_are_sized() {
    test! {
        program {
            #[lang(sized)]
            trait Sized { }
        }

        goal {
            forall<'a, T> { &'a T: Sized }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}

#[test]
fn mut_refs_are_well_formed() {
    test! {
        program { }

        goal {
            forall<'a, T> { WellFormed(&'a mut T) }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}

#[test]
fn mut_refs_are_sized() {
    test! {
        program {
            #[lang(sized)]
            trait Sized { }
        }

        goal {
            forall<'a, T> { &'a mut T: Sized }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}
