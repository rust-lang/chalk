use super::*;

#[test]
fn immut_refs_are_well_formed() {
    test! {
        program {
            struct A { }
        }

        goal {
            forall<'a, T> {
                WellFormed(&'a T)
            }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: !1_1: '!1_0 }]"]]
        }

        goal {
            exists<'a> {
                WellFormed(&'a A)
            }
        } yields {
            expect![["Unique; for<?U0> { substitution [?0 := '^0.0], lifetime constraints [InEnvironment { environment: Env([]), goal: A: '^0.0 }] }"]]
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
            expect![["Unique"]]
        }
    }
}

#[test]
fn mut_refs_are_well_formed() {
    test! {
        goal {
            forall<'a, T> { WellFormed(&'a mut T) }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: !1_1: '!1_0 }]"]]
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
            expect![["Unique"]]
        }
    }
}
