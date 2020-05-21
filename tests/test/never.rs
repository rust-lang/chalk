use super::*;

#[test]
fn never_is_well_formed() {
    test! {
        program { }
        goal {
            WellFormed(!)
        } yields {
            "Unique"
        }
    }
}
