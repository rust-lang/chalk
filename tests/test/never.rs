use super::*;

#[test]
fn never_is_well_formed() {
    test! {
        goal {
            WellFormed(!)
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn never_is_sized() {
    test! {
        program {
            #[lang(sized)]
            trait Sized { }
        }
        goal {
            !: Sized
        } yields {
            expect![["Unique"]]
        }
    }
}
